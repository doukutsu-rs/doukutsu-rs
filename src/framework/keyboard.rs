// SPDX-License-Identifier: MIT
// Copyright (c) 2016 ggez-dev
// Copyright (c) 2020 doukutsu-rs contributors (see AUTHORS.md)
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::bitfield;
use crate::framework::context::Context;

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
#[derive(Serialize, Deserialize)]
#[repr(u32)]
pub enum ScanCode {
    /// The '1' key over the letters.
    Key1,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// The Escape key, next to F1.
    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    Backspace,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    NonUsHash,
    NonUsBackslash,
    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Capslock,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Menu,
    Minus,
    Mute,
    MyComputer,
    // also called "Next"
    NavigateForward,
    // also called "Prior"
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Scrolllock,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
    Back,
}

bitfield! {
    /// Bitflags describing the state of keyboard modifiers, such as `Control` or `Shift`.
    #[derive(Debug, Copy, Clone)]
    #[allow(unused)]
    pub struct KeyMods(u8);

    /// No modifiers; equivalent to `KeyMods::default()` and
    /// [`KeyMods::empty()`](struct.KeyMods.html#method.empty).
    /// Left or right Shift key.
    pub shift, set_shift: 0;
    /// Left or right Control key.
    pub ctrl, set_ctrl: 1;
    /// Left or right Alt key.
    pub alt, set_alt: 2;
    /// Left or right Win/Cmd/equivalent key.
    pub win, set_win: 3;
}

#[derive(Clone, Debug)]
pub struct KeyboardContext {
    active_modifiers: KeyMods,
    pressed_keys_set: HashSet<ScanCode>,
    last_pressed: Option<ScanCode>,
    current_pressed: Option<ScanCode>,
}

impl KeyboardContext {
    pub(crate) fn new() -> Self {
        Self {
            active_modifiers: KeyMods(0),
            pressed_keys_set: HashSet::with_capacity(256),
            last_pressed: None,
            current_pressed: None,
        }
    }

    pub(crate) fn set_key(&mut self, key: ScanCode, pressed: bool) {
        if pressed {
            let _ = self.pressed_keys_set.insert(key);
            self.last_pressed = self.current_pressed;
            self.current_pressed = Some(key);
        } else {
            let _ = self.pressed_keys_set.remove(&key);
            self.current_pressed = None;
        }

        self.set_key_modifier(key, pressed);
    }

    /// Take a modifier key code and alter our state.
    ///
    /// Double check that this edge handling is necessary;
    /// winit sounds like it should do this for us,
    /// see https://docs.rs/winit/0.18.0/winit/struct.KeyboardInput.html#structfield.modifiers
    ///
    /// ...more specifically, we should refactor all this to consistant-ify events a bit and
    /// make winit do more of the work.
    /// But to quote Scott Pilgrim, "This is... this is... Booooooring."
    fn set_key_modifier(&mut self, key: ScanCode, pressed: bool) {
        if pressed {
            match key {
                ScanCode::LShift | ScanCode::RShift => self.active_modifiers.set_shift(true),
                ScanCode::LControl | ScanCode::RControl => self.active_modifiers.set_ctrl(true),
                ScanCode::LAlt | ScanCode::RAlt => self.active_modifiers.set_alt(true),
                ScanCode::LWin | ScanCode::RWin => self.active_modifiers.set_win(true),
                _ => (),
            }
        } else {
            match key {
                ScanCode::LShift | ScanCode::RShift => self.active_modifiers.set_shift(false),
                ScanCode::LControl | ScanCode::RControl => self.active_modifiers.set_ctrl(false),
                ScanCode::LAlt | ScanCode::RAlt => self.active_modifiers.set_alt(false),
                ScanCode::LWin | ScanCode::RWin => self.active_modifiers.set_win(false),
                _ => (),
            }
        }
    }

    pub(crate) fn set_modifiers(&mut self, keymods: KeyMods) {
        self.active_modifiers = keymods;
    }

    pub(crate) fn is_key_pressed(&self, key: ScanCode) -> bool {
        self.pressed_keys_set.contains(&key)
    }

    pub(crate) fn is_key_repeated(&self) -> bool {
        if self.last_pressed.is_some() {
            self.last_pressed == self.current_pressed
        } else {
            false
        }
    }

    pub(crate) fn pressed_keys(&self) -> &HashSet<ScanCode> {
        &self.pressed_keys_set
    }

    pub(crate) fn active_mods(&self) -> KeyMods {
        self.active_modifiers
    }
}

impl Default for KeyboardContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks if a key is currently pressed down.
pub fn is_key_pressed(ctx: &Context, key: ScanCode) -> bool {
    ctx.keyboard_context.is_key_pressed(key)
}

/// Checks if the last keystroke sent by the system is repeated,
/// like when a key is held down for a period of time.
pub fn is_key_repeated(ctx: &Context) -> bool {
    ctx.keyboard_context.is_key_repeated()
}

/// Returns a reference to the set of currently pressed keys.
pub fn pressed_keys(ctx: &Context) -> &HashSet<ScanCode> {
    ctx.keyboard_context.pressed_keys()
}

/// Checks if keyboard modifier (or several) is active.
pub fn is_mod_active(ctx: &Context, keymods: KeyMods) -> bool {
    (ctx.keyboard_context.active_mods().0 & keymods.0) != 0
}

/// Returns currently active keyboard modifiers.
pub fn active_mods(ctx: &Context) -> KeyMods {
    ctx.keyboard_context.active_mods()
}
