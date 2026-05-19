//! Framework-level input hub.
//!
//! Backends translate native input events into calls to the free functions in
//! this module. `InputContext` owns mouse state, a buffered queue of imgui
//! events (drained into `imgui::Io` each frame by `UI::draw`), and a
//! backend-observable text-input-active flag driven by `io.want_text_input`.
//!
//! TODO(horizon): when `backend_horizon` is revived, route its native keyboard,
//! touch, and text-input events through the functions here, and wire the libnx
//! swkbd applet to `consume_text_input_change`.

use crate::framework::keyboard::{KeyboardContext, ScanCode};

/// Mouse button enum that maps cleanly to imgui's `MouseButton`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Extra1,
    Extra2,
}

impl MouseButton {
    fn to_imgui(self) -> imgui::MouseButton {
        match self {
            MouseButton::Left => imgui::MouseButton::Left,
            MouseButton::Right => imgui::MouseButton::Right,
            MouseButton::Middle => imgui::MouseButton::Middle,
            MouseButton::Extra1 => imgui::MouseButton::Extra1,
            MouseButton::Extra2 => imgui::MouseButton::Extra2,
        }
    }
}

/// Internal buffered imgui events. Drained into `imgui::Io` by `UI::draw`.
enum ImguiInputEvent {
    Key(imgui::Key, bool),
    Char(char),
    MousePos(f32, f32),
    MouseButton(imgui::MouseButton, bool),
    MouseWheel(f32, f32),
}

pub struct InputContext {
    /// Last-known mouse position in logical (canvas) coordinates.
    pub mouse_position: (f32, f32),
    /// Mirrored from `imgui::Io::want_capture_keyboard` at the end of each
    /// frame — game-level key callbacks should early-return when this is true.
    pub want_capture_keyboard: bool,
    /// Mirrored from `imgui::Io::want_capture_mouse`.
    pub want_capture_mouse: bool,

    pending_imgui_events: Vec<ImguiInputEvent>,
    text_input_active: bool,
    text_input_applied: bool,
}

impl InputContext {
    pub(crate) fn new() -> Self {
        Self {
            mouse_position: (0.0, 0.0),
            want_capture_keyboard: false,
            want_capture_mouse: false,
            pending_imgui_events: Vec::new(),
            text_input_active: false,
            text_input_applied: false,
        }
    }

    /// Route a keyboard event. Always updates `KeyboardContext` (so key-up is
    /// never dropped even if imgui claims capture mid-press) and queues a
    /// corresponding imgui key event. Callers wanting to gate game-level
    /// behaviour should check `want_capture_keyboard` separately.
    pub fn on_key(&mut self, kb: &mut KeyboardContext, scan: ScanCode, pressed: bool) {
        kb.set_key(scan, pressed);
        if let Some(key) = scancode_to_imgui_key(scan) {
            self.pending_imgui_events.push(ImguiInputEvent::Key(key, pressed));
        }
    }

    /// Route UTF-8 text input (e.g. SDL2 `TextInput` events) to imgui.
    pub fn on_text(&mut self, s: &str) {
        for ch in s.chars() {
            self.pending_imgui_events.push(ImguiInputEvent::Char(ch));
        }
    }

    /// Update mouse cursor position in logical (canvas) coordinates.
    pub fn on_mouse_move(&mut self, x: f32, y: f32) {
        self.mouse_position = (x, y);
        self.pending_imgui_events.push(ImguiInputEvent::MousePos(x, y));
    }

    pub fn on_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        self.pending_imgui_events.push(ImguiInputEvent::MouseButton(button.to_imgui(), pressed));
    }

    pub fn on_mouse_wheel(&mut self, dx: f32, dy: f32) {
        self.pending_imgui_events.push(ImguiInputEvent::MouseWheel(dx, dy));
    }

    /// Feed a touch-down as a synthetic overlay primary-mouse press at
    /// `(overlay_x, overlay_y)` (already in overlay coord space).
    /// `is_primary` should be true only for the first active finger so
    /// subsequent touches don't retrigger the imgui click.
    pub fn on_overlay_touch_down(&mut self, overlay_x: f32, overlay_y: f32, is_primary: bool) {
        if !is_primary {
            return;
        }
        self.mouse_position = (overlay_x, overlay_y);
        self.pending_imgui_events.push(ImguiInputEvent::MousePos(overlay_x, overlay_y));
        self.pending_imgui_events.push(ImguiInputEvent::MouseButton(imgui::MouseButton::Left, true));
    }

    /// Feed a touch-move update for the primary finger. No-op otherwise.
    pub fn on_overlay_touch_move(&mut self, overlay_x: f32, overlay_y: f32, is_primary: bool) {
        if !is_primary {
            return;
        }
        self.mouse_position = (overlay_x, overlay_y);
        self.pending_imgui_events.push(ImguiInputEvent::MousePos(overlay_x, overlay_y));
    }

    /// Feed a touch-up release. `last_finger` should be true when no touches
    /// remain, so the synthetic primary click is released.
    pub fn on_overlay_touch_up(&mut self, last_finger: bool) {
        if !last_finger {
            return;
        }
        self.pending_imgui_events.push(ImguiInputEvent::MouseButton(imgui::MouseButton::Left, false));
    }

    /// Drain the buffered event queue into imgui's IO. Called from `UI::draw`
    /// before `imgui.new_frame()`.
    pub(crate) fn drain_to_imgui(&mut self, io: &mut imgui::Io) {
        for ev in self.pending_imgui_events.drain(..) {
            match ev {
                ImguiInputEvent::Key(k, down) => io.add_key_event(k, down),
                ImguiInputEvent::Char(c) => io.add_input_character(c),
                ImguiInputEvent::MousePos(x, y) => io.add_mouse_pos_event([x, y]),
                ImguiInputEvent::MouseButton(b, down) => io.add_mouse_button_event(b, down),
                ImguiInputEvent::MouseWheel(dx, dy) => io.add_mouse_wheel_event([dx, dy]),
            }
        }
    }

    /// Set the desired text-input-active state. Called by `UI::draw` from
    /// `imgui::Io::want_text_input`.
    pub(crate) fn set_text_input_active(&mut self, active: bool) {
        self.text_input_active = active;
    }

    /// Called by backends once per event-loop iteration. Returns
    /// `Some(active)` on transitions so the backend can apply the change via
    /// its native API (SDL2 `text_input().start()/stop()`, winit
    /// `set_ime_allowed`, libnx swkbd, etc.).
    pub(crate) fn consume_text_input_change(&mut self) -> Option<bool> {
        if self.text_input_applied != self.text_input_active {
            self.text_input_applied = self.text_input_active;
            Some(self.text_input_active)
        } else {
            None
        }
    }
}

impl Default for InputContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Map a doukutsu-rs `ScanCode` to the corresponding `imgui::Key`. Variants
/// imgui doesn't track (media keys, Sleep/Wake, OS-specific extras) return
/// `None`. The `match` is exhaustive on purpose — adding a `ScanCode` variant
/// is a compile error until it's mapped (or explicitly ignored here).
fn scancode_to_imgui_key(s: ScanCode) -> Option<imgui::Key> {
    use imgui::Key as K;
    Some(match s {
        ScanCode::Key0 => K::Alpha0,
        ScanCode::Key1 => K::Alpha1,
        ScanCode::Key2 => K::Alpha2,
        ScanCode::Key3 => K::Alpha3,
        ScanCode::Key4 => K::Alpha4,
        ScanCode::Key5 => K::Alpha5,
        ScanCode::Key6 => K::Alpha6,
        ScanCode::Key7 => K::Alpha7,
        ScanCode::Key8 => K::Alpha8,
        ScanCode::Key9 => K::Alpha9,

        ScanCode::A => K::A,
        ScanCode::B => K::B,
        ScanCode::C => K::C,
        ScanCode::D => K::D,
        ScanCode::E => K::E,
        ScanCode::F => K::F,
        ScanCode::G => K::G,
        ScanCode::H => K::H,
        ScanCode::I => K::I,
        ScanCode::J => K::J,
        ScanCode::K => K::K,
        ScanCode::L => K::L,
        ScanCode::M => K::M,
        ScanCode::N => K::N,
        ScanCode::O => K::O,
        ScanCode::P => K::P,
        ScanCode::Q => K::Q,
        ScanCode::R => K::R,
        ScanCode::S => K::S,
        ScanCode::T => K::T,
        ScanCode::U => K::U,
        ScanCode::V => K::V,
        ScanCode::W => K::W,
        ScanCode::X => K::X,
        ScanCode::Y => K::Y,
        ScanCode::Z => K::Z,

        ScanCode::Escape => K::Escape,

        ScanCode::F1 => K::F1,
        ScanCode::F2 => K::F2,
        ScanCode::F3 => K::F3,
        ScanCode::F4 => K::F4,
        ScanCode::F5 => K::F5,
        ScanCode::F6 => K::F6,
        ScanCode::F7 => K::F7,
        ScanCode::F8 => K::F8,
        ScanCode::F9 => K::F9,
        ScanCode::F10 => K::F10,
        ScanCode::F11 => K::F11,
        ScanCode::F12 => K::F12,

        ScanCode::Snapshot | ScanCode::Sysrq => K::PrintScreen,
        ScanCode::Scroll | ScanCode::Scrolllock => K::ScrollLock,
        ScanCode::Pause => K::Pause,

        ScanCode::Insert => K::Insert,
        ScanCode::Home => K::Home,
        ScanCode::Delete => K::Delete,
        ScanCode::End => K::End,
        ScanCode::PageDown => K::PageDown,
        ScanCode::PageUp => K::PageUp,

        ScanCode::Left => K::LeftArrow,
        ScanCode::Up => K::UpArrow,
        ScanCode::Right => K::RightArrow,
        ScanCode::Down => K::DownArrow,

        ScanCode::Backspace | ScanCode::Back => K::Backspace,
        ScanCode::Return => K::Enter,
        ScanCode::Space => K::Space,
        ScanCode::Tab => K::Tab,

        ScanCode::Numlock => K::NumLock,
        ScanCode::Numpad0 => K::Keypad0,
        ScanCode::Numpad1 => K::Keypad1,
        ScanCode::Numpad2 => K::Keypad2,
        ScanCode::Numpad3 => K::Keypad3,
        ScanCode::Numpad4 => K::Keypad4,
        ScanCode::Numpad5 => K::Keypad5,
        ScanCode::Numpad6 => K::Keypad6,
        ScanCode::Numpad7 => K::Keypad7,
        ScanCode::Numpad8 => K::Keypad8,
        ScanCode::Numpad9 => K::Keypad9,
        ScanCode::NumpadAdd => K::KeypadAdd,
        ScanCode::NumpadDivide => K::KeypadDivide,
        ScanCode::NumpadDecimal => K::KeypadDecimal,
        ScanCode::NumpadEnter => K::KeypadEnter,
        ScanCode::NumpadEquals => K::KeypadEqual,
        ScanCode::NumpadMultiply => K::KeypadMultiply,
        ScanCode::NumpadSubtract => K::KeypadSubtract,

        ScanCode::Apostrophe => K::Apostrophe,
        ScanCode::Backslash => K::Backslash,
        ScanCode::Capslock | ScanCode::Capital => K::CapsLock,
        ScanCode::Comma => K::Comma,
        ScanCode::Equals => K::Equal,
        ScanCode::Grave => K::GraveAccent,
        ScanCode::LAlt => K::LeftAlt,
        ScanCode::LBracket => K::LeftBracket,
        ScanCode::LControl => K::LeftCtrl,
        ScanCode::LShift => K::LeftShift,
        ScanCode::LWin => K::LeftSuper,
        ScanCode::Menu | ScanCode::Apps => K::Menu,
        ScanCode::Minus => K::Minus,
        ScanCode::Period => K::Period,
        ScanCode::RAlt => K::RightAlt,
        ScanCode::RBracket => K::RightBracket,
        ScanCode::RControl => K::RightCtrl,
        ScanCode::RShift => K::RightShift,
        ScanCode::RWin => K::RightSuper,
        ScanCode::Semicolon => K::Semicolon,
        ScanCode::Slash => K::Slash,

        // Variants imgui doesn't track — drs-specific or OS extras.
        ScanCode::F13
        | ScanCode::F14
        | ScanCode::F15
        | ScanCode::F16
        | ScanCode::F17
        | ScanCode::F18
        | ScanCode::F19
        | ScanCode::F20
        | ScanCode::F21
        | ScanCode::F22
        | ScanCode::F23
        | ScanCode::F24
        | ScanCode::Compose
        | ScanCode::NonUsHash
        | ScanCode::NonUsBackslash
        | ScanCode::Caret
        | ScanCode::NumpadComma
        | ScanCode::AbntC1
        | ScanCode::AbntC2
        | ScanCode::Asterisk
        | ScanCode::At
        | ScanCode::Ax
        | ScanCode::Calculator
        | ScanCode::Colon
        | ScanCode::Convert
        | ScanCode::Kana
        | ScanCode::Kanji
        | ScanCode::Mail
        | ScanCode::MediaSelect
        | ScanCode::MediaStop
        | ScanCode::Mute
        | ScanCode::MyComputer
        | ScanCode::NavigateForward
        | ScanCode::NavigateBackward
        | ScanCode::NextTrack
        | ScanCode::NoConvert
        | ScanCode::OEM102
        | ScanCode::PlayPause
        | ScanCode::Plus
        | ScanCode::Power
        | ScanCode::PrevTrack
        | ScanCode::Sleep
        | ScanCode::Stop
        | ScanCode::Underline
        | ScanCode::Unlabeled
        | ScanCode::VolumeDown
        | ScanCode::VolumeUp
        | ScanCode::Wake
        | ScanCode::WebBack
        | ScanCode::WebFavorites
        | ScanCode::WebForward
        | ScanCode::WebHome
        | ScanCode::WebRefresh
        | ScanCode::WebSearch
        | ScanCode::WebStop
        | ScanCode::Yen
        | ScanCode::Copy
        | ScanCode::Paste
        | ScanCode::Cut => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_key_mappings() {
        assert_eq!(scancode_to_imgui_key(ScanCode::A), Some(imgui::Key::A));
        assert_eq!(scancode_to_imgui_key(ScanCode::Return), Some(imgui::Key::Enter));
        assert_eq!(scancode_to_imgui_key(ScanCode::F1), Some(imgui::Key::F1));
        assert_eq!(scancode_to_imgui_key(ScanCode::Key0), Some(imgui::Key::Alpha0));
        assert_eq!(scancode_to_imgui_key(ScanCode::LShift), Some(imgui::Key::LeftShift));
        assert_eq!(scancode_to_imgui_key(ScanCode::Sleep), None);
        assert_eq!(scancode_to_imgui_key(ScanCode::VolumeUp), None);
    }
}
