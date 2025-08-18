#include "shared_game_state.h"

namespace doukutsu_rs::shared_game_state
{

void SharedGameState::create_caret(int32_t x, int32_t y, caret::CaretType ctype, common::Direction direct)
{
    // TODO: Implement caret creation properly
    // For now, this is a stub implementation to satisfy the linker
    caret::Caret new_caret(x, y, ctype, direct, constants);
    carets.push_back(new_caret);
}

}