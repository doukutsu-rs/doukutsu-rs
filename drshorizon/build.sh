#!/bin/bash

cd "$(dirname "$0")" || exit
set -e

DARK_GRAY=$(tput setaf 8)
YELLOW=$(tput bold)$(tput setaf 3)
RESET=$(tput sgr0)

function message() {
    echo "${DARK_GRAY}----${RESET} ${YELLOW}$*${RESET}"
}

function is_bin_avail() {
  # Temporarily disable exit on error, otherwise the script will be terminated if the binary is missing
  set +e
  if $($1 >/dev/null 2>&1) && [ $? -eq 0 ]; then
    echo true
  else
    echo false
  fi
  set -e
}

# Parse arguments
BUILD_MODE="debug"
CARGO_FLAGS=""
FLAGS=""
IS_DOCKER=false

# Check if Docker Compose is available
IS_COMPOSE_AVAIL=$(is_bin_avail "docker-compose")

function print_help() {
  echo -e "$(basename "$0") [-h] [-r] [-V] [<cargo args>...]\n"
  echo -e "Build script for the doukutsu-rs Horizon port.\n"
  echo "Arguments:"
  echo "  --docker          Run build in the Docker container."
  echo "  -r, --release     Build in release mode."
  echo "  -V, --version     Print version of build tools."
  echo "  -?, -h, --help    Prints help message."
}

function print_mode() {
  echo "Build mode: ${BUILD_MODE}"
  if $IS_DOCKER && $IS_COMPOSE_AVAIL; then
     echo -e "Running via Docker Compose\n"
  elif $IS_DOCKER; then
    echo -e "Running via Docker\n"
  fi
}

function print_version() {
  local pacman_bin="pacman"
  if $(is_bin_avail "dkp-pacman"); then
    pacman_bin="dkp-pacman"
  fi

  echo "===TOOLCHAIN==="
  # Cargo may generate warnings, that cannot be suppressed by `-q` flag, so we're suppressing the stderr output
  cargo --version 2>/dev/null
  rustc --version

  echo -e "\n===DEVKITPRO==="
  echo $pacman_bin $($pacman_bin --version | grep -oP 'Pacman v\K[0-9.]+')
  # Some executables from the devkitPro toolchain don't have a flag to print the version,
  # so we need to fetch it from the package manager
  $pacman_bin -Q deko3d libnx switch-tools
  uam --version

  if $(is_bin_avail "docker"); then
    echo -e "\n===DOCKER==="
    docker --version

    if $(is_bin_avail "docker-compose"); then
       docker-compose --version
    fi
  fi

}

function run_docker() {
  if $IS_COMPOSE_AVAIL; then
    docker compose run --rm --remove-orphans rust-switch ./build.sh $FLAGS
  else
    docker build -t rust-switch .
    docker run -it \
      -v "../:/workspace" \
      -v "${CARGO_HOME:-${HOME}/.cargo}/registry:/usr/local/cargo/registry" \
      -v "${CARGO_HOME:-${HOME}/.cargo}/git:/usr/local/cargo/git" \
      rust-switch \
      ./build.sh $FLAGS
  fi
}

while [[ $# -gt 0 ]]; do
  case $1 in
    --docker)
      IS_DOCKER=true
      shift
      ;;
    -r|--release)
      BUILD_MODE="release"
      CARGO_FLAGS+=" -r"
      FLAGS+=" $1"
      shift
      ;;
    -V|--version)
      print_version
      exit 0
      ;;
    -?|-h|--help)
      print_help
      exit 0
      ;;
    *)
      CARGO_FLAGS+=" $1"
      FLAGS+=" $1"
      shift
      ;;
  esac
done

print_mode
if $IS_DOCKER; then
  run_docker
  exit 0
fi

# Building the port
message "Compiling shaders..."
uam -s vert -o ../src/framework/shaders/deko3d/vertex_basic.dksh ../src/framework/shaders/deko3d/vertex_basic.glsl
uam -s frag -o ../src/framework/shaders/deko3d/fragment_textured.dksh ../src/framework/shaders/deko3d/fragment_textured.glsl
uam -s frag -o ../src/framework/shaders/deko3d/fragment_color.dksh ../src/framework/shaders/deko3d/fragment_color.glsl

message "Building crate..."
cargo build -Z build-std=core,alloc,std,panic_abort -Z json-target-spec --target aarch64-nintendo-switch.json $CARGO_FLAGS

rm -f target/aarch64-nintendo-switch/$BUILD_MODE/drshorizon.nro
rm -f target/aarch64-nintendo-switch/$BUILD_MODE/drshorizon.nacp

message "Creating NACP..."
nacptool --create 'doukutsu-rs' 'doukutsu-rs contributors' '1.0.0' target/aarch64-nintendo-switch/$BUILD_MODE/drshorizon.nacp

message "Running elf2nro..."
elf2nro target/aarch64-nintendo-switch/$BUILD_MODE/drshorizon.elf target/aarch64-nintendo-switch/$BUILD_MODE/drshorizon.nro \
  --icon=../res/crabsue-icon.jpg \
  --nacp=target/aarch64-nintendo-switch/$BUILD_MODE/drshorizon.nacp

message "done!"
