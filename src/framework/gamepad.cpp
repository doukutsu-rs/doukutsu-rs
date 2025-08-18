#include "gamepad.h"

namespace doukutsu {
namespace framework {

GamepadContext::GamepadContext() = default;
GamepadContext::~GamepadContext() = default;

void GamepadContext::add_gamepad(std::unique_ptr<BackendGamepad> gamepad, float axis_sensitivity) {
    uint32_t instance_id = gamepad->instance_id();
    GamepadState state;
    state.gamepad = std::move(gamepad);
    state.axis_sensitivity = axis_sensitivity;
    
    gamepads_[instance_id] = std::move(state);
}

void GamepadContext::remove_gamepad(uint32_t instance_id) {
    gamepads_.erase(instance_id);
}

void GamepadContext::set_button(uint32_t instance_id, Button button, bool pressed) {
    auto it = gamepads_.find(instance_id);
    if (it != gamepads_.end()) {
        it->second.button_states[button] = pressed;
    }
}

void GamepadContext::set_axis_value(uint32_t instance_id, Axis axis, double value) {
    auto it = gamepads_.find(instance_id);
    if (it != gamepads_.end()) {
        it->second.axis_values[axis] = value;
    }
}

void GamepadContext::update_axes(uint32_t instance_id) {
    auto it = gamepads_.find(instance_id);
    if (it != gamepads_.end()) {
        const auto& state = it->second;
        
        // Convert analog stick positions to D-pad inputs
        auto left_x = state.axis_values.find(Axis::LeftX);
        auto left_y = state.axis_values.find(Axis::LeftY);
        
        if (left_x != state.axis_values.end()) {
            if (left_x->second < -state.axis_sensitivity) {
                it->second.button_states[Button::DPadLeft] = true;
            } else if (left_x->second > state.axis_sensitivity) {
                it->second.button_states[Button::DPadRight] = true;
            }
        }
        
        if (left_y != state.axis_values.end()) {
            if (left_y->second < -state.axis_sensitivity) {
                it->second.button_states[Button::DPadUp] = true;
            } else if (left_y->second > state.axis_sensitivity) {
                it->second.button_states[Button::DPadDown] = true;
            }
        }
    }
}

void GamepadContext::set_gamepad_type(uint32_t instance_id, GamepadType type) {
    auto it = gamepads_.find(instance_id);
    if (it != gamepads_.end()) {
        it->second.type = type;
    }
}

bool GamepadContext::is_button_pressed(Button button) const {
    for (const auto& [id, state] : gamepads_) {
        auto it = state.button_states.find(button);
        if (it != state.button_states.end() && it->second) {
            return true;
        }
    }
    return false;
}

double GamepadContext::get_axis_value(Axis axis) const {
    for (const auto& [id, state] : gamepads_) {
        auto it = state.axis_values.find(axis);
        if (it != state.axis_values.end()) {
            return it->second;
        }
    }
    return 0.0;
}

} // namespace framework
} // namespace doukutsu