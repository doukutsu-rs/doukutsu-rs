#pragma once

#include "common.h"

namespace doukutsu {
namespace framework {
class Context;
}

/// Main game class
class Game {
public:
    Game();
    ~Game();
    
    /// Update game state
    GameResult update(framework::Context& ctx);
    
    /// Render game
    GameResult draw(framework::Context& ctx);

private:
    Game(const Game&) = delete;
    Game& operator=(const Game&) = delete;
};

} // namespace doukutsu