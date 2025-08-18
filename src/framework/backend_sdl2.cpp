#include "backend_sdl2.h"
#include "error.h"
#include "../game.h"
#include <iostream>

namespace doukutsu {
namespace framework {

// SDL2Backend implementation
SDL2Backend::SDL2Backend(const WindowParams& window_params) 
    : window_params_(window_params) {
    // TODO: Initialize SDL2 context when dependencies are added
    std::cout << "SDL2Backend created (stub implementation)" << std::endl;
}

SDL2Backend::~SDL2Backend() = default;

std::unique_ptr<BackendEventLoop> SDL2Backend::create_event_loop(Context& ctx) {
    return std::make_unique<SDL2EventLoop>(window_params_);
}

std::any SDL2Backend::as_any() const {
    return static_cast<const void*>(this);
}

// SDL2EventLoop implementation
SDL2EventLoop::SDL2EventLoop(const WindowParams& window_params) 
    : window_params_(window_params) {
    // TODO: Initialize SDL2 window and event pump
    std::cout << "SDL2EventLoop created with " << window_params.width 
              << "x" << window_params.height << " window" << std::endl;
}

SDL2EventLoop::~SDL2EventLoop() = default;

void SDL2EventLoop::run(doukutsu::Game& game, Context& ctx) {
    std::cout << "SDL2EventLoop::run() - starting game loop" << std::endl;
    
    // Basic game loop simulation for now
    // TODO: Replace with real SDL2 event handling
    for (int frame = 0; frame < 60; ++frame) { // Run for 60 frames as demo
        // Update game state
        game.update(ctx);
        
        // Render frame
        game.draw(ctx);
        
        // Simple timing - in real implementation this would be handled by SDL
        // std::this_thread::sleep_for(std::chrono::milliseconds(16)); // ~60 FPS
        
        // For demo, just exit after a few frames
        if (frame > 3) {
            std::cout << "Demo complete after " << frame << " frames" << std::endl;
            break;
        }
    }
}

std::unique_ptr<BackendRenderer> SDL2EventLoop::new_renderer(Context* ctx) {
    return std::make_unique<SDL2Renderer>();
}

std::any SDL2EventLoop::as_any() const {
    return static_cast<const void*>(this);
}

// SDL2Renderer implementation  
SDL2Renderer::SDL2Renderer() {
    std::cout << "SDL2Renderer created" << std::endl;
}

SDL2Renderer::~SDL2Renderer() = default;

std::string SDL2Renderer::renderer_name() const {
    return "SDL2Renderer (stub)";
}

void SDL2Renderer::clear(doukutsu::common::Color color) {
    // TODO: Clear SDL2 renderer with color
    const auto [r, g, b, a] = color.to_rgba();
    // std::cout << "Clear with color: (" << static_cast<int>(r) << ", " 
    //           << static_cast<int>(g) << ", " << static_cast<int>(b) 
    //           << ", " << static_cast<int>(a) << ")" << std::endl;
}

GameResult SDL2Renderer::present() {
    // TODO: Present SDL2 renderer
    // std::cout << "Present frame" << std::endl;
}

std::unique_ptr<BackendTexture> SDL2Renderer::create_texture_mutable(uint16_t width, uint16_t height) {
    return std::make_unique<SDL2Texture>(width, height);
}

std::unique_ptr<BackendTexture> SDL2Renderer::create_texture(uint16_t width, uint16_t height, const std::vector<uint8_t>& data) {
    // TODO: Create texture from data
    return std::make_unique<SDL2Texture>(width, height);
}

GameResult SDL2Renderer::set_blend_mode(BlendMode blend) {
    // TODO: Set SDL2 blend mode
}

GameResult SDL2Renderer::set_render_target(const BackendTexture* texture) {
    // TODO: Set SDL2 render target
}

GameResult SDL2Renderer::draw_rect(const doukutsu::common::Rect<int>& rect, doukutsu::common::Color color) {
    // TODO: Draw rectangle with SDL2
}

GameResult SDL2Renderer::draw_outline_rect(const doukutsu::common::Rect<int>& rect, size_t line_width, doukutsu::common::Color color) {
    // TODO: Draw outline rectangle with SDL2
}

GameResult SDL2Renderer::set_clip_rect(const std::optional<doukutsu::common::Rect<int>>& rect) {
    // TODO: Set SDL2 clip rectangle
}

GameResult SDL2Renderer::draw_triangle_list(
    const std::vector<VertexData>& vertices,
    const BackendTexture* texture,
    BackendShader shader) {
    // TODO: Draw triangles with SDL2
}

std::any SDL2Renderer::as_any() const {
    return static_cast<const void*>(this);
}

// SDL2Texture implementation
SDL2Texture::SDL2Texture(uint16_t width, uint16_t height) 
    : width_(width), height_(height) {
    // TODO: Create SDL2 texture
    // std::cout << "SDL2Texture created: " << width << "x" << height << std::endl;
}

SDL2Texture::~SDL2Texture() = default;

std::pair<uint16_t, uint16_t> SDL2Texture::dimensions() const {
    return {width_, height_};
}

void SDL2Texture::add(SpriteBatchCommand command, const SpriteBatchCommandData& data) {
    commands_.emplace_back(command, data);
}

void SDL2Texture::clear() {
    commands_.clear();
}

GameResult SDL2Texture::draw() {
    // TODO: Execute all sprite batch commands
    // std::cout << "Drawing " << commands_.size() << " sprite batch commands" << std::endl;
}

std::any SDL2Texture::as_any() const {
    return static_cast<const void*>(this);
}

} // namespace framework
} // namespace doukutsu