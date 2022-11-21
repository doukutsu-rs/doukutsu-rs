#!/bin/bash

cd "$(dirname "$0")" || exit
set -e

rustup run rust-switch cargo build -Z build-std=core,alloc,std,panic_abort --target aarch64-nintendo-switch.json

rm -f target/aarch64-nintendo-switch/debug/drshorizon.nro
rm -f target/aarch64-nintendo-switch/debug/drshorizon.nacp

echo "Creating NACP..."
nacptool --create 'doukutsu-rs' 'doukutsu-rs contributors' '0.100.0' target/aarch64-nintendo-switch/debug/drshorizon.nacp

echo "Running elf2nro..."
elf2nro target/aarch64-nintendo-switch/debug/drshorizon.elf target/aarch64-nintendo-switch/debug/drshorizon.nro \
  --icon=../res/nx_icon.jpg \
  --nacp=target/aarch64-nintendo-switch/debug/drshorizon.nacp

echo "done."
