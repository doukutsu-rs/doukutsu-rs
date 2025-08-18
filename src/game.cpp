#include "game.h"
#include "framework/context.h"

namespace doukutsu {

Game::Game() = default;
Game::~Game() = default;

GameResult Game::update(framework::Context& ctx) {
    // Basic game update loop - for now just a placeholder
    // This will be expanded with actual game logic later
}

GameResult Game::draw(framework::Context& ctx) {
    // Basic game rendering - for now just clear screen
    if (ctx.renderer) {
        ctx.renderer->clear(doukutsu::common::Color::from_rgb(0, 0, 0));
        ctx.renderer->present();
    }
}

} // namespace doukutsu