#include "game.h"
#include "framework/context.h"
#include "framework/error.h"
#include <iostream>

int main() {
    try {
        doukutsu::framework::Context ctx;
        doukutsu::Game game;
        
        std::cout << "Starting doukutsu-rs C++ port..." << std::endl;
        ctx.run(game);
        
    } catch (const doukutsu::framework::GameError& e) {
        std::cerr << "Game error: " << e.what() << std::endl;
        return 1;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }
    
    return 0;
}