#pragma once

#include "../common.h"
#include <unordered_map>
#include <array>

namespace doukutsu {
namespace framework {

/// Keyboard scan codes (similar to SDL scancodes)
enum class ScanCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Key0,
    Return, Escape, Backspace, Tab, Space,
    Minus, Equals, LBracket, RBracket, Backslash, NonUsHash,
    Semicolon, Apostrophe, Grave, Comma, Period, Slash,
    Capslock,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Sysrq, Scrolllock, Pause, Insert, Home, PageUp, Delete, End, PageDown,
    Right, Left, Down, Up,
    Numlock, NumpadDivide, NumpadMultiply, NumpadSubtract, NumpadAdd, NumpadEnter,
    Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9, Numpad0,
    NonUsBackslash, Apps, Power, NumpadEquals,
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,
    Stop, Cut, Copy, Paste, Mute, VolumeUp, VolumeDown,
    NumpadComma,
    LControl, LShift, LAlt, LWin, RControl, RShift, RAlt, RWin,
    NextTrack, PrevTrack, MediaStop, PlayPause, MediaSelect, Mail, Calculator, Sleep,
    COUNT // Keep this last
};

/// Keyboard input context
class KeyboardContext {
public:
    KeyboardContext();
    ~KeyboardContext();

    /// Set key state
    void set_key(ScanCode key, bool pressed);
    
    /// Check if key is currently pressed
    [[nodiscard]] bool is_key_pressed(ScanCode key) const;
    
    /// Check if key was just pressed this frame
    [[nodiscard]] bool is_key_triggered(ScanCode key) const;
    
    /// Update key states (call once per frame)
    void update();

private:
    std::array<bool, static_cast<size_t>(ScanCode::COUNT)> current_state_{};
    std::array<bool, static_cast<size_t>(ScanCode::COUNT)> previous_state_{};
};

} // namespace framework
} // namespace doukutsu