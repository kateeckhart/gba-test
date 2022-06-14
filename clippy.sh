#!/usr/bin/env bash
CC=clang rustup run nightly xargo clippy --target thumbv4t-none-eabi
