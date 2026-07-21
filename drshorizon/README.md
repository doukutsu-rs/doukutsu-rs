Experimental.

ld script and `.specs` taken from devkitPro (the linker script is modified).

# Known issues
- Crashes hbloader.

# Build
More detailed build instructions can be found in the [docs](https://doukutsu-rs.gitbook.io/docs/modders-handbook/initial-setup-and-compiling/building-the-ports#nintendo-switch-horizon).

## Dependencies
- Rust toolchain with the Switch patches ([link](https://github.com/doukutsu-rs/rust-hos/releases))
- Rust nightly toolchain (the `nightly` channel). The `cargo` binary will be used from it, since the patched toolchain miss it. Any other toolchain, even if it's nightly (like `nightly-2025-10-24`), won't work
- Switch toolchain by [devkitPro](https://devkitpro.org/wiki/Getting_Started#Setup) (install the `switch-dev` package)
- `libclang-dev` (on Debian; for other distros, find a package that provides `libclang.so`)
- Clang (it provides some header files, such as `stddef.h`)

## Standard build
Compile a debug build:
```
./build.sh
```

---

Compile an optimized (release) build:
```
./build.sh -r
```

---

All flags not listed in the help message are passed to the `cargo build` command (in this example, the `-vv` and `-j3` flags will be passed to `cargo build`):
```
./build.sh -r -vv -j3
```

## Building in a Docker container
To build in a Docker container, enter the `drshorizon` folder. Then run:
```
./build.sh --docker
```

This will build the port in a Docker container. The cargo cache and project directory will be mounted from the host, so the build artifacts can be found in the `target` folder, just as they would be in a regular build.

If you have Docker Compose installed, the script will run it using config params from `composer.yaml`. The created container will be removed on exit.

Otherwise the script will build the container image with `docker build` and run it via `docker run`.

# Notes for maintainers
1. The dependency on Clang is caused by the `deko3d-sys` crate, which ignores the presence of GCC/glibc headers.
2. The toolchain intended for use in a container should be built on a Debian Bookworm virtual machine, since this is the version used by the devkitPro container.

   Building the toolchain on newer systems will result in the toolchain’s libraries being linked against a newer version of glibc than the one present in the container, which in turn will make it impossible to build the port using this toolchain, as it will crash with a segfault.

   Attempting to build the toolchain inside the container results either in compilation errors or in the generation of artifacts that cannot be run/loaded properly (they crash with core dump).
