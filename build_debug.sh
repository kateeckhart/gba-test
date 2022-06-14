#!/usr/bin/env bash
python3 py/create_fs.py data data.bin
CC=clang rustup run nightly xargo build --target thumbv4t-none-eabi
arm-none-eabi-ld --whole-archive target/thumbv4t-none-eabi/debug/libgba_test.a -o debug-gba-test.elf --gc-sections -Tgba.LD
python3 py/makerom.py debug-gba-test.elf debug-gba-test.gba
