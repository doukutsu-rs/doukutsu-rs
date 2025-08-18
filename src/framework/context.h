#pragma once

#include "../common.h"
#include "backend.h"
#include <memory>

// Forward declaration
namespace doukutsu {
class Game;
}

namespace doukutsu {
namespace framework {

// Forward declarations
class Filesystem;
class GamepadContext;
class KeyboardContext;

/// Framework context - manages core system state
class Context {
public:
    Context();
    ~Context();

    // Core properties
    bool headless = false;
    WindowParams window;
    std::pair<uint32_t, uint32_t> real_screen_size{320, 240};
    std::pair<float, float> screen_size{320.0f, 240.0f};
    std::tuple<float, float, float, float> screen_insets{0.0f, 0.0f, 0.0f, 0.0f};
    VSyncMode vsync_mode = VSyncMode::Uncapped;

    // Subsystems
    std::unique_ptr<Filesystem> filesystem;
    std::unique_ptr<BackendRenderer> renderer;
    std::unique_ptr<GamepadContext> gamepad_context;
    std::unique_ptr<KeyboardContext> keyboard_context;

    // Main entry point
    GameResult run(doukutsu::Game& game);

private:
    Context(const Context&) = delete;
    Context& operator=(const Context&) = delete;
};

} // namespace framework
} // namespace doukutsu