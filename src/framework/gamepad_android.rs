#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use bitflags::bitflags;
use jni::sys::jobject;
use jni::{JNIEnv, JavaVM};
use ndk_sys::AInputEvent;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_int};
use std::ptr::NonNull;

use super::backend::BackendGamepad;
use super::gamepad::{Axis, Button, GamepadType};

use super::error::{GameError, GameResult};

/*
 * Copyright (C) 2021 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//==================================================================================================
// Constants
//==================================================================================================

pub const PADDLEBOAT_MAJOR_VERSION: u32 = 2;
pub const PADDLEBOAT_MINOR_VERSION: u32 = 2;
pub const PADDLEBOAT_BUGFIX_VERSION: u32 = 0;
pub const PADDLEBOAT_PACKED_VERSION: u32 =
    (PADDLEBOAT_MAJOR_VERSION << 16) | (PADDLEBOAT_MINOR_VERSION << 8) | PADDLEBOAT_BUGFIX_VERSION;

/// Maximum number of simultaneously connected controllers.
pub const PADDLEBOAT_MAX_CONTROLLERS: usize = 8;
/// The maximum number of characters, including the terminating
/// character, allowed in a string table entry
pub const PADDLEBOAT_STRING_TABLE_ENTRY_MAX_SIZE: usize = 64;
/// The expected value in the `fileIdentifier` field of the
/// `Paddleboat_Controller_Mapping_File_Header` for a valid
/// mapping file.
pub const PADDLEBOAT_MAPPING_FILE_IDENTIFIER: u32 = 0xadd1eb0a;

//==================================================================================================
// Enums and Bitflags
//==================================================================================================

/// Paddleboat error code results.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Paddleboat_ErrorCode {
    /// No error. Function call was successful.
    PADDLEBOAT_NO_ERROR = 0,
    /// ::Paddleboat_init was called a second time without a call to
    /// ::Paddleboat_destroy in between.
    PADDLEBOAT_ERROR_ALREADY_INITIALIZED = -2000,
    /// Paddleboat was not successfully initialized. Either
    /// ::Paddleboat_init was not called or returned an error.
    PADDLEBOAT_ERROR_NOT_INITIALIZED = -2001,
    /// Paddleboat could not be successfully initialized. Instantiation
    /// of the GameControllerManager class failed.
    PADDLEBOAT_ERROR_INIT_GCM_FAILURE = -2002,
    /// Invalid controller index specified. Valid index range is from 0
    /// to PADDLEBOAT_MAX_CONTROLLERS - 1
    PADDLEBOAT_ERROR_INVALID_CONTROLLER_INDEX = -2003,
    /// No controller is connected at the specified controller index.
    PADDLEBOAT_ERROR_NO_CONTROLLER = -2004,
    /// No virtual or physical mouse device is connected.
    PADDLEBOAT_ERROR_NO_MOUSE = -2005,
    /// The feature is not supported by the specified controller.
    PADDLEBOAT_ERROR_FEATURE_NOT_SUPPORTED = -2006,
    /// An invalid parameter was specified. This usually means NULL or
    /// nullptr was passed in a parameter that requires a valid pointer.
    PADDLEBOAT_ERROR_INVALID_PARAMETER = -2007,
    /// Invalid controller mapping data was provided. The data in the
    /// provided buffer does not match the expected mapping data format.
    PADDLEBOAT_INVALID_MAPPING_DATA = -2008,
    /// Incompatible controller mapping data was provided. The data in
    /// the provided buffer is from an incompatible version of the mapping data
    /// format.
    PADDLEBOAT_INCOMPATIBLE_MAPPING_DATA = -2009,
    /// A file I/O error occurred when trying to read mapping data from the
    /// file descriptor passed to ::Paddleboat_addControllerRemapDataFromFd
    PADDLEBOAT_ERROR_FILE_IO = -2010,
}

bitflags! {
    /// Paddleboat controller buttons defined as bitmask values.
    /// AND against `Paddleboat_Controller_Data.buttonsDown` to check for button
    /// status.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Paddleboat_Buttons: u32 {
        const DPAD_UP = 1 << 0;
        const DPAD_LEFT = 1 << 1;
        const DPAD_DOWN = 1 << 2;
        const DPAD_RIGHT = 1 << 3;
        const A = 1 << 4;
        const B = 1 << 5;
        const X = 1 << 6;
        const Y = 1 << 7;
        const L1 = 1 << 8;
        const L2 = 1 << 9;
        const L3 = 1 << 10;
        const R1 = 1 << 11;
        const R2 = 1 << 12;
        const R3 = 1 << 13;
        const SELECT = 1 << 14;
        const START = 1 << 15;
        const SYSTEM = 1 << 16;
        const TOUCHPAD = 1 << 17;
        const AUX1 = 1 << 18;
        const AUX2 = 1 << 19;
        const AUX3 = 1 << 20;
        const AUX4 = 1 << 21;
    }
}

pub const PADDLEBOAT_BUTTON_DPAD_UP: u32 = 1 << 0;
pub const PADDLEBOAT_BUTTON_DPAD_LEFT: u32 = 1 << 1;
pub const PADDLEBOAT_BUTTON_DPAD_DOWN: u32 = 1 << 2;
pub const PADDLEBOAT_BUTTON_DPAD_RIGHT: u32 = 1 << 3;
pub const PADDLEBOAT_BUTTON_A: u32 = 1 << 4;
pub const PADDLEBOAT_BUTTON_B: u32 = 1 << 5;
pub const PADDLEBOAT_BUTTON_X: u32 = 1 << 6;
pub const PADDLEBOAT_BUTTON_Y: u32 = 1 << 7;
pub const PADDLEBOAT_BUTTON_L1: u32 = 1 << 8;
pub const PADDLEBOAT_BUTTON_L2: u32 = 1 << 9;
pub const PADDLEBOAT_BUTTON_L3: u32 = 1 << 10;
pub const PADDLEBOAT_BUTTON_R1: u32 = 1 << 11;
pub const PADDLEBOAT_BUTTON_R2: u32 = 1 << 12;
pub const PADDLEBOAT_BUTTON_R3: u32 = 1 << 13;
pub const PADDLEBOAT_BUTTON_SELECT: u32 = 1 << 14;
pub const PADDLEBOAT_BUTTON_START: u32 = 1 << 15;
pub const PADDLEBOAT_BUTTON_SYSTEM: u32 = 1 << 16;
pub const PADDLEBOAT_BUTTON_TOUCHPAD: u32 = 1 << 17;
pub const PADDLEBOAT_BUTTON_AUX1: u32 = 1 << 18;
pub const PADDLEBOAT_BUTTON_AUX2: u32 = 1 << 19;
pub const PADDLEBOAT_BUTTON_AUX3: u32 = 1 << 20;
pub const PADDLEBOAT_BUTTON_AUX4: u32 = 1 << 21;
pub const PADDLEBOAT_BUTTON_COUNT: u32 = 22;

bitflags! {
    /// Paddleboat controller device feature flags as bitmask values
    /// AND against `Paddleboat_Controller_Info.controllerFlags` to determine feature
    /// availability.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Paddleboat_Controller_Flags: u32 {
        const GENERIC_PROFILE = 0x0000000010;
        const ACCELEROMETER = 0x00400000;
        const GYROSCOPE = 0x00800000;
        const LIGHT_PLAYER = 0x01000000;
        const LIGHT_RGB = 0x02000000;
        const BATTERY = 0x04000000;
        const VIBRATION = 0x08000000;
        const VIBRATION_DUAL_MOTOR = 0x10000000;
        const TOUCHPAD = 0x20000000;
        const VIRTUAL_MOUSE = 0x40000000;
    }
}

bitflags! {
    /// Bitmask values to use with ::Paddleboat_getIntegratedMotionSensorFlags
    /// and ::Paddleboat_setMotionDataCallbackWithIntegratedFlags
    /// Bitmask values represent integrated sensor types on the main device instead
    /// of a controller device.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Paddleboat_Integrated_Motion_Sensor_Flags: u32 {
        const NONE = 0;
        const ACCELEROMETER = 0x00000001;
        const GYROSCOPE = 0x00000002;
    }
}

/// Value passed in the `controllerIndex` parameter of the
/// `Paddleboat_MotionDataCallback` if integrated sensor data
/// reporting is active and the motion data event came from
/// an integrated sensor, rather than a controller sensor.
pub const PADDLEBOAT_INTEGRATED_SENSOR_INDEX: u32 = 0x40000000;

bitflags! {
    /// Paddleboat mouse buttons as bitmask values
    /// AND against `Paddleboat_Mouse_Data.buttonsDown` to determine button status.
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Paddleboat_Mouse_Buttons: u32 {
        const LEFT = 1 << 0;
        const RIGHT = 1 << 1;
        const MIDDLE = 1 << 2;
        const BACK = 1 << 3;
        const FORWARD = 1 << 4;
        const BUTTON_6 = 1 << 5;
        const BUTTON_7 = 1 << 6;
        const BUTTON_8 = 1 << 7;
    }
}

/// Paddleboat axis mapping table axis order.
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Paddleboat_Mapping_Axis {
    PADDLEBOAT_MAPPING_AXIS_LEFTSTICK_X = 0,
    PADDLEBOAT_MAPPING_AXIS_LEFTSTICK_Y,
    PADDLEBOAT_MAPPING_AXIS_RIGHTSTICK_X,
    PADDLEBOAT_MAPPING_AXIS_RIGHTSTICK_Y,
    PADDLEBOAT_MAPPING_AXIS_L1,
    PADDLEBOAT_MAPPING_AXIS_L2,
    PADDLEBOAT_MAPPING_AXIS_R1,
    PADDLEBOAT_MAPPING_AXIS_R2,
    PADDLEBOAT_MAPPING_AXIS_HATX,
    PADDLEBOAT_MAPPING_AXIS_HATY,
    PADDLEBOAT_MAPPING_AXIS_COUNT = 10,
}
pub const PADDLEBOAT_MAPPING_AXIS_COUNT: usize = Paddleboat_Mapping_Axis::PADDLEBOAT_MAPPING_AXIS_COUNT as usize;

pub const PADDLEBOAT_AXIS_BUTTON_IGNORED: u32 = 0xFE;
pub const PADDLEBOAT_AXIS_IGNORED: u32 = 0xFFFE;
pub const PADDLEBOAT_BUTTON_IGNORED: u32 = 0xFFFE;

/// Battery status of a controller
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Paddleboat_BatteryStatus {
    PADDLEBOAT_CONTROLLER_BATTERY_UNKNOWN = 0,
    PADDLEBOAT_CONTROLLER_BATTERY_CHARGING = 1,
    PADDLEBOAT_CONTROLLER_BATTERY_DISCHARGING = 2,
    PADDLEBOAT_CONTROLLER_BATTERY_NOT_CHARGING = 3,
    PADDLEBOAT_CONTROLLER_BATTERY_FULL = 4,
}

/// Current status of a controller (at a specified controller index)
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Paddleboat_ControllerStatus {
    PADDLEBOAT_CONTROLLER_INACTIVE = 0,
    PADDLEBOAT_CONTROLLER_ACTIVE = 1,
    PADDLEBOAT_CONTROLLER_JUST_CONNECTED = 2,
    PADDLEBOAT_CONTROLLER_JUST_DISCONNECTED = 3,
}

/// The button layout and iconography of the controller buttons
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Paddleboat_ControllerButtonLayout {
    PADDLEBOAT_CONTROLLER_LAYOUT_STANDARD = 0,
    PADDLEBOAT_CONTROLLER_LAYOUT_SHAPES = 1,
    PADDLEBOAT_CONTROLLER_LAYOUT_REVERSE = 2,
    PADDLEBOAT_CONTROLLER_LAYOUT_ARCADE_STICK = 3,
}
const PADDLEBOAT_CONTROLLER_LAYOUT_MASK: u32 = 3;

/// The type of light being specified by a call to ::Paddleboat_setControllerLight
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Paddleboat_LightType {
    PADDLEBOAT_LIGHT_PLAYER_NUMBER = 0,
    PADDLEBOAT_LIGHT_RGB = 1,
}

/// The type of motion data being reported in a Paddleboat_Motion_Data structure
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Paddleboat_Motion_Type {
    PADDLEBOAT_MOTION_ACCELEROMETER = 0,
    PADDLEBOAT_MOTION_GYROSCOPE = 1,
}

/// The status of the mouse device
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Paddleboat_MouseStatus {
    PADDLEBOAT_MOUSE_NONE = 0,
    PADDLEBOAT_MOUSE_CONTROLLER_EMULATED = 1,
    PADDLEBOAT_MOUSE_PHYSICAL,
}

/// The addition mode to use when passing new controller mapping data
/// to ::Paddleboat_addControllerRemapData
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Paddleboat_Remap_Addition_Mode {
    PADDLEBOAT_REMAP_ADD_MODE_DEFAULT = 0,
    PADDLEBOAT_REMAP_ADD_MODE_REPLACE_ALL,
}

//==================================================================================================
// Structs
//==================================================================================================

/// A structure that describes the current battery state of a controller.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Paddleboat_Controller_Battery {
    pub batteryStatus: Paddleboat_BatteryStatus,
    pub batteryLevel: f32,
}

/// A structure that contains virtual pointer position data.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Paddleboat_Controller_Pointer {
    pub pointerX: f32,
    pub pointerY: f32,
}

/// A structure that contains X and Y axis data for an analog thumbstick.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Paddleboat_Controller_Thumbstick {
    pub stickX: f32,
    pub stickY: f32,
}

/// A structure that contains axis precision data for a thumbstick in the
/// X and Y axis.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Paddleboat_Controller_Thumbstick_Precision {
    pub stickFlatX: f32,
    pub stickFlatY: f32,
    pub stickFuzzX: f32,
    pub stickFuzzY: f32,
}

/// A structure that contains the current data for a controller's inputs and sensors.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Paddleboat_Controller_Data {
    pub timestamp: u64,
    pub buttonsDown: u32, // Corresponds to Paddleboat_Buttons flags
    pub leftStick: Paddleboat_Controller_Thumbstick,
    pub rightStick: Paddleboat_Controller_Thumbstick,
    pub triggerL1: f32,
    pub triggerL2: f32,
    pub triggerR1: f32,
    pub triggerR2: f32,
    pub virtualPointer: Paddleboat_Controller_Pointer,
    pub battery: Paddleboat_Controller_Battery,
}

/// A structure that contains information about a particular controller device.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Paddleboat_Controller_Info {
    pub controllerFlags: u32, // Corresponds to Paddleboat_Controller_Flags
    pub controllerNumber: i32,
    pub vendorId: i32,
    pub productId: i32,
    pub deviceId: i32,
    pub leftStickPrecision: Paddleboat_Controller_Thumbstick_Precision,
    pub rightStickPrecision: Paddleboat_Controller_Thumbstick_Precision,
}

/// A structure that contains motion data reported by a controller.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Paddleboat_Motion_Data {
    pub timestamp: u64,
    pub motionType: Paddleboat_Motion_Type,
    pub motionX: f32,
    pub motionY: f32,
    pub motionZ: f32,
}

/// A structure that contains input data for the mouse device.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Paddleboat_Mouse_Data {
    pub timestamp: u64,
    pub buttonsDown: u32, // Corresponds to Paddleboat_Mouse_Buttons flags
    pub mouseScrollDeltaH: i32,
    pub mouseScrollDeltaV: i32,
    pub mouseX: f32,
    pub mouseY: f32,
}

/// A structure that describes the parameters of a vibration effect.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Paddleboat_Vibration_Data {
    pub durationLeft: i32,
    pub durationRight: i32,
    pub intensityLeft: f32,
    pub intensityRight: f32,
}

/// A structure that describes the button and axis mappings for a specified controller device.
#[deprecated(
    note = "Use the `Paddleboat_Controller_Mapping_File_Header` in combination with the `Paddleboat_addControllerRemapDataFromFileBuffer` function instead."
)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Paddleboat_Controller_Mapping_Data {
    pub minimumEffectiveApiLevel: i16,
    pub maximumEffectiveApiLevel: i16,
    pub vendorId: i32,
    pub productId: i32,
    pub flags: i32,
    pub axisMapping: [u16; PADDLEBOAT_MAPPING_AXIS_COUNT],
    pub axisPositiveButtonMapping: [u8; PADDLEBOAT_MAPPING_AXIS_COUNT],
    pub axisNegativeButtonMapping: [u8; PADDLEBOAT_MAPPING_AXIS_COUNT],
    pub buttonMapping: [u16; PADDLEBOAT_BUTTON_COUNT as usize],
}

//==================================================================================================
// Function Pointer Typedefs
//==================================================================================================

pub type Paddleboat_ControllerStatusCallback = Option<
    unsafe extern "C" fn(controllerIndex: i32, controllerStatus: Paddleboat_ControllerStatus, userData: *mut c_void),
>;

pub type Paddleboat_MouseStatusCallback =
    Option<unsafe extern "C" fn(mouseStatus: Paddleboat_MouseStatus, userData: *mut c_void)>;

pub type Paddleboat_MotionDataCallback = Option<
    unsafe extern "C" fn(controllerIndex: i32, motionData: *const Paddleboat_Motion_Data, userData: *mut c_void),
>;

pub type Paddleboat_PhysicalKeyboardStatusCallback =
    Option<unsafe extern "C" fn(physicalKeyboardStatus: bool, userData: *mut c_void)>;

//==================================================================================================
// Function Prototypes
//==================================================================================================

unsafe extern "C" {
    pub fn Paddleboat_init(env: JNIEnv, jcontext: jobject) -> Paddleboat_ErrorCode;
    pub fn Paddleboat_isInitialized() -> bool;
    pub fn Paddleboat_destroy(env: JNIEnv);
    pub fn Paddleboat_onStop(env: JNIEnv);
    pub fn Paddleboat_onStart(env: JNIEnv);
    pub fn Paddleboat_processInputEvent(event: *const AInputEvent) -> i32;
    pub fn Paddleboat_processGameActivityKeyInputEvent(event: *const c_void, eventSize: usize) -> i32;
    pub fn Paddleboat_processGameActivityMotionInputEvent(event: *const c_void, eventSize: usize) -> i32;
    pub fn Paddleboat_getActiveAxisMask() -> u64;
    pub fn Paddleboat_getBackButtonConsumed() -> bool;
    pub fn Paddleboat_getIntegratedMotionSensorFlags() -> Paddleboat_Integrated_Motion_Sensor_Flags;
    pub fn Paddleboat_setBackButtonConsumed(consumeBackButton: bool);
    pub fn Paddleboat_setControllerStatusCallback(
        statusCallback: Paddleboat_ControllerStatusCallback,
        userData: *mut c_void,
    );
    pub fn Paddleboat_setMotionDataCallback(motionDataCallback: Paddleboat_MotionDataCallback, userData: *mut c_void);
    pub fn Paddleboat_setMotionDataCallbackWithIntegratedFlags(
        motionDataCallback: Paddleboat_MotionDataCallback,
        integratedSensorFlags: Paddleboat_Integrated_Motion_Sensor_Flags,
        userData: *mut c_void,
    ) -> Paddleboat_ErrorCode;
    pub fn Paddleboat_setMouseStatusCallback(statusCallback: Paddleboat_MouseStatusCallback, userData: *mut c_void);
    pub fn Paddleboat_setPhysicalKeyboardStatusCallback(
        statusCallback: Paddleboat_PhysicalKeyboardStatusCallback,
        userData: *mut c_void,
    );
    pub fn Paddleboat_getControllerData(
        controllerIndex: i32,
        controllerData: *mut Paddleboat_Controller_Data,
    ) -> Paddleboat_ErrorCode;
    pub fn Paddleboat_getControllerInfo(
        controllerIndex: i32,
        controllerInfo: *mut Paddleboat_Controller_Info,
    ) -> Paddleboat_ErrorCode;
    pub fn Paddleboat_getControllerName(
        controllerIndex: i32,
        bufferSize: usize,
        controllerName: *mut c_char,
    ) -> Paddleboat_ErrorCode;
    pub fn Paddleboat_getControllerStatus(controllerIndex: i32) -> Paddleboat_ControllerStatus;
    pub fn Paddleboat_setControllerLight(
        controllerIndex: i32,
        lightType: Paddleboat_LightType,
        lightData: u32,
        env: JNIEnv,
    ) -> Paddleboat_ErrorCode;
    pub fn Paddleboat_setControllerVibrationData(
        controllerIndex: i32,
        vibrationData: *const Paddleboat_Vibration_Data,
        env: JNIEnv,
    ) -> Paddleboat_ErrorCode;
    pub fn Paddleboat_getMouseData(mouseData: *mut Paddleboat_Mouse_Data) -> Paddleboat_ErrorCode;
    pub fn Paddleboat_getMouseStatus() -> Paddleboat_MouseStatus;
    pub fn Paddleboat_getPhysicalKeyboardStatus() -> bool;
    #[deprecated(
        note = "Use ::Paddleboat_addControllerRemapDataFromFd or ::Paddleboat_addControllerRemapDataFromFileBuffer instead."
    )]
    pub fn Paddleboat_addControllerRemapData(
        addMode: Paddleboat_Remap_Addition_Mode,
        remapTableEntryCount: i32,
        mappingData: *const Paddleboat_Controller_Mapping_Data,
    );
    pub fn Paddleboat_addControllerRemapDataFromFd(
        addMode: Paddleboat_Remap_Addition_Mode,
        mappingFileDescriptor: c_int,
    ) -> Paddleboat_ErrorCode;
    pub fn Paddleboat_addControllerRemapDataFromFileBuffer(
        addMode: Paddleboat_Remap_Addition_Mode,
        mappingFileBuffer: *const c_void,
        mappingFileBufferSize: usize,
    ) -> Paddleboat_ErrorCode;
    #[deprecated(note = "The number of elements returned will always be zero.")]
    pub fn Paddleboat_getControllerRemapTableData(
        destRemapTableEntryCount: i32,
        mappingData: *mut Paddleboat_Controller_Mapping_Data,
    ) -> i32;
    pub fn Paddleboat_update(env: JNIEnv);
    pub fn Paddleboat_getLastKeycode() -> i32;
}
pub fn paddleboat_error_to_game_error(err: Paddleboat_ErrorCode) -> GameResult {
    match err {
        Paddleboat_ErrorCode::PADDLEBOAT_NO_ERROR => Ok(()),
        Paddleboat_ErrorCode::PADDLEBOAT_ERROR_ALREADY_INITIALIZED => {
            Err(GameError::GamepadError("Paddleboat: already initialized (Paddleboat_init called twice)".to_string()))
        }
        Paddleboat_ErrorCode::PADDLEBOAT_ERROR_NOT_INITIALIZED => {
            Err(GameError::GamepadError("Paddleboat: not initialized".to_string()))
        }
        Paddleboat_ErrorCode::PADDLEBOAT_ERROR_INIT_GCM_FAILURE => {
            Err(GameError::GamepadError("Paddleboat: failed to initialize GameControllerManager".to_string()))
        }
        Paddleboat_ErrorCode::PADDLEBOAT_ERROR_INVALID_CONTROLLER_INDEX => {
            Err(GameError::GamepadError("Paddleboat: invalid controller index".to_string()))
        }
        Paddleboat_ErrorCode::PADDLEBOAT_ERROR_NO_CONTROLLER => {
            Err(GameError::GamepadError("Paddleboat: no controller connected".to_string()))
        }
        Paddleboat_ErrorCode::PADDLEBOAT_ERROR_NO_MOUSE => {
            Err(GameError::GamepadError("Paddleboat: no mouse connected".to_string()))
        }
        Paddleboat_ErrorCode::PADDLEBOAT_ERROR_FEATURE_NOT_SUPPORTED => {
            Err(GameError::GamepadError("Paddleboat: feature not supported by controller".to_string()))
        }
        Paddleboat_ErrorCode::PADDLEBOAT_ERROR_INVALID_PARAMETER => {
            Err(GameError::GamepadError("Paddleboat: invalid parameter".to_string()))
        }
        Paddleboat_ErrorCode::PADDLEBOAT_INVALID_MAPPING_DATA => {
            Err(GameError::GamepadError("Paddleboat: invalid mapping data".to_string()))
        }
        Paddleboat_ErrorCode::PADDLEBOAT_INCOMPATIBLE_MAPPING_DATA => {
            Err(GameError::GamepadError("Paddleboat: incompatible mapping data version".to_string()))
        }
        Paddleboat_ErrorCode::PADDLEBOAT_ERROR_FILE_IO => {
            Err(GameError::GamepadError("Paddleboat: file I/O error while reading mapping data".to_string()))
        }
        // Fallback for any future/unknown variants
        _ => Err(GameError::GamepadError(format!("Paddleboat: unknown error ({:?})", err))),
    }
}

/// Convert Paddleboat button flags to our Button enum
fn conv_paddleboat_button(button_flag: u32) -> Option<Button> {
    match button_flag {
        PADDLEBOAT_BUTTON_A => Some(Button::South),
        PADDLEBOAT_BUTTON_B => Some(Button::East),
        PADDLEBOAT_BUTTON_X => Some(Button::West),
        PADDLEBOAT_BUTTON_Y => Some(Button::North),
        PADDLEBOAT_BUTTON_SELECT => Some(Button::Back),
        PADDLEBOAT_BUTTON_START => Some(Button::Start),
        PADDLEBOAT_BUTTON_L3 => Some(Button::LeftStick),
        PADDLEBOAT_BUTTON_R3 => Some(Button::RightStick),
        PADDLEBOAT_BUTTON_L1 => Some(Button::LeftShoulder),
        PADDLEBOAT_BUTTON_R1 => Some(Button::RightShoulder),
        PADDLEBOAT_BUTTON_DPAD_UP => Some(Button::DPadUp),
        PADDLEBOAT_BUTTON_DPAD_DOWN => Some(Button::DPadDown),
        PADDLEBOAT_BUTTON_DPAD_LEFT => Some(Button::DPadLeft),
        PADDLEBOAT_BUTTON_DPAD_RIGHT => Some(Button::DPadRight),
        PADDLEBOAT_BUTTON_SYSTEM => Some(Button::Guide),
        _ => None,
    }
}

/// Convert Paddleboat controller type to our GamepadType enum
fn get_gamepad_type_from_controller_info(info: &Paddleboat_Controller_Info) -> GamepadType {
    // Try to determine type based on vendor/product ID
    match (info.vendorId, info.productId) {
        // Sony Controllers
        (0x054C, 0x05C4) => GamepadType::PS4,
        (0x054C, 0x0CE6) => GamepadType::PS5,
        (0x054C, 0x0268) => GamepadType::PS3,

        // Microsoft Controllers
        (0x045E, 0x028E) => GamepadType::Xbox360,
        (0x045E, 0x02D1) => GamepadType::XboxOne,
        (0x045E, 0x02DD) => GamepadType::XboxOne,
        (0x045E, 0x02E3) => GamepadType::XboxOne,
        (0x045E, 0x02EA) => GamepadType::XboxOne,
        (0x045E, 0x02FD) => GamepadType::XboxOne,

        // Nintendo Controllers (Switch 1)
        (0x057E, 0x2006) => GamepadType::NintendoSwitchJoyConLeft,
        (0x057E, 0x2007) => GamepadType::NintendoSwitchJoyConRight,
        (0x057E, 0x2008) => GamepadType::NintendoSwitchJoyConPair, // Used by joycond
        (0x057E, 0x200E) => GamepadType::NintendoSwitchJoyConGrip,
        (0x057E, 0x2009) => GamepadType::NintendoSwitchPro,

        // Nintendo Controllers (Switch 2)
        (0x057E, 0x2066) => GamepadType::NintendoSwitch2JoyConRight,
        (0x057E, 0x2067) => GamepadType::NintendoSwitch2JoyConLeft,
        (0x057E, 0x2068) => GamepadType::NintendoSwitch2JoyConPair,
        (0x057E, 0x2069) => GamepadType::NintendoSwitch2Pro,
        (0x057E, 0x2073) => GamepadType::NintendoSwitch2GameCubeController,

        // Google Stadia
        (0x18D1, 0x9400) => GamepadType::GoogleStadia,

        _ => {
            // If we couldn't match VID/PID, try to guess from the controller's
            // button layout bits stored in controllerFlags. The lower bits
            // (masked by PADDLEBOAT_CONTROLLER_LAYOUT_MASK) correspond to
            // Paddleboat_ControllerButtonLayout.
            let layout_bits = info.controllerFlags & (PADDLEBOAT_CONTROLLER_LAYOUT_MASK as u32);
            match layout_bits {
                x if x == Paddleboat_ControllerButtonLayout::PADDLEBOAT_CONTROLLER_LAYOUT_STANDARD as u32 => {
                    // Standard (Y/X B/A layout) -> Xbox-style controller
                    GamepadType::XboxOne
                }
                x if x == Paddleboat_ControllerButtonLayout::PADDLEBOAT_CONTROLLER_LAYOUT_SHAPES as u32 => {
                    // Shapes (PlayStation icons) -> PS4-style controller
                    GamepadType::PS4
                }
                x if x == Paddleboat_ControllerButtonLayout::PADDLEBOAT_CONTROLLER_LAYOUT_REVERSE as u32 => {
                    // Reverse (X/Y A/B swapped) -> Nintendo Switch Pro style
                    GamepadType::NintendoSwitchPro
                }
                _ => GamepadType::Unknown,
            }
        }
    }
}

struct AndroidGamepadManagerInner {
    vm: JavaVM,
    gamepads: HashMap<i32, AndroidGamepadData>,
    connected_gamepads: HashMap<i32, Box<AndroidGamepad>>,
}

struct AndroidGamepadData {
    controller_index: i32,
    info: Paddleboat_Controller_Info,
    data: Paddleboat_Controller_Data,
    gamepad_type: GamepadType,
}

pub struct AndroidGamepadManager {
    inner: AndroidGamepadManagerInner,
}

impl AndroidGamepadManager {
    pub fn new() -> GameResult<Self> {
        unsafe {
            let vm_ptr = ndk_glue::native_activity().vm();
            let vm = JavaVM::from_raw(vm_ptr)?;
            let jni_env = vm.get_env()?;
            let activity = ndk_glue::native_activity().activity();

            paddleboat_error_to_game_error(Paddleboat_init(jni_env, activity))?;

            Ok(Self {
                inner: AndroidGamepadManagerInner { vm, gamepads: HashMap::new(), connected_gamepads: HashMap::new() },
            })
        }
    }

    pub fn process_event(&mut self, event: NonNull<ndk_sys::AInputEvent>) -> bool {
        unsafe { Paddleboat_processInputEvent(event.as_ptr()) != 0 }
    }

    pub fn update(&mut self) {
        unsafe {
            if let Ok(jni_env) = self.inner.vm.get_env() {
                Paddleboat_update(jni_env);
            }
        }

        // Check for controller status changes
        for controller_index in 0..PADDLEBOAT_MAX_CONTROLLERS {
            let controller_index = controller_index as i32;

            unsafe {
                let controller_status = Paddleboat_getControllerStatus(controller_index);
                match controller_status {
                    Paddleboat_ControllerStatus::PADDLEBOAT_CONTROLLER_JUST_CONNECTED => {
                        self.handle_controller_connected(controller_index);
                    }
                    Paddleboat_ControllerStatus::PADDLEBOAT_CONTROLLER_JUST_DISCONNECTED => {
                        self.handle_controller_disconnected(controller_index);
                    }
                    Paddleboat_ControllerStatus::PADDLEBOAT_CONTROLLER_ACTIVE => {
                        // Update controller data for active controllers
                        if let Some(gamepad_data) = self.inner.gamepads.get_mut(&controller_index) {
                            use std::mem::MaybeUninit;

                            // Zeroed uninitialized controller data, Paddleboat will fill it in
                            let mut controller_data_uninit: MaybeUninit<Paddleboat_Controller_Data> =
                                MaybeUninit::zeroed();
                            let controller_data_ptr = controller_data_uninit.as_mut_ptr();

                            if Paddleboat_getControllerData(controller_index, controller_data_ptr)
                                == Paddleboat_ErrorCode::PADDLEBOAT_NO_ERROR
                            {
                                // Safe because Paddleboat_getControllerData initialized the struct
                                let controller_data = controller_data_uninit.assume_init();
                                gamepad_data.data = controller_data;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn handle_controller_connected(&mut self, controller_index: i32) {
        unsafe {
            // Zero-init the info struct efficiently
            let mut info_uninit = MaybeUninit::<Paddleboat_Controller_Info>::zeroed();
            let res = Paddleboat_getControllerInfo(controller_index, info_uninit.as_mut_ptr());
            if res == Paddleboat_ErrorCode::PADDLEBOAT_NO_ERROR {
                let controller_info = info_uninit.assume_init();
                let gamepad_type = get_gamepad_type_from_controller_info(&controller_info);

                // Prepare an empty controller data struct using MaybeUninit and then fill it
                let mut data_uninit = MaybeUninit::<Paddleboat_Controller_Data>::zeroed();
                // It's okay if the following call fails; we'll keep zeroed data in that case
                let _ = Paddleboat_getControllerData(controller_index, data_uninit.as_mut_ptr());
                let controller_data = data_uninit.assume_init();

                let gamepad_data =
                    AndroidGamepadData { controller_index, info: controller_info, data: controller_data, gamepad_type };

                self.inner.gamepads.insert(controller_index, gamepad_data);
            }
        }
    }

    fn handle_controller_disconnected(&mut self, controller_index: i32) {
        self.inner.gamepads.remove(&controller_index);
        self.inner.connected_gamepads.remove(&controller_index);
    }
    pub fn get_gamepad(&self, instance_id: u32) -> Option<Box<dyn BackendGamepad>> {
        let vm = unsafe { JavaVM::from_raw(self.inner.vm.get_java_vm_pointer()).unwrap() };

        if let Some(gamepad_data) = self.inner.gamepads.get(&(instance_id as i32)) {
            Some(Box::new(AndroidGamepad {
                controller_index: gamepad_data.controller_index,
                gamepad_type: gamepad_data.gamepad_type,
                data: gamepad_data.data.clone(),
                vm,
                instance_id: gamepad_data.info.deviceId as u32,
            }))
        } else {
            None
        }
    }

    pub fn get_gamepad_ids(&self) -> Vec<u32> {
        self.inner.gamepads.keys().map(|&k| k as u32).collect()
    }

    pub fn get_button_pressed(&self, instance_id: u32, button: Button) -> bool {
        if let Some(gamepad_data) = self.inner.gamepads.get(&(instance_id as i32)) {
            let button_flag = match button {
                Button::South => PADDLEBOAT_BUTTON_A,
                Button::East => PADDLEBOAT_BUTTON_B,
                Button::West => PADDLEBOAT_BUTTON_X,
                Button::North => PADDLEBOAT_BUTTON_Y,
                Button::Back => PADDLEBOAT_BUTTON_SELECT,
                Button::Start => PADDLEBOAT_BUTTON_START,
                Button::LeftStick => PADDLEBOAT_BUTTON_L3,
                Button::RightStick => PADDLEBOAT_BUTTON_R3,
                Button::LeftShoulder => PADDLEBOAT_BUTTON_L1,
                Button::RightShoulder => PADDLEBOAT_BUTTON_R1,
                Button::DPadUp => PADDLEBOAT_BUTTON_DPAD_UP,
                Button::DPadDown => PADDLEBOAT_BUTTON_DPAD_DOWN,
                Button::DPadLeft => PADDLEBOAT_BUTTON_DPAD_LEFT,
                Button::DPadRight => PADDLEBOAT_BUTTON_DPAD_RIGHT,
                Button::Guide => PADDLEBOAT_BUTTON_SYSTEM,
                _ => return false,
            };
            (gamepad_data.data.buttonsDown & button_flag) != 0
        } else {
            false
        }
    }

    pub fn get_axis_value(&self, instance_id: u32, axis: Axis) -> f32 {
        if let Some(gamepad_data) = self.inner.gamepads.get(&(instance_id as i32)) {
            match axis {
                Axis::LeftX => gamepad_data.data.leftStick.stickX,
                Axis::LeftY => -gamepad_data.data.leftStick.stickY, // Invert Y axis to match SDL2 convention
                Axis::RightX => gamepad_data.data.rightStick.stickX,
                Axis::RightY => -gamepad_data.data.rightStick.stickY, // Invert Y axis to match SDL2 convention
                Axis::TriggerLeft => gamepad_data.data.triggerL2,
                Axis::TriggerRight => gamepad_data.data.triggerR2,
                _ => 0.0,
            }
        } else {
            0.0
        }
    }
}

impl Drop for AndroidGamepadManager {
    fn drop(&mut self) {
        if let Ok(jni_env) = self.inner.vm.get_env() {
            unsafe {
                Paddleboat_destroy(jni_env);
            }
        }
    }
}

struct AndroidGamepad {
    controller_index: i32,
    gamepad_type: GamepadType,
    data: Paddleboat_Controller_Data,
    vm: JavaVM,
    instance_id: u32,
}

impl BackendGamepad for AndroidGamepad {
    fn set_rumble(&mut self, low_freq: u16, high_freq: u16, duration_ms: u32) -> GameResult {
        if let Ok(jni_env) = self.vm.get_env() {
            let vibration_data = Paddleboat_Vibration_Data {
                durationLeft: duration_ms as i32,
                durationRight: duration_ms as i32,
                intensityLeft: (low_freq as f32) / 65535.0,
                intensityRight: (high_freq as f32) / 65535.0,
            };

            unsafe {
                paddleboat_error_to_game_error(Paddleboat_setControllerVibrationData(
                    self.controller_index,
                    &vibration_data,
                    jni_env,
                ))?;
            }
        }
        Ok(())
    }

    fn instance_id(&self) -> u32 {
        self.instance_id
    }
}
