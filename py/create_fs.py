#!/usr/bin/env python3
import sys
import os
import struct

def align_dir(dir):
    if len(dir) % 4 != 0:
        dir += bytes(4 - (len(dir) % 4))

def create_dir(path):
    dir = bytearray()
    with os.scandir(path) as it:
        for entry in it:
            name = bytes(entry.name, 'utf-8')
            if len(name) > 255:
                raise RuntimeError('Name length is too big')
            dir += bytes([len(name)])
            dir += name
            align_dir(dir)
            if not (entry.is_file() or entry.is_dir()):
                raise RuntimeError('Unknown object in dir')
            is_dir = entry.is_dir()
            if is_dir:
               data = create_dir(entry.path)
            else:
                with open(entry.path, mode='rb') as f:
                    data = f.read()
            flag_size = len(data)
            if is_dir:
                flag_size |= 0x2000000
            dir += struct.pack("<I", flag_size)
            dir += data
            align_dir(dir)
    dir += bytes(4)
    return dir

root_dir = create_dir(sys.argv[1])
with open(sys.argv[2], 'wb') as out:
    out.write(root_dir)
