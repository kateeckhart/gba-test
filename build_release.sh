#!/usr/bin/env bash
python3 py/create_fs.py data data.bin
CC=clang rustup run nightly xargo build --release --target thumbv4t-none-eabi
arm-none-eabi-ld --whole-archive target/thumbv4t-none-eabi/release/libgba_test.a -o release-gba-test.elf --gc-sections -Tgba.LD
python3 py/makerom.py release-gba-test.elf release-gba-test.gba
