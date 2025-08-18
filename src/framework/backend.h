#pragma once

#include "../common.h"
#include <memory>
#include <vector>
#include <any>

// Forward declaration
namespace doukutsu {
class Game;
}

namespace doukutsu {
namespace framework {

// Forward declarations
class Context;

/// VSyncMode for renderer
enum class VSyncMode {
    Uncapped,
    VSync,
    Adaptive
};

/// Window parameters
struct WindowParams {
    uint16_t width = 640;
    uint16_t height = 480;
    bool fullscreen = false;
    bool resizable = true;
};

/// Vertex data for triangle rendering
struct VertexData {
    std::pair<float, float> position;
    doukutsu::common::Color color;
    std::pair<float, float> uv;
};

/// Available shader types
enum class BackendShader {
    Fill,
    Texture,
    WaterFill
};

/// Sprite batch drawing command data
struct SpriteBatchCommandData {
    doukutsu::common::Rect<float> src_rect;
    doukutsu::common::Rect<float> dest_rect;
    bool flip_x = false;
    bool flip_y = false;  
    doukutsu::common::Color tint{1.0f, 1.0f, 1.0f, 1.0f};
};

/// Sprite batch drawing commands
enum class SpriteBatchCommand {
    DrawRect,
    DrawRectFlip, 
    DrawRectTinted,
    DrawRectFlipTinted
};

/// Blend modes for rendering
enum class BlendMode {
    Alpha,
    Add,
    Multiply,
    None
};

/// Graphics backend texture interface
class BackendTexture {
public:
    virtual ~BackendTexture() = default;
    
    [[nodiscard]] virtual std::pair<uint16_t, uint16_t> dimensions() const = 0;
    virtual void add(SpriteBatchCommand command, const SpriteBatchCommandData& data) = 0;
    virtual void clear() = 0;
    virtual GameResult draw() = 0;
    
    [[nodiscard]] virtual std::any as_any() const = 0;
};

/// Graphics backend gamepad interface  
class BackendGamepad {
public:
    virtual ~BackendGamepad() = default;
    
    virtual GameResult set_rumble(uint16_t low_freq, uint16_t high_freq, uint32_t duration_ms) = 0;
    [[nodiscard]] virtual uint32_t instance_id() const = 0;
};

/// Graphics backend renderer interface
class BackendRenderer {
public:
    virtual ~BackendRenderer() = default;
    
    [[nodiscard]] virtual std::string renderer_name() const = 0;
    virtual void clear(doukutsu::common::Color color) = 0;
    virtual GameResult present() = 0;
    
    virtual GameResult set_vsync_mode(VSyncMode mode) { return GameResult{}; }
    virtual GameResult prepare_draw(float width, float height) { return GameResult{}; }
    
    virtual std::unique_ptr<BackendTexture> create_texture_mutable(uint16_t width, uint16_t height) = 0;
    virtual std::unique_ptr<BackendTexture> create_texture(uint16_t width, uint16_t height, const std::vector<uint8_t>& data) = 0;
    
    virtual GameResult set_blend_mode(BlendMode blend) = 0;
    virtual GameResult set_render_target(const BackendTexture* texture) = 0;
    
    virtual GameResult draw_rect(const doukutsu::common::Rect<int>& rect, doukutsu::common::Color color) = 0;
    virtual GameResult draw_outline_rect(const doukutsu::common::Rect<int>& rect, size_t line_width, doukutsu::common::Color color) = 0;
    virtual GameResult set_clip_rect(const std::optional<doukutsu::common::Rect<int>>& rect) = 0;
    
    [[nodiscard]] virtual bool supports_vertex_draw() const { return false; }
    virtual GameResult draw_triangle_list(
        const std::vector<VertexData>& vertices,
        const BackendTexture* texture,
        BackendShader shader) = 0;
        
    [[nodiscard]] virtual std::any as_any() const = 0;
};

/// Backend event loop interface
class BackendEventLoop {
public:
    virtual ~BackendEventLoop() = default;
    
    virtual void run(doukutsu::Game& game, Context& ctx) = 0;
    virtual std::unique_ptr<BackendRenderer> new_renderer(Context* ctx) = 0;
    
    [[nodiscard]] virtual std::any as_any() const = 0;
};

/// Main backend interface
class Backend {
public:
    virtual ~Backend() = default;
    
    virtual std::unique_ptr<BackendEventLoop> create_event_loop(Context& ctx) = 0;
    [[nodiscard]] virtual std::any as_any() const = 0;
};

/// Initialize and create a backend
[[nodiscard]] std::unique_ptr<Backend> init_backend(bool headless, const WindowParams& window_params);

} // namespace framework  
} // namespace doukutsu