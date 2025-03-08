#!/bin/bash

cd "$(dirname "$0")" || exit
set -e

DARK_GRAY=$(tput setaf 8)
YELLOW=$(tput bold)$(tput setaf 3)
RESET=$(tput sgr0)

function message() {
    echo "${DARK_GRAY}----${RESET} ${YELLOW}$*${RESET}"
}

message "Compiling shaders..."
uam -s vert -o ../src/framework/shaders/deko3d/vertex_basic.dksh ../src/framework/shaders/deko3d/vertex_basic.glsl
uam -s frag -o ../src/framework/shaders/deko3d/fragment_textured.dksh ../src/framework/shaders/deko3d/fragment_textured.glsl
uam -s frag -o ../src/framework/shaders/deko3d/fragment_color.dksh ../src/framework/shaders/deko3d/fragment_color.glsl

message "Building crate..."
rustup run rust-switch cargo build -Z build-std=core,alloc,std,panic_abort --target aarch64-nintendo-switch.json

rm -f target/aarch64-nintendo-switch/debug/drshorizon.nro
rm -f target/aarch64-nintendo-switch/debug/drshorizon.nacp

message "Creating NACP..."
nacptool --create 'doukutsu-rs' 'doukutsu-rs contributors' '0.102.0' target/aarch64-nintendo-switch/debug/drshorizon.nacp

message "Running elf2nro..."
elf2nro target/aarch64-nintendo-switch/debug/drshorizon.elf target/aarch64-nintendo-switch/debug/drshorizon.nro \
  --icon=../res/crabsue-icon.jpg \
  --nacp=target/aarch64-nintendo-switch/debug/drshorizon.nacp

message "done!"
