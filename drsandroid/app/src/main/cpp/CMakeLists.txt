# Sets the minimum version of CMake required to build your native library.
# This ensures that a certain set of CMake features is available to
# your build.
project(doukutsu-rs)
cmake_minimum_required(VERSION 3.22)

include(FetchContent)
FetchContent_Declare(
        Corrosion
        GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
        GIT_TAG v0.5.2
)
FetchContent_MakeAvailable(Corrosion)

set(DRSANDROID_ROOT "${CMAKE_SOURCE_DIR}/../../../..")

# Build SDL from the sources cargo already checked out for sdl2-sys, so the version always matches the Rust bindings.
execute_process(
        COMMAND "${Rust_CARGO_CACHED}" metadata --format-version 1 --manifest-path "${DRSANDROID_ROOT}/Cargo.toml"
        OUTPUT_VARIABLE CARGO_METADATA
        RESULT_VARIABLE CARGO_METADATA_RESULT
)
if(NOT CARGO_METADATA_RESULT EQUAL 0)
  message(FATAL_ERROR "cargo metadata failed: ${CARGO_METADATA_RESULT}")
endif()
if(NOT CARGO_METADATA MATCHES "\"manifest_path\":\"([^\"]*sdl2-sys[/\\\\]+Cargo\\.toml)\"")
  message(FATAL_ERROR "Could not find sdl2-sys in cargo metadata")
endif()
string(REPLACE "\\\\" "/" SDL2_SYS_MANIFEST "${CMAKE_MATCH_1}")
get_filename_component(SDL2_SYS_DIR "${SDL2_SYS_MANIFEST}" DIRECTORY)
add_subdirectory("${SDL2_SYS_DIR}/SDL" "${CMAKE_BINARY_DIR}/SDL2")

function(configure_shared_stl lib_path so_base)
  message("Configuring STL ${so_base} for ${ANDROID_ABI}")
  configure_file(
    "${ANDROID_NDK}/toolchains/llvm/prebuilt/${ANDROID_HOST_TAG}/sysroot/usr/lib/${CMAKE_LIBRARY_ARCHITECTURE}/lib${so_base}.so"
    "${CMAKE_LIBRARY_OUTPUT_DIRECTORY}/lib${so_base}.so"
    COPYONLY)
endfunction()

if("${ANDROID_STL}" STREQUAL "libstdc++")
  # The default minimal system C++ runtime library.
elseif("${ANDROID_STL}" STREQUAL "gabi++_shared")
  # The GAbi++ runtime (shared).
  message(FATAL_ERROR "gabi++_shared was not configured by ndk-stl package")
elseif("${ANDROID_STL}" STREQUAL "stlport_shared")
  # The STLport runtime (shared).
  configure_shared_stl("stlport" "stlport_shared")
elseif("${ANDROID_STL}" STREQUAL "gnustl_shared")
  # The GNU STL (shared).
  configure_shared_stl("gnu-libstdc++/4.9" "gnustl_shared")
elseif("${ANDROID_STL}" STREQUAL "c++_shared")
  # The LLVM libc++ runtime (static).
  configure_shared_stl("llvm-libc++" "c++_shared")
else()
   message(FATAL_ERROR "STL configuration ANDROID_STL=${ANDROID_STL} is not supported")
endif()

corrosion_import_crate(MANIFEST_PATH "${DRSANDROID_ROOT}/Cargo.toml")

# Specifies a library name, specifies whether the library is STATIC or
# SHARED, and provides relative paths to the source code. You can
# define multiple libraries by adding multiple add_library() commands,
# and CMake builds them for you. When you build your app, Gradle
# automatically packages shared libraries with your APK.

add_library(libdrsandroid SHARED dummy.cpp)
target_link_libraries(libdrsandroid PRIVATE
        "-Wl,--whole-archive"
        drsandroid-static
        "-Wl,--no-whole-archive"
        android
        EGL
        log
        SDL2::SDL2
)
target_link_options(libdrsandroid PRIVATE "-s" "-fvisibility=hidden" "-u ANativeActivity_onCreate")
if ("$CMAKE_BUILD_TYPE" STREQUAL "Release" OR "$CMAKE_BUILD_TYPE" STREQUAL "RelWithDebInfo")
  target_compile_options(libdrsandroid PRIVATE "-flto -fuse-linker-plugin")
  target_link_options(libdrsandroid PRIVATE "-flto -fuse-linker-plugin")
endif()
set_target_properties(libdrsandroid PROPERTIES OUTPUT_NAME "drsandroid")
