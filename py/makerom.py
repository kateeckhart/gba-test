#!/usr/bin/env python3
import subprocess
import tempfile
import re
import sys
import struct
import math

with tempfile.TemporaryDirectory() as fragments:
    subprocess.run(["arm-none-eabi-objcopy", sys.argv[1], "--dump-section", "header={0}/header.bin".format(fragments), "--dump-section", "read_only={0}/read_only.bin".format(fragments), "--dump-section", "modify={0}/modify.bin".format(fragments), "--dump-section", "fast_mem={0}/fast_mem.bin".format(fragments), "--dump-section", "exidx={0}/exidx.bin".format(fragments)], check=True)

    symbols = subprocess.run(["arm-none-eabi-nm", sys.argv[1], "-P"], capture_output=True, check=True, text=True).stdout

    rom = bytearray()
    with open("{0}/header.bin".format(fragments), mode="rb") as header:
        rom += header.read()
    with open("{0}/read_only.bin".format(fragments), mode="rb") as read_only:
        rom += read_only.read()
    with open("{0}/exidx.bin".format(fragments), mode="rb") as exidx:
        rom += exidx.read()
    modify_loc = len(rom)
    with open("{0}/modify.bin".format(fragments), mode="rb") as modify_file:
        modify = modify_file.read()
        rom += modify
    fast_mem_loc = len(rom)
    with open("{0}/fast_mem.bin".format(fragments), mode="rb") as fast_mem_file:
        fast_mem = fast_mem_file.read()
        rom += fast_mem

    load_modify_str = re.search("__load_modify_begin T ([0-9a-fA-F]+)", symbols, flags=re.M).group(1)
    load_modify_loc = int(load_modify_str, base=16) - 0x8000000
    rom[load_modify_loc: load_modify_loc + 4] = struct.pack("<I", modify_loc + 0x8000000)

    load_fast_mem_str = re.search("__load_fast_mem_begin T ([0-9a-fA-F]+)", symbols, flags=re.M).group(1)
    load_fast_mem_loc = int(load_fast_mem_str, base=16) - 0x8000000
    rom[load_fast_mem_loc: load_fast_mem_loc + 4] = struct.pack("<I", fast_mem_loc + 0x8000000)

    rounded_size = 2 ** math.ceil(math.log2(len(rom)))
    if rounded_size < 4 * 1024 * 1024:
        rounded_size = 4 * 1024 * 1024
    rom += bytes(rounded_size - len(rom))

    checksum_data = rom[0xa0: 0xbd]
    checksum = 0
    for b in checksum_data:
        checksum -= b

    checksum -= 0x19
    checksum &= 0xFF
    rom[0xbd] = checksum

    with open(sys.argv[2], mode="wb") as rom_file:
        rom_file.write(rom)
