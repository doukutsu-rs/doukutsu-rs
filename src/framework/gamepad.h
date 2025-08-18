#pragma once

#include "../common.h"
#include "backend.h"
#include <memory>
#include <unordered_map>

namespace doukutsu {
namespace framework {

/// Gamepad button types
enum class Button {
    South,      // A/Cross
    East,       // B/Circle  
    West,       // X/Square
    North,      // Y/Triangle
    Back,
    Guide,
    Start,
    LeftStick,
    RightStick,
    LeftShoulder,
    RightShoulder,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
};

/// Gamepad axis types  
enum class Axis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    TriggerLeft,
    TriggerRight,
};

/// Gamepad types for different controller support
enum class GamepadType {
    Unknown,
    Xbox360,
    XboxOne,
    PS3,
    PS4,
    PS5,
    NintendoSwitchPro,
    NintendoSwitchJoyConLeft,
    NintendoSwitchJoyConRight,
    NintendoSwitchJoyConPair,
    Virtual,
    AmazonLuma,
    GoogleStadia,
    NVIDIAShield,
};

/// Gamepad input context
class GamepadContext {
public:
    GamepadContext();
    ~GamepadContext();

    /// Add a gamepad
    void add_gamepad(std::unique_ptr<BackendGamepad> gamepad, float axis_sensitivity = 0.5f);
    
    /// Remove a gamepad by instance ID
    void remove_gamepad(uint32_t instance_id);
    
    /// Set button state
    void set_button(uint32_t instance_id, Button button, bool pressed);
    
    /// Set axis value (-1.0 to 1.0)  
    void set_axis_value(uint32_t instance_id, Axis axis, double value);
    
    /// Update axis button states based on current axis values
    void update_axes(uint32_t instance_id);
    
    /// Set gamepad type
    void set_gamepad_type(uint32_t instance_id, GamepadType type);
    
    /// Check if button is pressed on any gamepad
    [[nodiscard]] bool is_button_pressed(Button button) const;
    
    /// Get axis value from first available gamepad
    [[nodiscard]] double get_axis_value(Axis axis) const;

private:
    struct GamepadState {
        std::unique_ptr<BackendGamepad> gamepad;
        GamepadType type = GamepadType::Unknown;
        float axis_sensitivity = 0.5f;
        std::unordered_map<Button, bool> button_states;
        std::unordered_map<Axis, double> axis_values;
    };
    
    std::unordered_map<uint32_t, GamepadState> gamepads_;
};

} // namespace framework  
} // namespace doukutsu