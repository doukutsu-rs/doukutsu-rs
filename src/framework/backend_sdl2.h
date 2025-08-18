#pragma once

#include "backend.h"
#include <memory>

namespace doukutsu {
namespace framework {

/// SDL2 Backend implementation  
class SDL2Backend : public Backend {
public:
    explicit SDL2Backend(const WindowParams& window_params);
    ~SDL2Backend() override;

    std::unique_ptr<BackendEventLoop> create_event_loop(Context& ctx) override;
    std::any as_any() const override;

private:
    WindowParams window_params_;
    // SDL context will be stored here when we add SDL2 dependency
};

/// SDL2 Event Loop implementation
class SDL2EventLoop : public BackendEventLoop {
public:
    explicit SDL2EventLoop(const WindowParams& window_params);
    ~SDL2EventLoop() override;

    void run(doukutsu::Game& game, Context& ctx) override;
    std::unique_ptr<BackendRenderer> new_renderer(Context* ctx) override;
    std::any as_any() const override;

private:
    WindowParams window_params_;
    // SDL event pump and window will be stored here
};

/// SDL2 Renderer implementation
class SDL2Renderer : public BackendRenderer {
public:
    SDL2Renderer();
    ~SDL2Renderer() override;

    std::string renderer_name() const override;
    void clear(doukutsu::common::Color color) override;
    GameResult present() override;

    std::unique_ptr<BackendTexture> create_texture_mutable(uint16_t width, uint16_t height) override;
    std::unique_ptr<BackendTexture> create_texture(uint16_t width, uint16_t height, const std::vector<uint8_t>& data) override;

    GameResult set_blend_mode(BlendMode blend) override;
    GameResult set_render_target(const BackendTexture* texture) override;

    GameResult draw_rect(const doukutsu::common::Rect<int>& rect, doukutsu::common::Color color) override;
    GameResult draw_outline_rect(const doukutsu::common::Rect<int>& rect, size_t line_width, doukutsu::common::Color color) override;
    GameResult set_clip_rect(const std::optional<doukutsu::common::Rect<int>>& rect) override;

    GameResult draw_triangle_list(
        const std::vector<VertexData>& vertices,
        const BackendTexture* texture,
        BackendShader shader) override;

    std::any as_any() const override;

private:
    // SDL renderer will be stored here
};

/// SDL2 Texture implementation
class SDL2Texture : public BackendTexture {
public:
    SDL2Texture(uint16_t width, uint16_t height);
    ~SDL2Texture() override;

    std::pair<uint16_t, uint16_t> dimensions() const override;
    void add(SpriteBatchCommand command, const SpriteBatchCommandData& data) override;
    void clear() override;
    GameResult draw() override;
    std::any as_any() const override;

private:
    uint16_t width_, height_;
    std::vector<std::pair<SpriteBatchCommand, SpriteBatchCommandData>> commands_;
    // SDL texture will be stored here
};

} // namespace framework
} // namespace doukutsu