use core::mem;
use std::any::Any;
use std::cell::{RefCell, UnsafeCell};
use std::ffi::c_void;
use std::ops::Deref;
use std::ptr::null_mut;
use std::rc::Rc;
use std::time::Duration;

use imgui::internal::RawWrapper;
use imgui::{ConfigFlags, DrawCmd, DrawData, Key, MouseCursor, TextureId};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use sdl2::mouse::{Cursor, SystemCursor};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureCreator, TextureQuery, WindowCanvas};
use sdl2::video::GLProfile;
use sdl2::video::WindowContext;
use sdl2::{keyboard, pixels, EventPump, Sdl, VideoSubsystem};

use crate::common::{Color, Rect};
use crate::framework::backend::{
    Backend, BackendEventLoop, BackendRenderer, BackendShader, BackendTexture, SpriteBatchCommand, VertexData,
};
use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};
use crate::framework::graphics::BlendMode;
use crate::framework::keyboard::ScanCode;
use crate::framework::render_opengl::{GLContext, OpenGLRenderer};
use crate::framework::ui::init_imgui;
use crate::Game;
use crate::GameError::RenderError;
use crate::GAME_SUSPENDED;

pub struct SDL2Backend {
    context: Sdl,
    size_hint: (u16, u16),
}

impl SDL2Backend {
    pub fn new(size_hint: (u16, u16)) -> GameResult<Box<dyn Backend>> {
        let context = sdl2::init().map_err(|e| GameError::WindowError(e))?;

        let backend = SDL2Backend { context, size_hint };

        Ok(Box::new(backend))
    }
}

impl Backend for SDL2Backend {
    fn create_event_loop(&self) -> GameResult<Box<dyn BackendEventLoop>> {
        SDL2EventLoop::new(&self.context, self.size_hint)
    }
}

struct SDL2EventLoop {
    event_pump: EventPump,
    refs: Rc<RefCell<SDL2Context>>,
    opengl_available: RefCell<bool>,
}

struct SDL2Context {
    video: VideoSubsystem,
    canvas: WindowCanvas,
    texture_creator: TextureCreator<WindowContext>,
    gl_context: Option<sdl2::video::GLContext>,
    blend_mode: sdl2::render::BlendMode,
}

impl SDL2EventLoop {
    pub fn new(sdl: &Sdl, size_hint: (u16, u16)) -> GameResult<Box<dyn BackendEventLoop>> {
        let event_pump = sdl.event_pump().map_err(|e| GameError::WindowError(e))?;
        let video = sdl.video().map_err(|e| GameError::WindowError(e))?;
        let gl_attr = video.gl_attr();

        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 0);

        let mut window = video.window("Cave Story (doukutsu-rs)", size_hint.0 as _, size_hint.1 as _);
        window.position_centered();
        window.resizable();

        #[cfg(feature = "render-opengl")]
        window.opengl();

        let window = window.build().map_err(|e| GameError::WindowError(e.to_string()))?;

        let canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .map_err(|e| GameError::RenderError(e.to_string()))?;

        let texture_creator = canvas.texture_creator();

        let opengl_available = if let Ok(v) = std::env::var("CAVESTORY_NO_OPENGL") { v != "1" } else { true };
        let event_loop = SDL2EventLoop {
            event_pump,
            refs: Rc::new(RefCell::new(SDL2Context {
                video,
                canvas,
                texture_creator,
                gl_context: None,
                blend_mode: sdl2::render::BlendMode::Blend,
            })),
            opengl_available: RefCell::new(opengl_available),
        };

        Ok(Box::new(event_loop))
    }
}

impl BackendEventLoop for SDL2EventLoop {
    fn run(&mut self, game: &mut Game, ctx: &mut Context) {
        let state = unsafe { &mut *game.state.get() };

        let (imgui, imgui_sdl2) = unsafe {
            let renderer: &Box<SDL2Renderer> = std::mem::transmute(ctx.renderer.as_ref().unwrap());

            (&mut *renderer.imgui.as_ptr(), &mut *renderer.imgui_event.as_ptr())
        };

        {
            let (width, height) = self.refs.deref().borrow().canvas.window().size();
            ctx.screen_size = (width.max(1) as f32, height.max(1) as f32);

            imgui.io_mut().display_size = [ctx.screen_size.0, ctx.screen_size.1];
            let _ = state.handle_resize(ctx);
        }

        loop {
            #[cfg(target_os = "macos")]
            unsafe {
                use objc::*;

                // no UB: fields are initialized by SDL_GetWindowWMInfo
                let mut winfo: sdl2_sys::SDL_SysWMinfo = mem::MaybeUninit::uninit().assume_init();
                winfo.version.major = sdl2_sys::SDL_MAJOR_VERSION as _;
                winfo.version.minor = sdl2_sys::SDL_MINOR_VERSION as _;
                winfo.version.patch = sdl2_sys::SDL_PATCHLEVEL as _;

                let mut whandle = self.refs.deref().borrow().canvas.window().raw();

                if sdl2_sys::SDL_GetWindowWMInfo(whandle, &mut winfo as *mut _) != sdl2_sys::SDL_bool::SDL_FALSE {
                    let window = winfo.info.x11.display as *mut objc::runtime::Object;
                    let _: () = msg_send![window, setTitlebarAppearsTransparent:1];
                    let _: () = msg_send![window, setTitleVisibility:1]; // NSWindowTitleHidden
                    let mut style_mask: u32 = msg_send![window, styleMask];

                    style_mask |= 1 << 15; // NSWindowStyleMaskFullSizeContentView

                    let _: () = msg_send![window, setStyleMask: style_mask];
                }
            }

            for event in self.event_pump.poll_iter() {
                imgui_sdl2.handle_event(imgui, &event);

                match event {
                    Event::Quit { .. } => {
                        state.shutdown();
                    }
                    Event::Window { win_event, .. } => match win_event {
                        WindowEvent::FocusGained | WindowEvent::Shown => {
                            {
                                let mut mutex = GAME_SUSPENDED.lock().unwrap();
                                *mutex = false;
                            }

                            game.loops = 0;
                        }
                        WindowEvent::FocusLost | WindowEvent::Hidden => {
                            let mut mutex = GAME_SUSPENDED.lock().unwrap();
                            *mutex = true;
                        }
                        WindowEvent::SizeChanged(width, height) => {
                            ctx.screen_size = (width.max(1) as f32, height.max(1) as f32);

                            if let Some(renderer) = ctx.renderer.as_ref() {
                                if let Ok(imgui) = renderer.imgui() {
                                    imgui.io_mut().display_size = [ctx.screen_size.0, ctx.screen_size.1];
                                }
                            }
                            state.handle_resize(ctx).unwrap();
                        }
                        _ => {}
                    },
                    Event::KeyDown { scancode, repeat, .. } => {
                        if let Some(scancode) = scancode {
                            if let Some(drs_scan) = conv_scancode(scancode) {
                                if !repeat {
                                    state.process_debug_keys(drs_scan);
                                }
                                ctx.keyboard_context.set_key(drs_scan, true);
                            }
                        }
                    }
                    Event::KeyUp { scancode, .. } => {
                        if let Some(scancode) = scancode {
                            if let Some(drs_scan) = conv_scancode(scancode) {
                                ctx.keyboard_context.set_key(drs_scan, false);
                            }
                        }
                    }
                    _ => {}
                }
            }

            if state.shutdown {
                log::info!("Shutting down...");
                break;
            }

            {
                let mutex = GAME_SUSPENDED.lock().unwrap();
                if *mutex {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
            }

            game.update(ctx).unwrap();

            if let Some(_) = &state.next_scene {
                game.scene = mem::take(&mut state.next_scene);
                game.scene.as_mut().unwrap().init(state, ctx).unwrap();
                game.loops = 0;
                state.frame_time = 0.0;
            }

            imgui_sdl2.prepare_frame(
                imgui.io_mut(),
                self.refs.deref().borrow().canvas.window(),
                &self.event_pump.mouse_state(),
            );

            game.draw(ctx).unwrap();
        }
    }

    fn new_renderer(&self) -> GameResult<Box<dyn BackendRenderer>> {
        #[cfg(feature = "render-opengl")]
        {
            let mut refs = self.refs.borrow_mut();
            match refs.canvas.window().gl_create_context() {
                Ok(gl_ctx) => {
                    refs.canvas.window().gl_make_current(&gl_ctx).map_err(|e| GameError::RenderError(e.to_string()))?;
                    refs.gl_context = Some(gl_ctx);
                }
                Err(err) => {
                    *self.opengl_available.borrow_mut() = false;
                    log::error!("Failed to initialize OpenGL context, falling back to SDL2 renderer: {}", err);
                }
            }
        }

        #[cfg(feature = "render-opengl")]
        if *self.opengl_available.borrow() {
            let imgui = init_imgui()?;

            let refs = self.refs.clone();

            let user_data = Rc::into_raw(refs) as *mut c_void;

            unsafe fn get_proc_address(user_data: &mut *mut c_void, name: &str) -> *const c_void {
                let refs = Rc::from_raw(*user_data as *mut RefCell<SDL2Context>);

                let result = {
                    let refs = &mut *refs.as_ptr();
                    refs.video.gl_get_proc_address(name) as *const _
                };

                // log::info!("gl proc {} -> {:?}", name, result);
                *user_data = Rc::into_raw(refs) as *mut c_void;

                result
            }

            unsafe fn swap_buffers(user_data: &mut *mut c_void) {
                let refs = Rc::from_raw(*user_data as *mut RefCell<SDL2Context>);

                {
                    let refs = &mut *refs.as_ptr();

                    refs.canvas.window().gl_swap_window();
                }

                *user_data = Rc::into_raw(refs) as *mut c_void;
            }

            let gl_context = GLContext { gles2_mode: false, get_proc_address, swap_buffers, user_data };

            return Ok(Box::new(OpenGLRenderer::new(gl_context, UnsafeCell::new(imgui))));
        }

        SDL2Renderer::new(self.refs.clone())
    }
}

struct SDL2Renderer {
    refs: Rc<RefCell<SDL2Context>>,
    imgui: Rc<RefCell<imgui::Context>>,
    imgui_event: Rc<RefCell<ImguiSdl2>>,
    #[allow(unused)] // the rendering pipeline uses pointers to SDL_Texture, and we manually manage the lifetimes
    imgui_font_tex: SDL2Texture,
}

impl SDL2Renderer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(refs: Rc<RefCell<SDL2Context>>) -> GameResult<Box<dyn BackendRenderer>> {
        let mut imgui = init_imgui()?;

        imgui.set_renderer_name("SDL2Renderer".to_owned());
        let imgui_font_tex = {
            let refs = refs.clone();
            let mut fonts = imgui.fonts();
            let font_tex = fonts.build_rgba32_texture();

            let mut texture = refs
                .borrow_mut()
                .texture_creator
                .create_texture_streaming(PixelFormatEnum::RGBA32, font_tex.width, font_tex.height)
                .map_err(|e| GameError::RenderError(e.to_string()))?;

            texture.set_blend_mode(sdl2::render::BlendMode::Blend);
            texture
                .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                    for y in 0..(font_tex.height as usize) {
                        for x in 0..(font_tex.width as usize) {
                            let offset = y * pitch + x * 4;
                            let data_offset = (y * font_tex.width as usize + x) * 4;

                            buffer[offset] = font_tex.data[data_offset];
                            buffer[offset + 1] = font_tex.data[data_offset + 1];
                            buffer[offset + 2] = font_tex.data[data_offset + 2];
                            buffer[offset + 3] = font_tex.data[data_offset + 3];
                        }
                    }
                })
                .map_err(|e| GameError::RenderError(e.to_string()))?;

            SDL2Texture {
                refs: refs.clone(),
                texture: Some(texture),
                width: font_tex.width as u16,
                height: font_tex.height as u16,
                commands: vec![],
            }
        };
        imgui.fonts().tex_id = TextureId::new(imgui_font_tex.texture.as_ref().unwrap().raw() as usize);

        let imgui_sdl2 = unsafe {
            let refs = &mut *refs.as_ptr();
            ImguiSdl2::new(&mut imgui, refs.canvas.window())
        };

        Ok(Box::new(SDL2Renderer {
            refs,
            imgui: Rc::new(RefCell::new(imgui)),
            imgui_event: Rc::new(RefCell::new(imgui_sdl2)),
            imgui_font_tex,
        }))
    }
}

fn to_sdl(color: Color) -> pixels::Color {
    let (r, g, b, a) = color.to_rgba();
    pixels::Color::RGBA(r, g, b, a)
}

unsafe fn set_raw_target(
    renderer: *mut sdl2::sys::SDL_Renderer,
    raw_texture: *mut sdl2::sys::SDL_Texture,
) -> GameResult {
    if sdl2::sys::SDL_SetRenderTarget(renderer, raw_texture) == 0 {
        Ok(())
    } else {
        Err(GameError::RenderError(sdl2::get_error()))
    }
}

fn min3(x: f32, y: f32, z: f32) -> f32 {
    if x < y && x < z {
        x
    } else if y < z {
        y
    } else {
        z
    }
}

fn max3(x: f32, y: f32, z: f32) -> f32 {
    if x > y && x > z {
        x
    } else if y > z {
        y
    } else {
        z
    }
}

impl BackendRenderer for SDL2Renderer {
    fn renderer_name(&self) -> String {
        "SDL2_Renderer (fallback)".to_owned()
    }

    fn clear(&mut self, color: Color) {
        let mut refs = self.refs.borrow_mut();

        refs.canvas.set_draw_color(to_sdl(color));
        refs.canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        refs.canvas.clear();
    }

    fn present(&mut self) -> GameResult {
        let mut refs = self.refs.borrow_mut();

        refs.canvas.present();

        Ok(())
    }

    fn prepare_draw(&mut self, width: f32, height: f32) -> GameResult {
        let mut refs = self.refs.borrow_mut();

        refs.canvas.set_clip_rect(Some(sdl2::rect::Rect::new(0, 0, width as u32, height as u32)));

        Ok(())
    }

    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        let refs = self.refs.borrow_mut();

        let texture = refs
            .texture_creator
            .create_texture_target(PixelFormatEnum::RGBA32, width as u32, height as u32)
            .map_err(|e| GameError::RenderError(e.to_string()))?;

        Ok(Box::new(SDL2Texture { refs: self.refs.clone(), texture: Some(texture), width, height, commands: vec![] }))
    }

    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        let refs = self.refs.borrow_mut();

        let mut texture = refs
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::RGBA32, width as u32, height as u32)
            .map_err(|e| GameError::RenderError(e.to_string()))?;

        texture.set_blend_mode(sdl2::render::BlendMode::Blend);
        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..(height as usize) {
                    for x in 0..(width as usize) {
                        let offset = y * pitch + x * 4;
                        let data_offset = (y * width as usize + x) * 4;

                        buffer[offset] = data[data_offset];
                        buffer[offset + 1] = data[data_offset + 1];
                        buffer[offset + 2] = data[data_offset + 2];
                        buffer[offset + 3] = data[data_offset + 3];
                    }
                }
            })
            .map_err(|e| GameError::RenderError(e.to_string()))?;

        Ok(Box::new(SDL2Texture { refs: self.refs.clone(), texture: Some(texture), width, height, commands: vec![] }))
    }

    fn set_blend_mode(&mut self, blend: BlendMode) -> GameResult {
        let mut refs = self.refs.borrow_mut();

        refs.blend_mode = match blend {
            BlendMode::Add => sdl2::render::BlendMode::Add,
            BlendMode::Alpha => sdl2::render::BlendMode::Blend,
            BlendMode::Multiply => sdl2::render::BlendMode::Mod,
        };

        Ok(())
    }

    fn set_render_target(&mut self, texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        let renderer = self.refs.deref().borrow().canvas.raw();

        match texture {
            Some(texture) => {
                let sdl2_texture = texture
                    .as_any()
                    .downcast_ref::<SDL2Texture>()
                    .ok_or(RenderError("This texture was not created by OpenGL backend.".to_string()))?;

                unsafe {
                    if let Some(target) = sdl2_texture.texture.as_ref() {
                        set_raw_target(renderer, target.raw())?;
                    } else {
                        set_raw_target(renderer, std::ptr::null_mut())?;
                    }
                }
            }
            None => unsafe {
                set_raw_target(renderer, std::ptr::null_mut())?;
            },
        }

        Ok(())
    }

    fn draw_rect(&mut self, rect: Rect<isize>, color: Color) -> GameResult<()> {
        let mut refs = self.refs.borrow_mut();

        let (r, g, b, a) = color.to_rgba();

        let blend = refs.blend_mode;
        refs.canvas.set_draw_color(pixels::Color::RGBA(r, g, b, a));
        refs.canvas.set_blend_mode(blend);
        refs.canvas
            .fill_rect(sdl2::rect::Rect::new(
                rect.left as i32,
                rect.top as i32,
                rect.width() as u32,
                rect.height() as u32,
            ))
            .map_err(|e| GameError::RenderError(e.to_string()))?;

        Ok(())
    }

    fn draw_outline_rect(&mut self, rect: Rect<isize>, line_width: usize, color: Color) -> GameResult<()> {
        let mut refs = self.refs.borrow_mut();

        let (r, g, b, a) = color.to_rgba();

        let blend = refs.blend_mode;
        refs.canvas.set_draw_color(pixels::Color::RGBA(r, g, b, a));
        refs.canvas.set_blend_mode(blend);

        match line_width {
            0 => {} // no-op
            1 => {
                refs.canvas
                    .draw_rect(sdl2::rect::Rect::new(
                        rect.left as i32,
                        rect.top as i32,
                        rect.width() as u32,
                        rect.height() as u32,
                    ))
                    .map_err(|e| GameError::RenderError(e.to_string()))?;
            }
            _ => {
                let rects = [
                    sdl2::rect::Rect::new(rect.left as i32, rect.top as i32, rect.width() as u32, line_width as u32),
                    sdl2::rect::Rect::new(
                        rect.left as i32,
                        rect.bottom as i32 - line_width as i32,
                        rect.width() as u32,
                        line_width as u32,
                    ),
                    sdl2::rect::Rect::new(rect.left as i32, rect.top as i32, line_width as u32, rect.height() as u32),
                    sdl2::rect::Rect::new(
                        rect.right as i32 - line_width as i32,
                        rect.top as i32,
                        line_width as u32,
                        rect.height() as u32,
                    ),
                ];

                refs.canvas.fill_rects(&rects).map_err(|e| GameError::RenderError(e.to_string()))?;
            }
        }

        Ok(())
    }

    fn set_clip_rect(&mut self, rect: Option<Rect>) -> GameResult {
        let mut refs = self.refs.borrow_mut();

        if let Some(rect) = &rect {
            refs.canvas.set_clip_rect(Some(sdl2::rect::Rect::new(
                rect.left as i32,
                rect.top as i32,
                rect.width() as u32,
                rect.height() as u32,
            )));
        } else {
            refs.canvas.set_clip_rect(None);
        }

        Ok(())
    }

    fn imgui(&self) -> GameResult<&mut imgui::Context> {
        unsafe { Ok(&mut *self.imgui.as_ptr()) }
    }

    fn imgui_texture_id(&self, texture: &Box<dyn BackendTexture>) -> GameResult<TextureId> {
        let sdl_texture = texture
            .as_any()
            .downcast_ref::<SDL2Texture>()
            .ok_or(GameError::RenderError("This texture was not created by SDL backend.".to_string()))?;

        Ok(TextureId::new(sdl_texture.texture.as_ref().map(|t| t.raw()).unwrap_or(null_mut()) as usize))
    }

    fn render_imgui(&mut self, draw_data: &DrawData) -> GameResult {
        let mut refs = self.refs.borrow_mut();

        for draw_list in draw_data.draw_lists() {
            for cmd in draw_list.commands() {
                match cmd {
                    DrawCmd::Elements { count, cmd_params } => {
                        refs.canvas.set_clip_rect(Some(sdl2::rect::Rect::new(
                            cmd_params.clip_rect[0] as i32,
                            cmd_params.clip_rect[1] as i32,
                            (cmd_params.clip_rect[2] - cmd_params.clip_rect[0]) as u32,
                            (cmd_params.clip_rect[3] - cmd_params.clip_rect[1]) as u32,
                        )));

                        let idx_buffer = draw_list.idx_buffer();
                        let mut vert_x = [0i16; 6];
                        let mut vert_y = [0i16; 6];
                        let mut min = [0f32; 2];
                        let mut max = [0f32; 2];
                        let mut tex_pos = [0f32; 4];
                        let mut is_rect = false;

                        for i in (0..count).step_by(3) {
                            if is_rect {
                                is_rect = false;
                                continue;
                            }

                            let v1 = draw_list.vtx_buffer()
                                [cmd_params.vtx_offset + idx_buffer[cmd_params.idx_offset + i] as usize];
                            let v2 = draw_list.vtx_buffer()
                                [cmd_params.vtx_offset + idx_buffer[cmd_params.idx_offset + i + 1] as usize];
                            let v3 = draw_list.vtx_buffer()
                                [cmd_params.vtx_offset + idx_buffer[cmd_params.idx_offset + i + 2] as usize];

                            vert_x[0] = (v1.pos[0] - 0.5) as i16;
                            vert_y[0] = (v1.pos[1] - 0.5) as i16;
                            vert_x[1] = (v2.pos[0] - 0.5) as i16;
                            vert_y[1] = (v2.pos[1] - 0.5) as i16;
                            vert_x[2] = (v3.pos[0] - 0.5) as i16;
                            vert_y[2] = (v3.pos[1] - 0.5) as i16;

                            #[allow(clippy::float_cmp)]
                            if i < count - 3 {
                                let v4 = draw_list.vtx_buffer()
                                    [cmd_params.vtx_offset + idx_buffer[cmd_params.idx_offset + i + 3] as usize];
                                let v5 = draw_list.vtx_buffer()
                                    [cmd_params.vtx_offset + idx_buffer[cmd_params.idx_offset + i + 4] as usize];
                                let v6 = draw_list.vtx_buffer()
                                    [cmd_params.vtx_offset + idx_buffer[cmd_params.idx_offset + i + 5] as usize];

                                min[0] = min3(v1.pos[0], v2.pos[0], v3.pos[0]);
                                min[1] = min3(v1.pos[1], v2.pos[1], v3.pos[1]);
                                max[0] = max3(v1.pos[0], v2.pos[0], v3.pos[0]);
                                max[1] = max3(v1.pos[1], v2.pos[1], v3.pos[1]);

                                is_rect = (v1.pos[0] == min[0] || v1.pos[0] == max[0])
                                    && (v1.pos[1] == min[1] || v1.pos[1] == max[1])
                                    && (v2.pos[0] == min[0] || v2.pos[0] == max[0])
                                    && (v2.pos[1] == min[1] || v2.pos[1] == max[1])
                                    && (v3.pos[0] == min[0] || v3.pos[0] == max[0])
                                    && (v3.pos[1] == min[1] || v3.pos[1] == max[1])
                                    && (v4.pos[0] == min[0] || v4.pos[0] == max[0])
                                    && (v4.pos[1] == min[1] || v4.pos[1] == max[1])
                                    && (v5.pos[0] == min[0] || v5.pos[0] == max[0])
                                    && (v5.pos[1] == min[1] || v5.pos[1] == max[1])
                                    && (v6.pos[0] == min[0] || v6.pos[0] == max[0])
                                    && (v6.pos[1] == min[1] || v6.pos[1] == max[1]);

                                if is_rect {
                                    tex_pos[0] = min3(v1.uv[0], v2.uv[0], v3.uv[0]);
                                    tex_pos[1] = min3(v1.uv[1], v2.uv[1], v3.uv[1]);
                                    tex_pos[2] = max3(v1.uv[0], v2.uv[0], v3.uv[0]);
                                    tex_pos[3] = max3(v1.uv[1], v2.uv[1], v3.uv[1]);
                                }
                            }

                            let ptr = cmd_params.texture_id.id() as *mut sdl2::sys::SDL_Texture;
                            if !ptr.is_null() {
                                let mut surf = unsafe { refs.texture_creator.raw_create_texture(ptr) };
                                let TextureQuery { width, height, .. } = surf.query();

                                if is_rect {
                                    let src = sdl2::rect::Rect::new(
                                        (tex_pos[0] * width as f32) as i32,
                                        (tex_pos[1] * height as f32) as i32,
                                        ((tex_pos[2] - tex_pos[0]) * width as f32) as u32,
                                        ((tex_pos[3] - tex_pos[1]) * height as f32) as u32,
                                    );
                                    let dest = sdl2::rect::Rect::new(
                                        min[0] as i32,
                                        min[1] as i32,
                                        (max[0] - min[0]) as u32,
                                        (max[1] - min[1]) as u32,
                                    );

                                    surf.set_color_mod(v1.col[0], v1.col[1], v1.col[2]);
                                    surf.set_alpha_mod(v1.col[3]);

                                    refs.canvas
                                        .copy(&surf, src, dest)
                                        .map_err(|e| GameError::RenderError(e.to_string()))?;
                                } else {
                                    /*sdl2::sys::gfx::primitives::filledPolygonRGBA(
                                        refs.canvas.raw(),
                                        vert_x.as_ptr(),
                                        vert_y.as_ptr(),
                                        3,
                                        v1.col[0],
                                        v1.col[1],
                                        v1.col[2],
                                        v1.col[3],
                                    );*/
                                }
                            }
                        }

                        refs.canvas.set_clip_rect(None);
                    }
                    DrawCmd::ResetRenderState => {}
                    DrawCmd::RawCallback { callback, raw_cmd } => unsafe { callback(draw_list.raw(), raw_cmd) },
                }
            }
        }

        Ok(())
    }

    fn draw_triangle_list(
        &mut self,
        _vertices: Vec<VertexData>,
        _texture: Option<&Box<dyn BackendTexture>>,
        _shader: BackendShader,
    ) -> GameResult<()> {
        Err(GameError::RenderError("Unsupported operation".to_string()))
    }
}

struct SDL2Texture {
    refs: Rc<RefCell<SDL2Context>>,
    texture: Option<Texture>,
    width: u16,
    height: u16,
    commands: Vec<SpriteBatchCommand>,
}

impl BackendTexture for SDL2Texture {
    fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn add(&mut self, command: SpriteBatchCommand) {
        self.commands.push(command);
    }

    fn clear(&mut self) {
        self.commands.clear();
    }

    fn draw(&mut self) -> GameResult {
        match self.texture.as_mut() {
            None => Ok(()),
            Some(texture) => {
                let mut refs = self.refs.borrow_mut();
                for command in self.commands.iter() {
                    match command {
                        SpriteBatchCommand::DrawRect(src, dest) => {
                            texture.set_color_mod(255, 255, 255);
                            texture.set_alpha_mod(255);
                            texture.set_blend_mode(refs.blend_mode);

                            refs.canvas
                                .copy(
                                    texture,
                                    Some(sdl2::rect::Rect::new(
                                        src.left.round() as i32,
                                        src.top.round() as i32,
                                        src.width().round() as u32,
                                        src.height().round() as u32,
                                    )),
                                    Some(sdl2::rect::Rect::new(
                                        dest.left.round() as i32,
                                        dest.top.round() as i32,
                                        dest.width().round() as u32,
                                        dest.height().round() as u32,
                                    )),
                                )
                                .map_err(|e| GameError::RenderError(e.to_string()))?;
                        }
                        SpriteBatchCommand::DrawRectTinted(src, dest, color) => {
                            let (r, g, b, a) = color.to_rgba();
                            texture.set_color_mod(r, g, b);
                            texture.set_alpha_mod(a);
                            texture.set_blend_mode(refs.blend_mode);

                            refs.canvas
                                .copy(
                                    texture,
                                    Some(sdl2::rect::Rect::new(
                                        src.left.round() as i32,
                                        src.top.round() as i32,
                                        src.width().round() as u32,
                                        src.height().round() as u32,
                                    )),
                                    Some(sdl2::rect::Rect::new(
                                        dest.left.round() as i32,
                                        dest.top.round() as i32,
                                        dest.width().round() as u32,
                                        dest.height().round() as u32,
                                    )),
                                )
                                .map_err(|e| GameError::RenderError(e.to_string()))?;
                        }
                        SpriteBatchCommand::DrawRectFlip(src, dest, flip_x, flip_y) => {
                            texture.set_color_mod(255, 255, 255);
                            texture.set_alpha_mod(255);
                            texture.set_blend_mode(refs.blend_mode);

                            refs.canvas
                                .copy_ex(
                                    texture,
                                    Some(sdl2::rect::Rect::new(
                                        src.left.round() as i32,
                                        src.top.round() as i32,
                                        src.width().round() as u32,
                                        src.height().round() as u32,
                                    )),
                                    Some(sdl2::rect::Rect::new(
                                        dest.left.round() as i32,
                                        dest.top.round() as i32,
                                        dest.width().round() as u32,
                                        dest.height().round() as u32,
                                    )),
                                    0.0,
                                    None,
                                    *flip_x,
                                    *flip_y,
                                )
                                .map_err(|e| GameError::RenderError(e.to_string()))?;
                        }
                    }
                }

                Ok(())
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for SDL2Texture {
    fn drop(&mut self) {
        let mut texture_opt = None;
        mem::swap(&mut self.texture, &mut texture_opt);

        if let Some(texture) = texture_opt {
            unsafe {
                texture.destroy();
            }
        }
    }
}

fn conv_scancode(code: keyboard::Scancode) -> Option<ScanCode> {
    match code {
        Scancode::A => Some(ScanCode::A),
        Scancode::B => Some(ScanCode::B),
        Scancode::C => Some(ScanCode::C),
        Scancode::D => Some(ScanCode::D),
        Scancode::E => Some(ScanCode::E),
        Scancode::F => Some(ScanCode::F),
        Scancode::G => Some(ScanCode::G),
        Scancode::H => Some(ScanCode::H),
        Scancode::I => Some(ScanCode::I),
        Scancode::J => Some(ScanCode::J),
        Scancode::K => Some(ScanCode::K),
        Scancode::L => Some(ScanCode::L),
        Scancode::M => Some(ScanCode::M),
        Scancode::N => Some(ScanCode::N),
        Scancode::O => Some(ScanCode::O),
        Scancode::P => Some(ScanCode::P),
        Scancode::Q => Some(ScanCode::Q),
        Scancode::R => Some(ScanCode::R),
        Scancode::S => Some(ScanCode::S),
        Scancode::T => Some(ScanCode::T),
        Scancode::U => Some(ScanCode::U),
        Scancode::V => Some(ScanCode::V),
        Scancode::W => Some(ScanCode::W),
        Scancode::X => Some(ScanCode::X),
        Scancode::Y => Some(ScanCode::Y),
        Scancode::Z => Some(ScanCode::Z),
        Scancode::Num1 => Some(ScanCode::Key1),
        Scancode::Num2 => Some(ScanCode::Key2),
        Scancode::Num3 => Some(ScanCode::Key3),
        Scancode::Num4 => Some(ScanCode::Key4),
        Scancode::Num5 => Some(ScanCode::Key5),
        Scancode::Num6 => Some(ScanCode::Key6),
        Scancode::Num7 => Some(ScanCode::Key7),
        Scancode::Num8 => Some(ScanCode::Key8),
        Scancode::Num9 => Some(ScanCode::Key9),
        Scancode::Num0 => Some(ScanCode::Key0),
        Scancode::Return => Some(ScanCode::Return),
        Scancode::Escape => Some(ScanCode::Escape),
        Scancode::Backspace => Some(ScanCode::Backspace),
        Scancode::Tab => Some(ScanCode::Tab),
        Scancode::Space => Some(ScanCode::Space),
        Scancode::Minus => Some(ScanCode::Minus),
        Scancode::Equals => Some(ScanCode::Equals),
        Scancode::LeftBracket => Some(ScanCode::LBracket),
        Scancode::RightBracket => Some(ScanCode::RBracket),
        Scancode::Backslash => Some(ScanCode::Backslash),
        Scancode::NonUsHash => Some(ScanCode::NonUsHash),
        Scancode::Semicolon => Some(ScanCode::Semicolon),
        Scancode::Apostrophe => Some(ScanCode::Apostrophe),
        Scancode::Grave => Some(ScanCode::Grave),
        Scancode::Comma => Some(ScanCode::Comma),
        Scancode::Period => Some(ScanCode::Period),
        Scancode::Slash => Some(ScanCode::Slash),
        Scancode::CapsLock => Some(ScanCode::Capslock),
        Scancode::F1 => Some(ScanCode::F1),
        Scancode::F2 => Some(ScanCode::F2),
        Scancode::F3 => Some(ScanCode::F3),
        Scancode::F4 => Some(ScanCode::F4),
        Scancode::F5 => Some(ScanCode::F5),
        Scancode::F6 => Some(ScanCode::F6),
        Scancode::F7 => Some(ScanCode::F7),
        Scancode::F8 => Some(ScanCode::F8),
        Scancode::F9 => Some(ScanCode::F9),
        Scancode::F10 => Some(ScanCode::F10),
        Scancode::F11 => Some(ScanCode::F11),
        Scancode::F12 => Some(ScanCode::F12),
        Scancode::PrintScreen => Some(ScanCode::Sysrq),
        Scancode::ScrollLock => Some(ScanCode::Scrolllock),
        Scancode::Pause => Some(ScanCode::Pause),
        Scancode::Insert => Some(ScanCode::Insert),
        Scancode::Home => Some(ScanCode::Home),
        Scancode::PageUp => Some(ScanCode::PageUp),
        Scancode::Delete => Some(ScanCode::Delete),
        Scancode::End => Some(ScanCode::End),
        Scancode::PageDown => Some(ScanCode::PageDown),
        Scancode::Right => Some(ScanCode::Right),
        Scancode::Left => Some(ScanCode::Left),
        Scancode::Down => Some(ScanCode::Down),
        Scancode::Up => Some(ScanCode::Up),
        Scancode::NumLockClear => Some(ScanCode::Numlock),
        Scancode::KpDivide => Some(ScanCode::NumpadDivide),
        Scancode::KpMultiply => Some(ScanCode::NumpadMultiply),
        Scancode::KpMinus => Some(ScanCode::NumpadSubtract),
        Scancode::KpPlus => Some(ScanCode::NumpadAdd),
        Scancode::KpEnter => Some(ScanCode::NumpadEnter),
        Scancode::Kp1 => Some(ScanCode::Numpad1),
        Scancode::Kp2 => Some(ScanCode::Numpad2),
        Scancode::Kp3 => Some(ScanCode::Numpad3),
        Scancode::Kp4 => Some(ScanCode::Numpad4),
        Scancode::Kp5 => Some(ScanCode::Numpad5),
        Scancode::Kp6 => Some(ScanCode::Numpad6),
        Scancode::Kp7 => Some(ScanCode::Numpad7),
        Scancode::Kp8 => Some(ScanCode::Numpad8),
        Scancode::Kp9 => Some(ScanCode::Numpad9),
        Scancode::Kp0 => Some(ScanCode::Numpad0),
        Scancode::NonUsBackslash => Some(ScanCode::NonUsBackslash),
        Scancode::Application => Some(ScanCode::Apps),
        Scancode::Power => Some(ScanCode::Power),
        Scancode::KpEquals => Some(ScanCode::NumpadEquals),
        Scancode::F13 => Some(ScanCode::F13),
        Scancode::F14 => Some(ScanCode::F14),
        Scancode::F15 => Some(ScanCode::F15),
        Scancode::F16 => Some(ScanCode::F16),
        Scancode::F17 => Some(ScanCode::F17),
        Scancode::F18 => Some(ScanCode::F18),
        Scancode::F19 => Some(ScanCode::F19),
        Scancode::F20 => Some(ScanCode::F20),
        Scancode::F21 => Some(ScanCode::F21),
        Scancode::F22 => Some(ScanCode::F22),
        Scancode::F23 => Some(ScanCode::F23),
        Scancode::F24 => Some(ScanCode::F24),
        Scancode::Stop => Some(ScanCode::Stop),
        Scancode::Cut => Some(ScanCode::Cut),
        Scancode::Copy => Some(ScanCode::Copy),
        Scancode::Paste => Some(ScanCode::Paste),
        Scancode::Mute => Some(ScanCode::Mute),
        Scancode::VolumeUp => Some(ScanCode::VolumeUp),
        Scancode::VolumeDown => Some(ScanCode::VolumeDown),
        Scancode::KpComma => Some(ScanCode::NumpadComma),
        Scancode::SysReq => Some(ScanCode::Sysrq),
        Scancode::Return2 => Some(ScanCode::NumpadEnter),
        Scancode::LCtrl => Some(ScanCode::LControl),
        Scancode::LShift => Some(ScanCode::LShift),
        Scancode::LAlt => Some(ScanCode::LAlt),
        Scancode::LGui => Some(ScanCode::LWin),
        Scancode::RCtrl => Some(ScanCode::RControl),
        Scancode::RShift => Some(ScanCode::RShift),
        Scancode::RAlt => Some(ScanCode::RAlt),
        Scancode::RGui => Some(ScanCode::RWin),
        Scancode::AudioNext => Some(ScanCode::NextTrack),
        Scancode::AudioPrev => Some(ScanCode::PrevTrack),
        Scancode::AudioStop => Some(ScanCode::MediaStop),
        Scancode::AudioPlay => Some(ScanCode::PlayPause),
        Scancode::AudioMute => Some(ScanCode::Mute),
        Scancode::MediaSelect => Some(ScanCode::MediaSelect),
        Scancode::Mail => Some(ScanCode::Mail),
        Scancode::Calculator => Some(ScanCode::Calculator),
        Scancode::Sleep => Some(ScanCode::Sleep),
        _ => None,
    }
}

// based on imgui-sdl2 crate
pub struct ImguiSdl2 {
    mouse_press: [bool; 5],
    ignore_mouse: bool,
    ignore_keyboard: bool,
    cursor: Option<MouseCursor>,
    sdl_cursor: Option<Cursor>,
}

struct Sdl2ClipboardBackend(sdl2::clipboard::ClipboardUtil);

impl imgui::ClipboardBackend for Sdl2ClipboardBackend {
    fn get(&mut self) -> Option<String> {
        if !self.0.has_clipboard_text() {
            return None;
        }

        self.0.clipboard_text().ok()
    }

    fn set(&mut self, value: &str) {
        let _ = self.0.set_clipboard_text(value);
    }
}

impl ImguiSdl2 {
    pub fn new(imgui: &mut imgui::Context, window: &sdl2::video::Window) -> Self {
        let clipboard_util = window.subsystem().clipboard();
        imgui.set_clipboard_backend(Sdl2ClipboardBackend(clipboard_util));

        imgui.io_mut().key_map[Key::Tab as usize] = Scancode::Tab as u32;
        imgui.io_mut().key_map[Key::LeftArrow as usize] = Scancode::Left as u32;
        imgui.io_mut().key_map[Key::RightArrow as usize] = Scancode::Right as u32;
        imgui.io_mut().key_map[Key::UpArrow as usize] = Scancode::Up as u32;
        imgui.io_mut().key_map[Key::DownArrow as usize] = Scancode::Down as u32;
        imgui.io_mut().key_map[Key::PageUp as usize] = Scancode::PageUp as u32;
        imgui.io_mut().key_map[Key::PageDown as usize] = Scancode::PageDown as u32;
        imgui.io_mut().key_map[Key::Home as usize] = Scancode::Home as u32;
        imgui.io_mut().key_map[Key::End as usize] = Scancode::End as u32;
        imgui.io_mut().key_map[Key::Delete as usize] = Scancode::Delete as u32;
        imgui.io_mut().key_map[Key::Backspace as usize] = Scancode::Backspace as u32;
        imgui.io_mut().key_map[Key::Enter as usize] = Scancode::Return as u32;
        imgui.io_mut().key_map[Key::Escape as usize] = Scancode::Escape as u32;
        imgui.io_mut().key_map[Key::Space as usize] = Scancode::Space as u32;
        imgui.io_mut().key_map[Key::A as usize] = Scancode::A as u32;
        imgui.io_mut().key_map[Key::C as usize] = Scancode::C as u32;
        imgui.io_mut().key_map[Key::V as usize] = Scancode::V as u32;
        imgui.io_mut().key_map[Key::X as usize] = Scancode::X as u32;
        imgui.io_mut().key_map[Key::Y as usize] = Scancode::Y as u32;
        imgui.io_mut().key_map[Key::Z as usize] = Scancode::Z as u32;

        Self { mouse_press: [false; 5], ignore_keyboard: false, ignore_mouse: false, cursor: None, sdl_cursor: None }
    }

    pub fn ignore_event(&self, event: &Event) -> bool {
        match *event {
            Event::KeyDown { .. } | Event::KeyUp { .. } | Event::TextEditing { .. } | Event::TextInput { .. } => {
                self.ignore_keyboard
            }
            Event::MouseMotion { .. }
            | Event::MouseButtonDown { .. }
            | Event::MouseButtonUp { .. }
            | Event::MouseWheel { .. }
            | Event::FingerDown { .. }
            | Event::FingerUp { .. }
            | Event::FingerMotion { .. }
            | Event::DollarGesture { .. }
            | Event::DollarRecord { .. }
            | Event::MultiGesture { .. } => self.ignore_mouse,
            _ => false,
        }
    }

    pub fn handle_event(&mut self, imgui: &mut imgui::Context, event: &Event) {
        use sdl2::mouse::MouseButton;

        fn set_mod(imgui: &mut imgui::Context, keymod: keyboard::Mod) {
            let ctrl = keymod.intersects(keyboard::Mod::RCTRLMOD | keyboard::Mod::LCTRLMOD);
            let alt = keymod.intersects(keyboard::Mod::RALTMOD | keyboard::Mod::LALTMOD);
            let shift = keymod.intersects(keyboard::Mod::RSHIFTMOD | keyboard::Mod::LSHIFTMOD);
            let super_ = keymod.intersects(keyboard::Mod::RGUIMOD | keyboard::Mod::LGUIMOD);

            imgui.io_mut().key_ctrl = ctrl;
            imgui.io_mut().key_alt = alt;
            imgui.io_mut().key_shift = shift;
            imgui.io_mut().key_super = super_;
        }

        match *event {
            Event::MouseWheel { y, .. } => {
                imgui.io_mut().mouse_wheel = y as f32;
            }
            Event::MouseButtonDown { mouse_btn, .. } => {
                if mouse_btn != MouseButton::Unknown {
                    let index = match mouse_btn {
                        MouseButton::Left => 0,
                        MouseButton::Right => 1,
                        MouseButton::Middle => 2,
                        MouseButton::X1 => 3,
                        MouseButton::X2 => 4,
                        MouseButton::Unknown => unreachable!(),
                    };
                    self.mouse_press[index] = true;
                }
            }
            Event::TextInput { ref text, .. } => {
                for chr in text.chars() {
                    imgui.io_mut().add_input_character(chr);
                }
            }
            Event::KeyDown { scancode, keymod, .. } => {
                set_mod(imgui, keymod);
                if let Some(scancode) = scancode {
                    imgui.io_mut().keys_down[scancode as usize] = true;
                }
            }
            Event::KeyUp { scancode, keymod, .. } => {
                set_mod(imgui, keymod);
                if let Some(scancode) = scancode {
                    imgui.io_mut().keys_down[scancode as usize] = false;
                }
            }
            _ => {}
        }
    }

    pub fn prepare_frame(
        &mut self,
        io: &mut imgui::Io,
        window: &sdl2::video::Window,
        mouse_state: &sdl2::mouse::MouseState,
    ) {
        let mouse_util = window.subsystem().sdl().mouse();

        let (win_w, win_h) = window.size();
        let (draw_w, draw_h) = window.drawable_size();

        io.display_size = [win_w as f32, win_h as f32];
        io.display_framebuffer_scale = [(draw_w as f32) / (win_w as f32), (draw_h as f32) / (win_h as f32)];

        // Merging the mousedown events we received into the current state prevents us from missing
        // clicks that happen faster than a frame
        io.mouse_down = [
            self.mouse_press[0] || mouse_state.left(),
            self.mouse_press[1] || mouse_state.right(),
            self.mouse_press[2] || mouse_state.middle(),
            self.mouse_press[3] || mouse_state.x1(),
            self.mouse_press[4] || mouse_state.x2(),
        ];
        self.mouse_press = [false; 5];

        let any_mouse_down = io.mouse_down.iter().any(|&b| b);
        mouse_util.capture(any_mouse_down);

        io.mouse_pos = [mouse_state.x() as f32, mouse_state.y() as f32];

        self.ignore_keyboard = io.want_capture_keyboard;
        self.ignore_mouse = io.want_capture_mouse;
    }

    pub fn prepare_render(&mut self, ui: &imgui::Ui, window: &sdl2::video::Window) {
        let io = ui.io();
        if !io.config_flags.contains(ConfigFlags::NO_MOUSE_CURSOR_CHANGE) {
            let mouse_util = window.subsystem().sdl().mouse();

            match ui.mouse_cursor() {
                Some(mouse_cursor) if !io.mouse_draw_cursor => {
                    mouse_util.show_cursor(true);

                    let sdl_cursor = match mouse_cursor {
                        MouseCursor::Arrow => SystemCursor::Arrow,
                        MouseCursor::TextInput => SystemCursor::IBeam,
                        MouseCursor::ResizeAll => SystemCursor::SizeAll,
                        MouseCursor::ResizeNS => SystemCursor::SizeNS,
                        MouseCursor::ResizeEW => SystemCursor::SizeWE,
                        MouseCursor::ResizeNESW => SystemCursor::SizeNESW,
                        MouseCursor::ResizeNWSE => SystemCursor::SizeNWSE,
                        MouseCursor::Hand => SystemCursor::Hand,
                        MouseCursor::NotAllowed => SystemCursor::No,
                    };

                    if self.cursor != Some(mouse_cursor) {
                        let sdl_cursor = Cursor::from_system(sdl_cursor).unwrap();
                        sdl_cursor.set();
                        self.cursor = Some(mouse_cursor);
                        self.sdl_cursor = Some(sdl_cursor);
                    }
                }
                _ => {
                    self.cursor = None;
                    self.sdl_cursor = None;
                    mouse_util.show_cursor(false);
                }
            }
        }
    }
}
