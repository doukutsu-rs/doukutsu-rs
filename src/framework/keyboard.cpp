#include "keyboard.h"

namespace doukutsu {
namespace framework {

KeyboardContext::KeyboardContext() = default;
KeyboardContext::~KeyboardContext() = default;

void KeyboardContext::set_key(ScanCode key, bool pressed) {
    if (key < ScanCode::COUNT) {
        current_state_[static_cast<size_t>(key)] = pressed;
    }
}

bool KeyboardContext::is_key_pressed(ScanCode key) const {
    if (key < ScanCode::COUNT) {
        return current_state_[static_cast<size_t>(key)];
    }
    return false;
}

bool KeyboardContext::is_key_triggered(ScanCode key) const {
    if (key < ScanCode::COUNT) {
        size_t index = static_cast<size_t>(key);
        return current_state_[index] && !previous_state_[index];
    }
    return false;
}

void KeyboardContext::update() {
    previous_state_ = current_state_;
}

} // namespace framework
} // namespace doukutsu