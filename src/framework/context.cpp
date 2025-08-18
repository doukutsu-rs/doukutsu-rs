#include "context.h"
#include "backend.h"
#include "filesystem.h"
#include "keyboard.h"
#include "gamepad.h"
#include "error.h"
#include "../game.h"

namespace doukutsu {
namespace framework {

Context::Context() 
    : filesystem(std::make_unique<Filesystem>())
    , keyboard_context(std::make_unique<KeyboardContext>())
    , gamepad_context(std::make_unique<GamepadContext>())
{
}

Context::~Context() = default;

GameResult Context::run(doukutsu::Game& game) {
    try {
        auto backend = init_backend(headless, window);
        auto event_loop = backend->create_event_loop(*this);
        renderer = event_loop->new_renderer(this);

        event_loop->run(game, *this);
    } catch (const GameError& e) {
        throw; // Re-throw for caller to handle
    }
}

} // namespace framework
} // namespace doukutsu