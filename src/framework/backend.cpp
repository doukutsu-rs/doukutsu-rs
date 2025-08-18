#include "backend.h"
#include "error.h"

namespace doukutsu {
namespace framework {

// Null implementations for headless/testing mode

class NullTexture : public BackendTexture {
public:
    NullTexture(uint16_t width, uint16_t height) : width_(width), height_(height) {}
    
    std::pair<uint16_t, uint16_t> dimensions() const override {
        return {width_, height_};
    }
    
    void add(SpriteBatchCommand command, const SpriteBatchCommandData& data) override {
        // No-op
    }
    
    void clear() override {
        // No-op  
    }
    
    GameResult draw() override {
        // No-op
    }
    
    std::any as_any() const override {
        return static_cast<const void*>(this);
    }

private:
    uint16_t width_, height_;
};

class NullRenderer : public BackendRenderer {
public:
    std::string renderer_name() const override {
        return "NullRenderer";
    }
    
    void clear(doukutsu::common::Color color) override {
        // No-op
    }
    
    GameResult present() override {
        // No-op
    }
    
    std::unique_ptr<BackendTexture> create_texture_mutable(uint16_t width, uint16_t height) override {
        return std::make_unique<NullTexture>(width, height);
    }
    
    std::unique_ptr<BackendTexture> create_texture(uint16_t width, uint16_t height, const std::vector<uint8_t>& data) override {
        return std::make_unique<NullTexture>(width, height);
    }
    
    GameResult set_blend_mode(BlendMode blend) override {
        // No-op
    }
    
    GameResult set_render_target(const BackendTexture* texture) override {
        // No-op
    }
    
    GameResult draw_rect(const doukutsu::common::Rect<int>& rect, doukutsu::common::Color color) override {
        // No-op
    }
    
    GameResult draw_outline_rect(const doukutsu::common::Rect<int>& rect, size_t line_width, doukutsu::common::Color color) override {
        // No-op
    }
    
    GameResult set_clip_rect(const std::optional<doukutsu::common::Rect<int>>& rect) override {
        // No-op
    }
    
    GameResult draw_triangle_list(
        const std::vector<VertexData>& vertices,
        const BackendTexture* texture,
        BackendShader shader) override {
        // No-op
    }
    
    std::any as_any() const override {
        return static_cast<const void*>(this);
    }
};

class NullEventLoop : public BackendEventLoop {
public:
    void run(doukutsu::Game& game, Context& ctx) override {
        // For now, just exit immediately in null backend
        // In the future this could run a simple frame loop for testing
    }
    
    std::unique_ptr<BackendRenderer> new_renderer(Context* ctx) override {
        return std::make_unique<NullRenderer>();
    }
    
    std::any as_any() const override {
        return static_cast<const void*>(this);
    }
};

class NullBackend : public Backend {
public:
    std::unique_ptr<BackendEventLoop> create_event_loop(Context& ctx) override {
        return std::make_unique<NullEventLoop>();
    }
    
    std::any as_any() const override {
        return static_cast<const void*>(this);
    }
};

// Backend factory function
std::unique_ptr<Backend> init_backend(bool headless, const WindowParams& window_params) {
    if (headless) {
        return std::make_unique<NullBackend>();
    }
    
    // For now, always return null backend until SDL2 backend is implemented
    return std::make_unique<NullBackend>();
}

} // namespace framework
} // namespace doukutsu