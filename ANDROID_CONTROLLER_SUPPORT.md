# Android Controller Support Investigation

## Project Overview

**Repository:** doukutsu-rs (Cave Story Rust reimplementation)
**Goal:** Enable gamepad/controller support on Android builds
**Date:** 2024-12-21

## Current Architecture

### Backend System

The project uses a pluggable backend architecture:

| Backend | File | Platforms | Gamepad Support |
|---------|------|-----------|-----------------|
| SDL2 | `backend_sdl2.rs` | Windows, macOS, Linux | Full support |
| Glutin | `backend_glutin.rs` | Windows, macOS, Linux, Android | Touch + Keyboard only |
| Horizon | `backend_horizon.rs` | Nintendo Switch | Full support |

### Android Build Configuration

- **Current backend:** `backend-glutin` (configured in `drsandroid/Cargo.toml`)
- **Native integration:** Uses `ndk-glue` with Android's `NativeActivity`
- **Java Activity:** `GameActivity.java` extends `NativeActivity`

### Gamepad Abstraction Layer

The project has a well-designed gamepad abstraction:

```
src/framework/gamepad.rs          - Platform-independent gamepad context
src/framework/backend.rs          - BackendGamepad trait definition
src/input/gamepad_player_controller.rs - Player input handling
```

The `BackendGamepad` trait is minimal:
```rust
pub trait BackendGamepad {
    fn set_rumble(&mut self, low_freq: u16, high_freq: u16, duration_ms: u32) -> GameResult;
    fn instance_id(&self) -> u32;
}
```

## Attempted Solution: SDL2 Backend

### Approach

Changed `drsandroid/Cargo.toml` to use SDL2 backend:
```toml
# From:
features = ["default-base", "backend-glutin", "webbrowser"]
# To:
features = ["default-base", "backend-sdl", "android", "webbrowser"]
```

### Build Environment Setup

Required components:
- Android NDK 25.2.9519653
- Android SDK with platform 33
- Java 17 (via homebrew)
- Rust Android targets: `aarch64-linux-android`, `armv7-linux-androideabi`, etc.
- `cargo-ndk` tool
- CMake (from Android SDK)

Key environment variables:
```bash
export ANDROID_NDK_HOME=/opt/homebrew/share/android-commandlinetools/ndk/25.2.9519653
export RUSTC=/Users/brennanmoore/.cargo/bin/rustc  # Important: use rustup's rustc
export PATH="/opt/homebrew/share/android-commandlinetools/cmake/3.22.1/bin:$PATH"
```

### Build Command

```bash
env ANDROID_NDK_HOME=/opt/homebrew/share/android-commandlinetools/ndk/25.2.9519653 \
    RUSTC=/Users/brennanmoore/.cargo/bin/rustc \
    PATH="/opt/homebrew/share/android-commandlinetools/cmake/3.22.1/bin:..." \
    cargo ndk -t arm64-v8a build
```

### Result

**Status:** Build compiled successfully but failed at link stage

**Error:**
```
ld: error: unable to find library -lhidapi
```

### Root Cause

The `rust-sdl2` binding (specifically `sdl2-sys/build.rs` line 312) unconditionally links `hidapi` for Android:

```rust
// sdl2-sys/build.rs:310-314
println!("cargo:rustc-link-lib=GLESv1_CM");
println!("cargo:rustc-link-lib=GLESv2");
println!("cargo:rustc-link-lib=hidapi");  // <-- Problem
println!("cargo:rustc-link-lib=log");
println!("cargo:rustc-link-lib=OpenSLES");
```

`hidapi` is a USB/Bluetooth HID library used for controller access on desktop platforms. On Android, SDL2 uses Android's native input system instead, so this library isn't needed or available.

## Potential Solutions

### Option 1: Patch rust-sdl2 Fork

**Effort:** Low
**Risk:** Low

Modify the doukutsu-rs fork of rust-sdl2 to not link hidapi on Android:
- Edit `sdl2-sys/build.rs`
- Remove or conditionally skip the hidapi linking for Android target

### Option 2: Add Gamepad Support to Glutin Backend

**Effort:** Medium-High
**Risk:** Medium

Implement native Android gamepad support using:
- Android NDK's `AInputEvent` and `AMotionEvent` for input events
- JNI calls to `InputManager` for device enumeration
- Handle `SOURCE_JOYSTICK` and `SOURCE_GAMEPAD` input sources

Required changes:
1. Add gamepad event handling to `backend_glutin.rs`
2. Implement `BackendGamepad` trait for Android
3. Use `ndk` crate's input event handling

### Option 3: Use Alternative Gamepad Library

**Effort:** Medium
**Risk:** Unknown

Investigate Rust gamepad libraries with Android support:
- `gilrs` - Does NOT support Android (confirmed)
- Other options need research

### Option 4: Build hidapi for Android

**Effort:** High
**Risk:** Medium

Cross-compile `libhidapi` for Android and include it in the build. This is complex and may not be necessary since SDL2 on Android doesn't actually use hidapi.

## Research: How Other Projects Solve This

### 1. android-activity Crate with GameActivity

**Project:** [rust-mobile/android-activity](https://github.com/rust-mobile/android-activity)

The most promising solution is migrating from `NativeActivity` to `GameActivity`. GameActivity is Google's recommended replacement that provides:

- Better input handling for controllers via the [Paddleboat library](https://developer.android.com/games/sdk/game-controller)
- IME (keyboard) support
- Compatibility with Android Game Development Kit (AGDK)

```toml
[dependencies]
android-activity = { version = "0.5", features = ["game-activity"] }
```

**Key insight:** GameActivity is described as "a more feature-full NativeActivity which provides more C/C++ native bindings/glue for things like input/IME and controllers."

**Status:** The android-activity crate supports both NativeActivity and GameActivity backends. Gamepad support via Paddleboat integration is on the roadmap.

Sources:
- [GameActivity support issue](https://github.com/rust-mobile/ndk/issues/266)
- [android-activity crate](https://crates.io/crates/android-activity)

### 2. Paddleboat (Google's Game Controller Library)

**Project:** [Android Game Controller Library](https://developer.android.com/games/sdk/game-controller)

Paddleboat is Google's official native library for game controller support, providing:
- Controller connection/disconnection callbacks
- Standardized dual-stick controller input
- Vibration, lights, motion axis, battery status
- Virtual and physical mouse support

**Integration path:** The `android-activity` crate was designed to facilitate integration with Paddleboat, but Rust bindings don't appear to exist yet.

### 3. Bevy Engine Approach

**Project:** [Bevy](https://bevy.org/)

Bevy uses [gilrs](https://docs.rs/gilrs) for cross-platform gamepad support. However:

- **gilrs does NOT support Android** (confirmed - only Windows, macOS, Linux, FreeBSD, WebAssembly)
- Bevy's Android support is described as "not as good as iOS, but very usable"
- For mobile, developers often use [bevy_touch_stick](https://lib.rs/crates/bevy_touch_stick) for virtual joysticks

Sources:
- [Bevy Gamepad docs](https://bevy-cheatbook.github.io/input/gamepad.html)
- [Bevy Platform support](https://bevy-cheatbook.github.io/platforms.html)

### 4. Winit Gamepad Support

**Project:** [winit](https://github.com/rust-windowing/winit)

Winit has a [tracking issue for gamepad support (#944)](https://github.com/rust-windowing/winit/issues/944), but:
- Native gamepad support was marked "wontfix" in the original issue (#119)
- Current recommendation is to use gilrs directly
- [winit-input-map](https://crates.io/crates/winit-input-map) wraps winit + gilrs

### 5. rust-mobile/ndk Gamepad Issues

**Project:** [rust-mobile/ndk](https://github.com/rust-mobile/ndk)

There's an [open issue (#408)](https://github.com/rust-mobile/ndk/issues/408) about joystick/gamepad support not working via USB. The Android NDK itself has limited gamepad APIs - developers often need JNI to access Java's `InputDevice` and `InputManager`.

### 6. SDL2 HIDAPI Workaround

For SDL2-based projects, setting `SDL_JOYSTICK_HIDAPI=0` can disable hidapi and fall back to evdev/native input. However, this doesn't solve the Rust linking issue.

Sources:
- [SDL2 Android controller issues](https://github.com/libsdl-org/SDL/issues/9562)

## Recommended Approaches (Ranked)

### Option A: Patch rust-sdl2 (NOT VIABLE)
**Effort:** High | **Risk:** High

~~Simply remove the hidapi linking from the doukutsu-rs fork of rust-sdl2~~

**Update (2024-12-21):** We attempted this approach and discovered that:

1. **Patching hidapi works** - Removing `println!("cargo:rustc-link-lib=hidapi");` from `sdl2-sys/build.rs` allows the build to complete
2. **But the app crashes** - The `drsandroid` crate uses `ndk_glue` which is incompatible with SDL2

The SDL2 backend requires a completely different Android integration:
- Replace `NativeActivity`/`GameActivity` with `SDLActivity` (SDL's Java activity)
- Remove `ndk_glue` from `drsandroid/src/lib.rs`
- Use SDL2's own entry point and main loop

This is not a simple feature flag change - it requires rewriting the Android integration layer.

### Option B: Migrate to GameActivity + Paddleboat (Best Long-term)
**Effort:** High | **Risk:** Medium

1. Update `drsandroid` to use `android-activity` crate with GameActivity
2. Create Rust bindings for Paddleboat, or use JNI to call it
3. Implement `BackendGamepad` trait using Paddleboat

This aligns with Google's recommended approach and provides the most robust solution.

### Option C: JNI to Android InputManager (Medium Complexity)
**Effort:** Medium | **Risk:** Medium

Keep using glutin/NativeActivity but add gamepad support via JNI:
1. Call `InputManager.getInputDeviceIds()` to enumerate controllers
2. Handle `MotionEvent` with `SOURCE_JOYSTICK` for analog input
3. Handle `KeyEvent` with gamepad button codes

### Option D: Virtual Touch Controls (Fallback)
**Effort:** Low | **Risk:** None

Add on-screen touch controls (like [bevy_touch_stick](https://lib.rs/crates/bevy_touch_stick)) as a fallback for devices without physical controllers.

## Summary Table

| Approach | Effort | Controller Types | Notes |
|----------|--------|------------------|-------|
| ~~Patch SDL2 hidapi~~ | ~~Low~~ | ~~USB, Bluetooth~~ | **NOT VIABLE** - Requires full SDL2 Android rewrite |
| GameActivity + Paddleboat | High | All supported | Google's recommended path |
| **JNI to InputManager** | **Medium** | **USB, Bluetooth** | **RECOMMENDED** - Works with current NativeActivity |
| Touch controls | Low | Virtual only | Fallback option |

## Conclusion

**The JNI to InputManager approach is recommended** because:
1. Works with the existing `NativeActivity`/glutin architecture
2. Doesn't require rewriting the Android integration layer
3. Can be implemented incrementally in `backend_glutin.rs`

The SDL2 approach would require significant refactoring of both the Java (`GameActivity.java` → `SDLActivity`) and Rust (`ndk_glue` removal) code.

## Files of Interest

- `drsandroid/Cargo.toml` - Android build configuration
- `src/framework/backend_glutin.rs` - Glutin backend (current Android backend)
- `src/framework/backend_sdl2.rs` - SDL2 backend (has gamepad support)
- `src/framework/gamepad.rs` - Gamepad abstraction layer
- `src/framework/backend.rs` - BackendGamepad trait
- `app/app/build.gradle` - Android gradle configuration
- `.cargo/config` - Cargo configuration for Android linking

## Build Issues Encountered

1. **Java version:** Requires Java 11+, not Java 8
2. **Rust toolchain:** Must use rustup's cargo/rustc, not homebrew's
3. **Missing cargo-ndk:** Install with `cargo install cargo-ndk`
4. **Missing cmake:** Add Android SDK's cmake to PATH
5. **hidapi linking:** Blocking issue (documented above)
