#pragma once

namespace doukutsu_rs {
namespace game {

/**
 * @brief (Opaque) game instance
 */
class Game;

struct LaunchOptions {
  bool server_mode;
  bool editor;
};

} // namespace game
} // namespace doukutsu_rs