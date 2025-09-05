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

#if(NOT ${ANDROID_STL} MATCHES "_shared")
#  return()
#endif()
#set(ENV{CC_x86_64} "x86_64-linux-android21-clang")
#set(ENV{CXX_x86_64} "x86_64-linux-android21-clang++")
#set(ENV{CC_aarch64} "aarch64-linux-android21-clang")
#set(ENV{CXX_aarch64} "aarch64-linux-android21-clang++")
#set(ENV{CC_x86} "i686-linux-android21-clang")
#set(ENV{CXX_x86} "i686-linux-android21-clang++")
#set(ENV{CC_arm} "armv7a-linux-androideabi21-clang")
#set(ENV{CXX_arm} "armv7a-linux-androideabi21-clang++")

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

set(DRSANDROID_ROOT "${CMAKE_SOURCE_DIR}/../../../..")
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
)
target_link_options(libdrsandroid PRIVATE "-s" "-fvisibility=hidden" "-u ANativeActivity_onCreate")
if ("$CMAKE_BUILD_TYPE" STREQUAL "Release" OR "$CMAKE_BUILD_TYPE" STREQUAL "RelWithDebInfo")
  target_compile_options(libdrsandroid PRIVATE "-flto -fuse-linker-plugin")
  target_link_options(libdrsandroid PRIVATE "-flto -fuse-linker-plugin")
endif()
set_target_properties(libdrsandroid PROPERTIES OUTPUT_NAME "drsandroid")