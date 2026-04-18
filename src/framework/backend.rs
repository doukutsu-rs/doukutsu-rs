use std::any::Any;
use std::cell::RefMut;
use std::pin::Pin;
use std::rc::Rc;

use imgui::DrawData;

use super::context::Context;
use super::error::GameResult;
use super::graphics::{BlendMode, SwapMode};
use super::keyboard::ScanCode;
use crate::common::{Color, Rect};
use crate::framework::graphics::IndexData;
use crate::framework::render::effect::BackendEffect;
use crate::framework::render::vertex::VertexDeclaration;
use crate::game::shared_game_state::WindowMode;
use crate::game::Game;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VertexData {
    pub position: (f32, f32),
    pub color: (u8, u8, u8, u8),
    pub uv: (f32, f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BufferUsage {
    /// Write once, read many times.
    Static,
    /// Write frequently, read frequently.
    Dynamic,
    /// Write every frame (current behavior for sprite batches).
    Stream,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    TriangleList,
    TriangleStrip,
    LineList,
    LineStrip,
}

pub trait BackendVertexBuffer: Any {
    fn set_data_raw(&mut self, data: &[u8], offset: usize) -> GameResult;
    fn vertex_count(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
}

pub trait BackendIndexBuffer: Any {
    fn set_data(&mut self, data: IndexData, offset: usize) -> GameResult;
    fn index_count(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Copy, Clone, PartialEq)]
pub enum BackendShader {
    /// (scale, t, (frame_x, frame_y))
    WaterFill(f32, f32, (f32, f32)),
    Fill,
    Texture,
}

#[derive(Clone, Copy, PartialEq)]
pub enum DeviceFormFactor {
    /// A PC-like device, where the main input method is a keyboard and mouse.
    Computer,
    /// A mobile device, where the main input method is a touchscreen.
    Mobile,
    /// A console-like device, where the main input method is a gamepad.
    Console,
}

/// Represents capabilities of the current platform/backend.
#[derive(Clone, Copy)]
pub struct BackendFlag {
    flags: u8,
    form_factor: DeviceFormFactor,
}

macro_rules! flag_method {
    ($get_name:ident, $set_name:ident, $flag:ident) => {
        pub(crate) const fn $set_name(&mut self, value: bool) {
            if value {
                self.flags |= Self::$flag;
            } else {
                self.flags &= !Self::$flag;
            }
        }

        pub const fn $get_name(&self) -> bool {
            self.flags & Self::$flag != 0
        }
    };
}

impl BackendFlag {
    pub(crate) const SUPPORTS_WINDOWED_FULLSCREEN: u8 = 1 << 0;
    pub(crate) const HAS_TOUCH_SCREEN: u8 = 1 << 1;
    pub(crate) const SUPPORTS_INSETS: u8 = 1 << 2;

    pub(crate) const fn new() -> Self {
        Self { flags: 0, form_factor: DeviceFormFactor::Computer }
    }

    // setters (internal)

    pub(crate) const fn set_form_factor(&mut self, form_factor: DeviceFormFactor) {
        self.form_factor = form_factor;
    }

    flag_method!(supports_windowed_fullscreen, set_supports_windowed_fullscreen, SUPPORTS_WINDOWED_FULLSCREEN);
    flag_method!(has_touch_screen, set_has_touch_screen, HAS_TOUCH_SCREEN);
    flag_method!(supports_insets, set_supports_insets, SUPPORTS_INSETS);

    // accessors

    pub const fn form_factor(&self) -> DeviceFormFactor {
        self.form_factor
    }

    pub const fn supports_coop(&self) -> bool {
        cfg!(not(target_os = "android"))
    }

    /// Whether the operating system supports quitting the game from it's UI.
    pub const fn supports_quit(&self) -> bool {
        cfg!(not(any(
            target_os = "ios",
            // TODO: possible/recommended with hbmenu flow, but broken
            target_os = "horizon"
        )))
    }

    /// Whether the operating system supports running the game from any location, not static user/data directories.
    pub const fn supports_portable(&self) -> bool {
        cfg!(any(
            // only Windows desktop can run from any location, this isn't supported for UWP apps
            all(target_os = "windows", target_vendor = "pc"),
            target_os = "macos",
            // Whatever that can run Wayland/X11
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "dragonfly",
            target_os = "haiku",
            target_os = "illumos",
            target_os = "solaris",
        ))
    }

    /// Whether the operating system supports opening a directory in the file manager.
    pub const fn supports_open_directory(&self) -> bool {
        cfg!(any(
            target_os = "linux",
            target_os = "android",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "illumos",
            target_os = "solaris",
            target_os = "ios",
            target_os = "macos",
            target_os = "windows",
            target_os = "haiku"
        ))
    }
}

pub trait BackendCallbacks {
    fn on_fullscreen_state_changed(&mut self, ctx: &mut Context, new_mode: WindowMode) -> GameResult;

    fn on_resize(&mut self, ctx: &mut Context) -> GameResult;

    fn on_focus_gained(&mut self, ctx: &mut Context) -> GameResult;

    fn on_focus_lost(&mut self, ctx: &mut Context) -> GameResult;

    fn on_key_down(&mut self, ctx: &mut Context, key: ScanCode) -> GameResult;

    fn on_key_up(&mut self, ctx: &mut Context, key: ScanCode) -> GameResult;

    fn on_context_lost(&mut self, ctx: &mut Context) -> GameResult;
}

pub trait Backend {
    fn create_event_loop(&self, ctx: &Context) -> GameResult<Box<dyn BackendEventLoop>>;

    fn as_any(&self) -> &dyn Any;
}

pub trait BackendEventLoop {
    fn run(&mut self, game: Pin<Box<Game>>, ctx: Pin<Box<Context>>);

    fn new_renderer(&self, ctx: &mut Context) -> GameResult<Box<dyn BackendRenderer>>;

    fn as_any(&self) -> &dyn Any;
}

pub trait BackendRenderer {
    fn renderer_name(&self) -> String;

    fn clear(&mut self, color: Color);

    fn present(&mut self) -> GameResult;

    fn set_swap_mode(&mut self, _mode: SwapMode) -> GameResult {
        Ok(())
    }

    fn prepare_draw(&mut self, _width: f32, _height: f32) -> GameResult {
        Ok(())
    }

    /// Called before [`present`](Self::present) with the final window size and the viewport rectangle
    /// (in physical pixels) where the game canvas should be blitted. Areas outside `viewport` are
    /// cleared to black for letterboxing.
    fn set_output_viewport(&mut self, _window_size: (u32, u32), _viewport: Rect<u32>) -> GameResult {
        Ok(())
    }

    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>>;

    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>>;

    fn set_blend_mode(&mut self, blend: BlendMode) -> GameResult;

    fn set_render_target(&mut self, texture: Option<&Box<dyn BackendTexture>>) -> GameResult;

    fn draw_rect(&mut self, rect: Rect, color: Color) -> GameResult;

    fn draw_outline_rect(&mut self, rect: Rect, line_width: usize, color: Color) -> GameResult;

    fn set_clip_rect(&mut self, rect: Option<Rect>) -> GameResult;

    fn draw_triangles(
        &mut self,
        vertices: &[VertexData],
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult;

    fn draw_triangles_indexed(
        &mut self,
        vertices: &[VertexData],
        indices: IndexData,
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult;

    fn create_vertex_buffer(
        &mut self,
        _decl: VertexDeclaration,
        _count: usize,
        _usage: BufferUsage,
    ) -> GameResult<Box<dyn BackendVertexBuffer>> {
        Err(super::error::GameError::RenderError("Vertex buffers not supported by this backend.".to_string()))
    }

    fn create_index_buffer(&mut self, _count: usize, _usage: BufferUsage) -> GameResult<Box<dyn BackendIndexBuffer>> {
        Err(super::error::GameError::RenderError("Index buffers not supported by this backend.".to_string()))
    }

    fn create_effect_from_glsl(
        &mut self,
        _vertex_src: &str,
        _fragment_src: &str,
        _name: &str,
    ) -> GameResult<Box<dyn BackendEffect>> {
        Err(super::error::GameError::RenderError("Effects not supported by this backend.".to_string()))
    }

    /// Returns the current scene/surface texture (what's been rendered so far this frame).
    /// Used by game code for post-processing effects (e.g. water distortion).
    fn scene_texture(&self) -> Option<&dyn BackendTexture> {
        None
    }

    fn draw_primitives(
        &mut self,
        _effect: &mut dyn BackendEffect,
        _vb: &dyn BackendVertexBuffer,
        _decl: &VertexDeclaration,
        _prim_type: PrimitiveType,
        _start: usize,
        _count: usize,
    ) -> GameResult {
        Err(super::error::GameError::RenderError("draw_primitives not supported by this backend.".to_string()))
    }

    fn draw_indexed_primitives(
        &mut self,
        _effect: &mut dyn BackendEffect,
        _vb: &dyn BackendVertexBuffer,
        _ib: &dyn BackendIndexBuffer,
        _decl: &VertexDeclaration,
        _prim_type: PrimitiveType,
        _start_index: usize,
        _count: usize,
    ) -> GameResult {
        Err(super::error::GameError::RenderError("draw_indexed_primitives not supported by this backend.".to_string()))
    }

    fn as_any(&self) -> &dyn Any;
}

pub trait BackendTexture {
    fn dimensions(&self) -> (u16, u16);

    fn as_any(&self) -> &dyn Any;
}

pub trait BackendGamepad {
    fn set_rumble(&mut self, low_freq: u16, high_freq: u16, duration_ms: u32) -> GameResult;

    fn instance_id(&self) -> u32;
}

#[derive(Clone, Copy)]
pub struct WindowParams {
    pub size_hint: (u16, u16), // (width, height)
    pub mode: WindowMode,
}

impl Default for WindowParams {
    fn default() -> Self {
        Self { size_hint: (640, 480), mode: WindowMode::Windowed }
    }
}

/// Request the platform to resize the window to the given size.
/// Picked up by the backend on the next event-pump tick. No-op on platforms that don't support app-requested resizing (e.g. iOS, Android, UWP, Horizon).
pub fn set_window_size(ctx: &mut Context, size: (u32, u32)) -> GameResult {
    ctx.pending_window_resize = Some(size);
    Ok(())
}

#[allow(unreachable_code)]
pub fn init_backend(headless: bool, window_params: WindowParams) -> GameResult<Box<dyn Backend>> {
    if headless {
        return super::backend_null::NullBackend::new();
    }

    #[cfg(all(feature = "backend-horizon"))]
    {
        return super::backend_horizon::HorizonBackend::new();
    }

    #[cfg(all(feature = "backend-glutin"))]
    {
        return super::backend_glutin::GlutinBackend::new(window_params);
    }

    #[cfg(feature = "backend-sdl")]
    {
        return super::backend_sdl2::SDL2Backend::new(window_params);
    }

    log::warn!("No backend compiled in, using null backend instead.");
    super::backend_null::NullBackend::new()
}

