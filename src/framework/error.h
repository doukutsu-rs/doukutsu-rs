#pragma once

#include <stdexcept>
#include <string>

namespace doukutsu {
namespace framework {

/// Game error types
enum class GameErrorType {
    WindowError,
    RenderError,
    GamepadError,
    IOError,
    ParseError,
    Unknown
};

/// Game error exception class
class GameError : public std::runtime_error {
public:
    explicit GameError(GameErrorType type, const std::string& message)
        : std::runtime_error(message), type_(type) {}
        
    [[nodiscard]] GameErrorType type() const noexcept { return type_; }

    // Factory methods for common error types
    [[nodiscard]] static GameError window_error(const std::string& message) {
        return GameError(GameErrorType::WindowError, message);
    }
    
    [[nodiscard]] static GameError render_error(const std::string& message) {
        return GameError(GameErrorType::RenderError, message);
    }
    
    [[nodiscard]] static GameError gamepad_error(const std::string& message) {
        return GameError(GameErrorType::GamepadError, message);
    }
    
    [[nodiscard]] static GameError io_error(const std::string& message) {
        return GameError(GameErrorType::IOError, message);
    }

private:
    GameErrorType type_;
};

} // namespace framework
} // namespace doukutsu