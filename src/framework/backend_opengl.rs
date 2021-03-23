use std::cell::{RefCell, UnsafeCell};
use std::ffi::CStr;
use std::mem;
use std::rc::Rc;
use std::sync::Arc;

use glutin::event::{Event, TouchPhase, WindowEvent, ElementState, VirtualKeyCode};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{Api, ContextBuilder, GlProfile, GlRequest, PossiblyCurrent, WindowedContext};
use imgui::{DrawCmd, DrawCmdParams, DrawData, DrawIdx, DrawVert};

use gl::types::*;

use crate::common::{Color, Rect};
use crate::framework::backend::{Backend, BackendEventLoop, BackendRenderer, BackendTexture, SpriteBatchCommand};
use crate::framework::context::Context;
use crate::framework::error::GameError::RenderError;
use crate::framework::error::GameResult;
use crate::framework::gl;
use crate::framework::graphics::BlendMode;
use crate::input::touch_controls::TouchPoint;
use crate::{Game, GAME_SUSPENDED};
use crate::framework::keyboard::ScanCode;

pub struct GlutinBackend;

impl GlutinBackend {
    pub fn new() -> GameResult<Box<dyn Backend>> {
        Ok(Box::new(GlutinBackend))
    }
}

impl Backend for GlutinBackend {
    fn create_event_loop(&self) -> GameResult<Box<dyn BackendEventLoop>> {
        #[cfg(target_os = "android")]
        loop {
            match ndk_glue::native_window().as_ref() {
                Some(_) => {
                    log::info!("NativeWindow Found: {:?}", ndk_glue::native_window());
                    break;
                }
                None => (),
            }
        }

        Ok(Box::new(GlutinEventLoop { refs: Rc::new(UnsafeCell::new(None)) }))
    }
}

pub struct GlutinEventLoop {
    refs: Rc<UnsafeCell<Option<WindowedContext<PossiblyCurrent>>>>,
}

impl GlutinEventLoop {
    fn get_context(&self, event_loop: &EventLoop<()>) -> &mut WindowedContext<PossiblyCurrent> {
        let mut refs = unsafe { &mut *self.refs.get() };

        if refs.is_none() {
            let mut window = WindowBuilder::new();
            let windowed_context = ContextBuilder::new()
                .with_gl(GlRequest::Specific(Api::OpenGlEs, (2, 0)))
                .with_gl_profile(GlProfile::Core)
                .with_gl_debug_flag(false)
                .with_pixel_format(24, 8)
                .with_vsync(true);

            #[cfg(target_os = "windows")]
                {
                    use glutin::platform::windows::WindowBuilderExtWindows;
                    window = window.with_drag_and_drop(false);
                }

            window = window.with_title("doukutsu-rs");

            let windowed_context = windowed_context.build_windowed(window, event_loop)
                .unwrap();

            let windowed_context = unsafe { windowed_context.make_current().unwrap() };

            #[cfg(target_os = "android")]
            if let Some(nwin) = ndk_glue::native_window().as_ref() {
                unsafe {
                    windowed_context.surface_created(nwin.ptr().as_ptr() as *mut std::ffi::c_void);
                }
            }

            refs.replace(windowed_context);
        }

        refs.as_mut().unwrap()
    }
}

#[cfg(target_os = "android")]
fn request_android_redraw() {
    match ndk_glue::native_window().as_ref() {
        Some(native_window) => {
            let a_native_window: *mut ndk_sys::ANativeWindow = native_window.ptr().as_ptr();
            let a_native_activity: *mut ndk_sys::ANativeActivity = ndk_glue::native_activity().ptr().as_ptr();
            unsafe {
                match (*(*a_native_activity).callbacks).onNativeWindowRedrawNeeded {
                    Some(callback) => callback(a_native_activity, a_native_window),
                    None => (),
                };
            };
        }
        None => (),
    }
}

#[cfg(target_os = "android")]
fn get_insets() -> GameResult<(f32, f32, f32, f32)> {
    unsafe {
        let vm_ptr = ndk_glue::native_activity().vm();
        let vm = unsafe { jni::JavaVM::from_raw(vm_ptr) }?;
        let vm_env = vm.attach_current_thread()?;

        //let class = vm_env.find_class("io/github/doukutsu_rs/MainActivity")?;
        let class = vm_env.new_global_ref(ndk_glue::native_activity().activity())?;
        let field = vm_env.get_field(class.as_obj(), "displayInsets", "[I")?.to_jni().l as jni::sys::jintArray;

        let mut elements = [0; 4];
        vm_env.get_int_array_region(field, 0, &mut elements)?;

        Ok((elements[0] as f32, elements[1] as f32, elements[2] as f32, elements[3] as f32))
    }
}

impl BackendEventLoop for GlutinEventLoop {
    fn run(&mut self, game: &mut Game, ctx: &mut Context) {
        let event_loop = EventLoop::new();
        let state_ref = unsafe { &mut *game.state.get() };
        let window: &'static mut WindowedContext<PossiblyCurrent> =
            unsafe { std::mem::transmute(self.get_context(&event_loop)) };

        {
            let size = window.window().inner_size();
            ctx.screen_size = (size.width.max(1) as f32, size.height.max(1) as f32);
            state_ref.handle_resize(ctx).unwrap();
        }

        // it won't ever return
        let (game, ctx): (&'static mut Game, &'static mut Context) =
            unsafe { (std::mem::transmute(game), std::mem::transmute(ctx)) };

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, window_id }
                    if window_id == window.window().id() =>
                {
                    state_ref.shutdown();
                }
                Event::Resumed => {
                    println!("resumed!");
                    {
                        let mut mutex = GAME_SUSPENDED.lock().unwrap();
                        *mutex = false;
                    }

                    #[cfg(target_os = "android")]
                    if let Some(nwin) = ndk_glue::native_window().as_ref() {
                        state_ref.graphics_reset();
                        unsafe {
                            window.surface_created(nwin.ptr().as_ptr() as *mut std::ffi::c_void);
                            request_android_redraw();
                        }
                    }
                }
                Event::Suspended => {
                    println!("suspended!");
                    {
                        let mut mutex = GAME_SUSPENDED.lock().unwrap();
                        *mutex = true;
                    }

                    #[cfg(target_os = "android")]
                    unsafe {
                        window.surface_destroyed();
                    }
                }
                Event::WindowEvent { event: WindowEvent::Resized(size), window_id }
                    if window_id == window.window().id() =>
                {
                    if let Some(renderer) = ctx.renderer.as_ref() {
                        if let Ok(imgui) = renderer.imgui() {
                            imgui.io_mut().display_size = [size.width as f32, size.height as f32];
                        }

                        ctx.screen_size = (size.width as f32, size.height as f32);
                        state_ref.handle_resize(ctx).unwrap();
                    }
                }
                Event::WindowEvent { event: WindowEvent::Touch(touch), window_id }
                    if window_id == window.window().id() =>
                {
                    let mut controls = &mut state_ref.touch_controls;
                    let scale = state_ref.scale as f64;

                    match touch.phase {
                        TouchPhase::Started | TouchPhase::Moved => {
                            if let Some(point) = controls.points.iter_mut().find(|p| p.id == touch.id) {
                                point.last_position = point.position;
                                point.position = (touch.location.x / scale, touch.location.y / scale);
                            } else {
                                controls.touch_id_counter = controls.touch_id_counter.wrapping_add(1);

                                let point = TouchPoint {
                                    id: touch.id,
                                    touch_id: controls.touch_id_counter,
                                    position: (touch.location.x / scale, touch.location.y / scale),
                                    last_position: (0.0, 0.0),
                                };
                                controls.points.push(point);

                                if touch.phase == TouchPhase::Started {
                                    controls.clicks.push(point);
                                }
                            }
                        }
                        TouchPhase::Ended | TouchPhase::Cancelled => {
                            controls.points.retain(|p| p.id != touch.id);
                            controls.clicks.retain(|p| p.id != touch.id);
                        }
                    }
                }
                Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, window_id }
                    if window_id == window.window().id() => {

                    if let Some(keycode) = input.virtual_keycode {
                        if let Some(drs_scan) = conv_keycode(keycode) {
                            let key_state = match input.state {
                                ElementState::Pressed => true,
                                ElementState::Released => false,
                            };

                            ctx.keyboard_context.set_key(drs_scan, key_state);
                        }
                    }
                }
                Event::RedrawRequested(id) if id == window.window().id() => {
                    {
                        let mutex = GAME_SUSPENDED.lock().unwrap();
                        if *mutex {
                            return;
                        }
                    }

                    #[cfg(not(target_os = "android"))]
                    {
                        if let Err(err) = game.draw(ctx) {
                            log::error!("Failed to draw frame: {}", err);
                        }

                        window.window().request_redraw();
                    }

                    #[cfg(target_os = "android")]
                    request_android_redraw();
                }
                Event::MainEventsCleared => {
                    if state_ref.shutdown {
                        log::info!("Shutting down...");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    {
                        let mutex = GAME_SUSPENDED.lock().unwrap();
                        if *mutex {
                            return;
                        }
                    }

                    game.update(ctx).unwrap();

                    #[cfg(target_os = "android")]
                    {
                        match get_insets() {
                            Ok(insets) => {
                                ctx.screen_insets = insets;
                            }
                            Err(e) => {
                                log::error!("Failed to update insets: {}", e);
                            }
                        }

                        if let Err(err) = game.draw(ctx) {
                            log::error!("Failed to draw frame: {}", err);
                        }
                    }

                    if state_ref.next_scene.is_some() {
                        mem::swap(&mut game.scene, &mut state_ref.next_scene);
                        state_ref.next_scene = None;
                        game.scene.as_mut().unwrap().init(state_ref, ctx).unwrap();
                        game.loops = 0;
                        state_ref.frame_time = 0.0;
                    }
                }
                _ => (),
            }
        });
    }

    fn new_renderer(&self) -> GameResult<Box<dyn BackendRenderer>> {
        let mut imgui = imgui::Context::create();
        imgui.io_mut().display_size = [640.0, 480.0];

        Ok(Box::new(GlutinRenderer {
            refs: self.refs.clone(),
            imgui: UnsafeCell::new(imgui),
            imgui_data: ImguiData::new(),
            context_active: Arc::new(RefCell::new(true)),
            def_matrix: [[0.0; 4]; 4],
        }))
    }
}

pub struct GlutinTexture {
    width: u16,
    height: u16,
    texture_id: u32,
    framebuffer_id: u32,
    locs: Locs,
    vbo: GLuint,
    vertices: Vec<VertexData>,
    context_active: Arc<RefCell<bool>>,
}

// #[repr(C)] // since we determine field offset dynamically, doesn't really matter
#[derive(Copy, Clone)]
struct VertexData {
    position: (f32, f32),
    uv: (f32, f32),
    color: (u8, u8, u8, u8),
}

impl BackendTexture for GlutinTexture {
    fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn add(&mut self, command: SpriteBatchCommand) {
        let (tex_scale_x, tex_scale_y) = (1.0 / self.width as f32, 1.0 / self.height as f32);

        match command {
            SpriteBatchCommand::DrawRect(src, dest) => {
                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                ];
                self.vertices.extend_from_slice(&vertices);
            }
            SpriteBatchCommand::DrawRectFlip(mut src, dest, flip_x, flip_y) => {
                if flip_x {
                    std::mem::swap(&mut src.left, &mut src.right);
                }

                if flip_y {
                    std::mem::swap(&mut src.top, &mut src.bottom);
                }

                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color: (255, 255, 255, 255),
                    },
                ];
                self.vertices.extend_from_slice(&vertices);
            }
            SpriteBatchCommand::DrawRectTinted(src, dest, color) => {
                let color = color.to_rgba();
                let vertices = [
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.left, dest.top),
                        uv: (src.left * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.left, dest.bottom),
                        uv: (src.left * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.top),
                        uv: (src.right * tex_scale_x, src.top * tex_scale_y),
                        color,
                    },
                    VertexData {
                        position: (dest.right, dest.bottom),
                        uv: (src.right * tex_scale_x, src.bottom * tex_scale_y),
                        color,
                    },
                ];
                self.vertices.extend_from_slice(&vertices);
            }
        }
    }

    fn clear(&mut self) {
        self.vertices.clear();
    }

    fn draw(&mut self) -> GameResult {
        unsafe {
            if let Some(gl) = GL_PROC.as_ref() {
                if self.texture_id == 0 {
                    return Ok(());
                }

                if gl.gl.BindSampler.is_loaded() {
                    gl.gl.BindSampler(0, 0);
                }

                gl.gl.Enable(gl::TEXTURE_2D);
                gl.gl.Enable(gl::BLEND);
                gl.gl.Disable(gl::DEPTH_TEST);

                gl.gl.BindTexture(gl::TEXTURE_2D, self.texture_id);
                gl.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl.gl.EnableVertexAttribArray(self.locs.position);
                gl.gl.EnableVertexAttribArray(self.locs.uv);
                gl.gl.EnableVertexAttribArray(self.locs.color);

                gl.gl.VertexAttribPointer(
                    self.locs.position,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<VertexData>() as _,
                    field_offset::<VertexData, _, _>(|v| &v.position) as _,
                );

                gl.gl.VertexAttribPointer(
                    self.locs.uv,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<VertexData>() as _,
                    field_offset::<VertexData, _, _>(|v| &v.uv) as _,
                );

                gl.gl.VertexAttribPointer(
                    self.locs.color,
                    4,
                    gl::UNSIGNED_BYTE,
                    gl::TRUE,
                    mem::size_of::<VertexData>() as _,
                    field_offset::<VertexData, _, _>(|v| &v.color) as _,
                );

                gl.gl.BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl.gl.BufferData(
                    gl::ARRAY_BUFFER,
                    (self.vertices.len() * mem::size_of::<VertexData>()) as _,
                    self.vertices.as_ptr() as _,
                    gl::STREAM_DRAW,
                );

                gl.gl.DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as _);

                gl.gl.BindTexture(gl::TEXTURE_2D, 0);
                gl.gl.BindBuffer(gl::ARRAY_BUFFER, 0);

                Ok(())
            } else {
                Err(RenderError("No OpenGL context available!".to_string()))
            }
        }
    }
}

impl Drop for GlutinTexture {
    fn drop(&mut self) {
        if *self.context_active.as_ref().borrow() {
            unsafe {
                if let Some(gl) = GL_PROC.as_ref() {
                    if self.texture_id != 0 {
                        let texture_id = &self.texture_id;
                        gl.gl.DeleteTextures(1, texture_id as *const _);
                    }

                    if self.framebuffer_id != 0 {}
                }
            }
        }
    }
}

fn check_shader_compile_status(shader: u32, gl: &Gl) -> bool {
    unsafe {
        let mut status: GLint = 0;
        gl.gl.GetShaderiv(shader, gl::COMPILE_STATUS, (&mut status) as *mut _);

        if status == (gl::FALSE as GLint) {
            let mut max_length: GLint = 0;
            let mut msg_length: GLsizei = 0;
            gl.gl.GetShaderiv(shader, gl::INFO_LOG_LENGTH, (&mut max_length) as *mut _);

            let mut data: Vec<u8> = vec![0; max_length as usize];
            gl.gl.GetShaderInfoLog(
                shader,
                max_length as GLsizei,
                (&mut msg_length) as *mut _,
                data.as_mut_ptr() as *mut _,
            );

            let data = String::from_utf8_lossy(&data);
            log::error!("Failed to compile shader {}: {}", shader, data);

            return false;
        }
    }

    true
}

const IMGUI_SHADER_VERT: &str = r"
#version 100

precision mediump float;

uniform mat4 ProjMtx;
attribute vec2 Position;
attribute vec2 UV;
attribute vec4 Color;
varying vec2 Frag_UV;
varying vec4 Frag_Color;

void main()
{
    Frag_UV = UV;
    Frag_Color = Color;
    gl_Position = ProjMtx * vec4(Position.xy, 0.0, 1.0);
}

";

const IMGUI_SHADER_FRAG: &str = r"
#version 100

precision mediump float;

uniform sampler2D Texture;
varying vec2 Frag_UV;
varying vec4 Frag_Color;

void main()
{
    gl_FragColor = Frag_Color * texture2D(Texture, Frag_UV.st);
}

";

#[derive(Copy, Clone)]
struct Locs {
    texture: GLint,
    proj_mtx: GLint,
    position: GLuint,
    uv: GLuint,
    color: GLuint,
}

struct ImguiData {
    initialized: bool,
    program: GLuint,
    locs: Locs,
    vbo: GLuint,
    ebo: GLuint,
    font_texture: GLuint,
    font_tex_size: (f32, f32),
}

impl ImguiData {
    fn new() -> Self {
        ImguiData {
            initialized: false,
            program: 0,
            locs: Locs { texture: 0, proj_mtx: 0, position: 0, uv: 0, color: 0 },
            vbo: 0,
            ebo: 0,
            font_texture: 0,
            font_tex_size: (1.0, 1.0),
        }
    }

    fn init(&mut self, imgui: &mut imgui::Context, gl: &Gl) {
        self.initialized = true;

        let vert_sources = [IMGUI_SHADER_VERT.as_ptr() as *const GLchar];
        let frag_sources = [IMGUI_SHADER_FRAG.as_ptr() as *const GLchar];
        let vert_sources_len = [IMGUI_SHADER_VERT.len() as GLint - 1];
        let frag_sources_len = [IMGUI_SHADER_FRAG.len() as GLint - 1];

        unsafe {
            self.program = gl.gl.CreateProgram();
            let vert_shader = gl.gl.CreateShader(gl::VERTEX_SHADER);
            let frag_shader = gl.gl.CreateShader(gl::FRAGMENT_SHADER);
            gl.gl.ShaderSource(vert_shader, 1, vert_sources.as_ptr(), vert_sources_len.as_ptr());
            gl.gl.ShaderSource(frag_shader, 1, frag_sources.as_ptr(), frag_sources_len.as_ptr());
            gl.gl.CompileShader(vert_shader);

            gl.gl.CompileShader(frag_shader);
            gl.gl.AttachShader(self.program, vert_shader);
            gl.gl.AttachShader(self.program, frag_shader);
            gl.gl.LinkProgram(self.program);

            if !check_shader_compile_status(vert_shader, gl) {
                gl.gl.DeleteShader(vert_shader);
            }

            if !check_shader_compile_status(frag_shader, gl) {
                gl.gl.DeleteShader(frag_shader);
            }

            self.locs = Locs {
                texture: gl.gl.GetUniformLocation(self.program, b"Texture\0".as_ptr() as _),
                proj_mtx: gl.gl.GetUniformLocation(self.program, b"ProjMtx\0".as_ptr() as _),
                position: gl.gl.GetAttribLocation(self.program, b"Position\0".as_ptr() as _) as _,
                uv: gl.gl.GetAttribLocation(self.program, b"UV\0".as_ptr() as _) as _,
                color: gl.gl.GetAttribLocation(self.program, b"Color\0".as_ptr() as _) as _,
            };

            self.vbo = return_param(|x| gl.gl.GenBuffers(1, x));
            self.ebo = return_param(|x| gl.gl.GenBuffers(1, x));

            let mut current_texture = 0;
            gl.gl.GetIntegerv(gl::TEXTURE_BINDING_2D, &mut current_texture);

            self.font_texture = return_param(|x| gl.gl.GenTextures(1, x));
            gl.gl.BindTexture(gl::TEXTURE_2D, self.font_texture);
            gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
            //gl.gl.PixelStorei(gl::UNPACK_ROW_LENGTH, 0);

            {
                let mut atlas = imgui.fonts();

                let texture = atlas.build_rgba32_texture();
                self.font_tex_size = (texture.width as _, texture.height as _);

                gl.gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as _,
                    texture.width as _,
                    texture.height as _,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    texture.data.as_ptr() as _,
                );

                atlas.tex_id = (self.font_texture as usize).into();
            }

            gl.gl.BindTexture(gl::TEXTURE_2D, current_texture as _);
        }
    }
}

pub struct GlutinRenderer {
    refs: Rc<UnsafeCell<Option<WindowedContext<PossiblyCurrent>>>>,
    imgui: UnsafeCell<imgui::Context>,
    imgui_data: ImguiData,
    context_active: Arc<RefCell<bool>>,
    def_matrix: [[f32; 4]; 4],
}

pub struct Gl {
    pub gl: gl::Gles2,
}

static mut GL_PROC: Option<Gl> = None;

pub fn load_gl(gl_context: &glutin::Context<PossiblyCurrent>) -> &'static Gl {
    unsafe {
        if let Some(gl) = GL_PROC.as_ref() {
            return gl;
        }

        let gl = gl::Gles2::load_with(|ptr| gl_context.get_proc_address(ptr) as *const _);

        let version = unsafe {
            let data = CStr::from_ptr(gl.GetString(gl::VERSION) as *const _).to_bytes().to_vec();
            String::from_utf8(data).unwrap()
        };

        log::info!("OpenGL version {}", version);

        GL_PROC = Some(Gl { gl });
        GL_PROC.as_ref().unwrap()
    }
}

impl GlutinRenderer {
    fn get_context(&mut self) -> Option<(&mut WindowedContext<PossiblyCurrent>, &'static Gl)> {
        let (refs, imgui) = unsafe { ((&mut *self.refs.get()).as_mut(), &mut *self.imgui.get()) };

        refs.map(|context| {
            let gl = load_gl(context);

            if !self.imgui_data.initialized {
                self.imgui_data.init(imgui, gl);
            }

            (context, gl)
        })
    }
}

fn field_offset<T, U, F: for<'a> FnOnce(&'a T) -> &'a U>(f: F) -> usize {
    unsafe {
        let instance = mem::zeroed::<T>();

        let offset = {
            let field: &U = f(&instance);
            field as *const U as usize - &instance as *const T as usize
        };

        mem::forget(instance);

        offset
    }
}

fn return_param<T, F>(f: F) -> T
where
    F: FnOnce(&mut T),
{
    let mut val = unsafe { mem::zeroed() };
    f(&mut val);
    val
}

impl BackendRenderer for GlutinRenderer {
    fn clear(&mut self, color: Color) {
        if let Some((_, gl)) = self.get_context() {
            unsafe {
                gl.gl.ClearColor(color.r, color.g, color.b, color.a);
                gl.gl.Clear(gl::COLOR_BUFFER_BIT);
            }
        }
    }

    fn present(&mut self) -> GameResult {
        {
            let mutex = GAME_SUSPENDED.lock().unwrap();
            if *mutex {
                return Ok(());
            }
        }

        if let Some((context, gl)) = self.get_context() {
            unsafe {
                gl.gl.Finish();
            }

            context.swap_buffers().map_err(|e| RenderError(e.to_string()))?;
        }

        Ok(())
    }

    fn prepare_draw(&mut self, width: f32, height: f32) -> GameResult {
        if let Some((_, gl)) = self.get_context() {
            unsafe {
                gl.gl.ActiveTexture(gl::TEXTURE0);
                gl.gl.BlendEquation(gl::FUNC_ADD);
                gl.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

                gl.gl.Viewport(0, 0, width as _, height as _);

                self.def_matrix = [
                    [2.0 / width, 0.0, 0.0, 0.0],
                    [0.0, 2.0 / -height, 0.0, 0.0],
                    [0.0, 0.0, -1.0, 0.0],
                    [-1.0, 1.0, 0.0, 1.0],
                ];

                gl.gl.BindBuffer(gl::ARRAY_BUFFER, 0);
                gl.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
                gl.gl.UseProgram(self.imgui_data.program);
                gl.gl.Uniform1i(self.imgui_data.locs.texture, 0);
                gl.gl.UniformMatrix4fv(self.imgui_data.locs.proj_mtx, 1, gl::FALSE, self.def_matrix.as_ptr() as _);
            }

            Ok(())
        } else {
            Err(RenderError("No OpenGL context available!".to_string()))
        }
    }

    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        if let Some((_, gl)) = self.get_context() {
            unsafe {
                let data = vec![0u8; width as usize * height as usize * 4];
                let current_texture_id = return_param(|x| gl.gl.GetIntegerv(gl::TEXTURE_BINDING_2D, x)) as u32;
                let texture_id = return_param(|x| gl.gl.GenTextures(1, x));

                gl.gl.BindTexture(gl::TEXTURE_2D, texture_id);
                gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
                gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);

                gl.gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as _,
                    width as _,
                    height as _,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    data.as_ptr() as _,
                );

                gl.gl.BindTexture(gl::TEXTURE_2D, current_texture_id);

                let framebuffer_id = return_param(|x| gl.gl.GenFramebuffers(1, x));

                gl.gl.BindFramebuffer(gl::FRAMEBUFFER, framebuffer_id);
                gl.gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture_id, 0);
                let draw_buffers = [gl::COLOR_ATTACHMENT0];
                gl.gl.DrawBuffers(1, draw_buffers.as_ptr() as _);

                gl.gl.Viewport(0, 0, width as _, height as _);
                gl.gl.BindFramebuffer(gl::FRAMEBUFFER, 0);

                // todo error checking: glCheckFramebufferStatus()

                Ok(Box::new(GlutinTexture {
                    texture_id,
                    framebuffer_id,
                    width,
                    height,
                    vertices: Vec::new(),
                    locs: self.imgui_data.locs,
                    vbo: self.imgui_data.vbo,
                    context_active: self.context_active.clone(),
                }))
            }
        } else {
            Err(RenderError("No OpenGL context available!".to_string()))
        }
    }

    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        if let Some((_, gl)) = self.get_context() {
            unsafe {
                let current_texture_id = return_param(|x| gl.gl.GetIntegerv(gl::TEXTURE_BINDING_2D, x)) as u32;
                let texture_id = return_param(|x| gl.gl.GenTextures(1, x));
                gl.gl.BindTexture(gl::TEXTURE_2D, texture_id);
                gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
                gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);

                gl.gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as _,
                    width as _,
                    height as _,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    data.as_ptr() as _,
                );

                gl.gl.BindTexture(gl::TEXTURE_2D, current_texture_id);

                Ok(Box::new(GlutinTexture {
                    texture_id,
                    framebuffer_id: 0,
                    width,
                    height,
                    vertices: Vec::new(),
                    locs: self.imgui_data.locs,
                    vbo: self.imgui_data.vbo,
                    context_active: self.context_active.clone(),
                }))
            }
        } else {
            Err(RenderError("No OpenGL context available!".to_string()))
        }
    }

    fn set_blend_mode(&mut self, blend: BlendMode) -> GameResult {
        if let Some((_, gl)) = self.get_context() {
            match blend {
                BlendMode::Add => unsafe {
                    gl.gl.BlendEquation(gl::FUNC_ADD);
                    gl.gl.BlendFunc(gl::ONE, gl::ONE);
                },
                BlendMode::Alpha => unsafe {
                    gl.gl.BlendEquation(gl::FUNC_ADD);
                    gl.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                },
                BlendMode::Multiply => unsafe {
                    gl.gl.BlendEquation(gl::FUNC_ADD);
                    gl.gl.BlendFuncSeparate(gl::ZERO, gl::SRC_COLOR, gl::ZERO, gl::SRC_ALPHA);
                },
            }

            Ok(())
        } else {
            Err(RenderError("No OpenGL context available!".to_string()))
        }
    }

    fn set_render_target(&mut self, texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        if let Some((_, gl)) = self.get_context() {
            unsafe {
                if let Some(texture) = texture {
                    let gl_texture: &Box<GlutinTexture> = std::mem::transmute(texture);

                    let matrix = [
                        [2.0 / (gl_texture.width as f32), 0.0, 0.0, 0.0],
                        [0.0, 2.0 / (gl_texture.height as f32), 0.0, 0.0],
                        [0.0, 0.0, -1.0, 0.0],
                        [-1.0, -1.0, 0.0, 1.0],
                    ];
                    gl.gl.UniformMatrix4fv(self.imgui_data.locs.proj_mtx, 1, gl::FALSE, matrix.as_ptr() as _);

                    gl.gl.BindFramebuffer(gl::FRAMEBUFFER, gl_texture.framebuffer_id);
                } else {
                    gl.gl.UniformMatrix4fv(self.imgui_data.locs.proj_mtx, 1, gl::FALSE, self.def_matrix.as_ptr() as _);
                    gl.gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
                }
            }

            Ok(())
        } else {
            Err(RenderError("No OpenGL context available!".to_string()))
        }
    }

    fn draw_rect(&mut self, rect: Rect<isize>, color: Color) -> GameResult {
        unsafe {
            if let Some(gl) = GL_PROC.as_ref() {
                let color = color.to_rgba();
                let mut uv = self.imgui_data.font_tex_size;
                uv.0 = 0.0 / uv.0;
                uv.1 = 0.0 / uv.1;

                let vertices = [
                    VertexData { position: (rect.left as _, rect.bottom as _), uv, color },
                    VertexData { position: (rect.left as _, rect.top as _), uv, color },
                    VertexData { position: (rect.right as _, rect.top as _), uv, color },
                    VertexData { position: (rect.left as _, rect.bottom as _), uv, color },
                    VertexData { position: (rect.right as _, rect.top as _), uv, color },
                    VertexData { position: (rect.right as _, rect.bottom as _), uv, color },
                ];

                if gl.gl.BindSampler.is_loaded() {
                    gl.gl.BindSampler(0, 0);
                }

                gl.gl.BindBuffer(gl::ARRAY_BUFFER, self.imgui_data.vbo);
                gl.gl.EnableVertexAttribArray(self.imgui_data.locs.position);
                gl.gl.EnableVertexAttribArray(self.imgui_data.locs.uv);
                gl.gl.EnableVertexAttribArray(self.imgui_data.locs.color);

                gl.gl.VertexAttribPointer(
                    self.imgui_data.locs.position,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<VertexData>() as _,
                    field_offset::<VertexData, _, _>(|v| &v.position) as _,
                );

                gl.gl.VertexAttribPointer(
                    self.imgui_data.locs.uv,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<VertexData>() as _,
                    field_offset::<VertexData, _, _>(|v| &v.uv) as _,
                );

                gl.gl.VertexAttribPointer(
                    self.imgui_data.locs.color,
                    4,
                    gl::UNSIGNED_BYTE,
                    gl::TRUE,
                    mem::size_of::<VertexData>() as _,
                    field_offset::<VertexData, _, _>(|v| &v.color) as _,
                );

                gl.gl.BindTexture(gl::TEXTURE_2D, self.imgui_data.font_texture);
                gl.gl.BindBuffer(gl::ARRAY_BUFFER, self.imgui_data.vbo);
                gl.gl.BufferData(
                    gl::ARRAY_BUFFER,
                    (vertices.len() * mem::size_of::<VertexData>()) as _,
                    vertices.as_ptr() as _,
                    gl::STREAM_DRAW,
                );

                gl.gl.DrawArrays(gl::TRIANGLES, 0, vertices.len() as _);

                gl.gl.BindTexture(gl::TEXTURE_2D, 0);
                gl.gl.BindBuffer(gl::ARRAY_BUFFER, 0);

                Ok(())
            } else {
                Err(RenderError("No OpenGL context available!".to_string()))
            }
        }
    }

    fn draw_outline_rect(&mut self, rect: Rect<isize>, line_width: usize, color: Color) -> GameResult {
        Ok(())
    }

    fn imgui(&self) -> GameResult<&mut imgui::Context> {
        unsafe { Ok(&mut *self.imgui.get()) }
    }

    fn render_imgui(&mut self, draw_data: &DrawData) -> GameResult {
        // https://github.com/michaelfairley/rust-imgui-opengl-renderer
        if let Some((_, gl)) = self.get_context() {
            unsafe {
                gl.gl.ActiveTexture(gl::TEXTURE0);
                gl.gl.Enable(gl::BLEND);
                gl.gl.BlendEquation(gl::FUNC_ADD);
                gl.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                gl.gl.Disable(gl::CULL_FACE);
                gl.gl.Disable(gl::DEPTH_TEST);
                gl.gl.Enable(gl::SCISSOR_TEST);

                let imgui = self.imgui()?;
                let [width, height] = imgui.io().display_size;
                let [scale_w, scale_h] = imgui.io().display_framebuffer_scale;

                let fb_width = width * scale_w;
                let fb_height = height * scale_h;

                gl.gl.Viewport(0, 0, fb_width as _, fb_height as _);
                let matrix = [
                    [2.0 / width as f32, 0.0, 0.0, 0.0],
                    [0.0, 2.0 / -(height as f32), 0.0, 0.0],
                    [0.0, 0.0, -1.0, 0.0],
                    [-1.0, 1.0, 0.0, 1.0],
                ];
                gl.gl.UseProgram(self.imgui_data.program);
                gl.gl.Uniform1i(self.imgui_data.locs.texture, 0);
                gl.gl.UniformMatrix4fv(self.imgui_data.locs.proj_mtx, 1, gl::FALSE, matrix.as_ptr() as _);

                if gl.gl.BindSampler.is_loaded() {
                    gl.gl.BindSampler(0, 0);
                }

                // let vao = return_param(|x| gl.gl.GenVertexArrays(1, x));
                //gl.gl.BindVertexArray(vao);
                gl.gl.BindBuffer(gl::ARRAY_BUFFER, self.imgui_data.vbo);
                gl.gl.EnableVertexAttribArray(self.imgui_data.locs.position);
                gl.gl.EnableVertexAttribArray(self.imgui_data.locs.uv);
                gl.gl.EnableVertexAttribArray(self.imgui_data.locs.color);

                gl.gl.VertexAttribPointer(
                    self.imgui_data.locs.position,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<DrawVert>() as _,
                    field_offset::<DrawVert, _, _>(|v| &v.pos) as _,
                );

                gl.gl.VertexAttribPointer(
                    self.imgui_data.locs.uv,
                    2,
                    gl::FLOAT,
                    gl::FALSE,
                    mem::size_of::<DrawVert>() as _,
                    field_offset::<DrawVert, _, _>(|v| &v.uv) as _,
                );

                gl.gl.VertexAttribPointer(
                    self.imgui_data.locs.color,
                    4,
                    gl::UNSIGNED_BYTE,
                    gl::TRUE,
                    mem::size_of::<DrawVert>() as _,
                    field_offset::<DrawVert, _, _>(|v| &v.col) as _,
                );

                for draw_list in draw_data.draw_lists() {
                    let vtx_buffer = draw_list.vtx_buffer();
                    let idx_buffer = draw_list.idx_buffer();

                    gl.gl.BindBuffer(gl::ARRAY_BUFFER, self.imgui_data.vbo);
                    gl.gl.BufferData(
                        gl::ARRAY_BUFFER,
                        (vtx_buffer.len() * mem::size_of::<DrawVert>()) as _,
                        vtx_buffer.as_ptr() as _,
                        gl::STREAM_DRAW,
                    );

                    gl.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.imgui_data.ebo);
                    gl.gl.BufferData(
                        gl::ELEMENT_ARRAY_BUFFER,
                        (idx_buffer.len() * mem::size_of::<DrawIdx>()) as _,
                        idx_buffer.as_ptr() as _,
                        gl::STREAM_DRAW,
                    );

                    for cmd in draw_list.commands() {
                        match cmd {
                            DrawCmd::Elements {
                                count,
                                cmd_params: DrawCmdParams { clip_rect: [x, y, z, w], texture_id, idx_offset, .. },
                            } => {
                                gl.gl.BindTexture(gl::TEXTURE_2D, texture_id.id() as _);

                                gl.gl.Scissor(
                                    (x * scale_w) as GLint,
                                    (fb_height - w * scale_h) as GLint,
                                    ((z - x) * scale_w) as GLint,
                                    ((w - y) * scale_h) as GLint,
                                );

                                let idx_size =
                                    if mem::size_of::<DrawIdx>() == 2 { gl::UNSIGNED_SHORT } else { gl::UNSIGNED_INT };

                                gl.gl.DrawElements(
                                    gl::TRIANGLES,
                                    count as _,
                                    idx_size,
                                    (idx_offset * mem::size_of::<DrawIdx>()) as _,
                                );
                            }
                            DrawCmd::ResetRenderState => {}
                            DrawCmd::RawCallback { .. } => {}
                        }
                    }
                }

                //gl.gl.DeleteVertexArrays(1, &vao);
            }
        }

        Ok(())
    }
}

impl Drop for GlutinRenderer {
    fn drop(&mut self) {
        *self.context_active.as_ref().borrow_mut() = false;
    }
}

fn conv_keycode(code: VirtualKeyCode) -> Option<ScanCode> {
    match code {
        VirtualKeyCode::Key1 => Some(ScanCode::Key1),
        VirtualKeyCode::Key2 => Some(ScanCode::Key2),
        VirtualKeyCode::Key3 => Some(ScanCode::Key3),
        VirtualKeyCode::Key4 => Some(ScanCode::Key4),
        VirtualKeyCode::Key5 => Some(ScanCode::Key5),
        VirtualKeyCode::Key6 => Some(ScanCode::Key6),
        VirtualKeyCode::Key7 => Some(ScanCode::Key7),
        VirtualKeyCode::Key8 => Some(ScanCode::Key8),
        VirtualKeyCode::Key9 => Some(ScanCode::Key9),
        VirtualKeyCode::Key0 => Some(ScanCode::Key0),
        VirtualKeyCode::A => Some(ScanCode::A),
        VirtualKeyCode::B => Some(ScanCode::B),
        VirtualKeyCode::C => Some(ScanCode::C),
        VirtualKeyCode::D => Some(ScanCode::D),
        VirtualKeyCode::E => Some(ScanCode::E),
        VirtualKeyCode::F => Some(ScanCode::F),
        VirtualKeyCode::G => Some(ScanCode::G),
        VirtualKeyCode::H => Some(ScanCode::H),
        VirtualKeyCode::I => Some(ScanCode::I),
        VirtualKeyCode::J => Some(ScanCode::J),
        VirtualKeyCode::K => Some(ScanCode::K),
        VirtualKeyCode::L => Some(ScanCode::L),
        VirtualKeyCode::M => Some(ScanCode::M),
        VirtualKeyCode::N => Some(ScanCode::N),
        VirtualKeyCode::O => Some(ScanCode::O),
        VirtualKeyCode::P => Some(ScanCode::P),
        VirtualKeyCode::Q => Some(ScanCode::Q),
        VirtualKeyCode::R => Some(ScanCode::R),
        VirtualKeyCode::S => Some(ScanCode::S),
        VirtualKeyCode::T => Some(ScanCode::T),
        VirtualKeyCode::U => Some(ScanCode::U),
        VirtualKeyCode::V => Some(ScanCode::V),
        VirtualKeyCode::W => Some(ScanCode::W),
        VirtualKeyCode::X => Some(ScanCode::X),
        VirtualKeyCode::Y => Some(ScanCode::Y),
        VirtualKeyCode::Z => Some(ScanCode::Z),
        VirtualKeyCode::Escape => Some(ScanCode::Escape),
        VirtualKeyCode::F1 => Some(ScanCode::F1),
        VirtualKeyCode::F2 => Some(ScanCode::F2),
        VirtualKeyCode::F3 => Some(ScanCode::F3),
        VirtualKeyCode::F4 => Some(ScanCode::F4),
        VirtualKeyCode::F5 => Some(ScanCode::F5),
        VirtualKeyCode::F6 => Some(ScanCode::F6),
        VirtualKeyCode::F7 => Some(ScanCode::F7),
        VirtualKeyCode::F8 => Some(ScanCode::F8),
        VirtualKeyCode::F9 => Some(ScanCode::F9),
        VirtualKeyCode::F10 => Some(ScanCode::F10),
        VirtualKeyCode::F11 => Some(ScanCode::F11),
        VirtualKeyCode::F12 => Some(ScanCode::F12),
        VirtualKeyCode::F13 => Some(ScanCode::F13),
        VirtualKeyCode::F14 => Some(ScanCode::F14),
        VirtualKeyCode::F15 => Some(ScanCode::F15),
        VirtualKeyCode::F16 => Some(ScanCode::F16),
        VirtualKeyCode::F17 => Some(ScanCode::F17),
        VirtualKeyCode::F18 => Some(ScanCode::F18),
        VirtualKeyCode::F19 => Some(ScanCode::F19),
        VirtualKeyCode::F20 => Some(ScanCode::F20),
        VirtualKeyCode::F21 => Some(ScanCode::F21),
        VirtualKeyCode::F22 => Some(ScanCode::F22),
        VirtualKeyCode::F23 => Some(ScanCode::F23),
        VirtualKeyCode::F24 => Some(ScanCode::F24),
        VirtualKeyCode::Snapshot => Some(ScanCode::Snapshot),
        VirtualKeyCode::Scroll => Some(ScanCode::Scroll),
        VirtualKeyCode::Pause => Some(ScanCode::Pause),
        VirtualKeyCode::Insert => Some(ScanCode::Insert),
        VirtualKeyCode::Home => Some(ScanCode::Home),
        VirtualKeyCode::Delete => Some(ScanCode::Delete),
        VirtualKeyCode::End => Some(ScanCode::End),
        VirtualKeyCode::PageDown => Some(ScanCode::PageDown),
        VirtualKeyCode::PageUp => Some(ScanCode::PageUp),
        VirtualKeyCode::Left => Some(ScanCode::Left),
        VirtualKeyCode::Up => Some(ScanCode::Up),
        VirtualKeyCode::Right => Some(ScanCode::Right),
        VirtualKeyCode::Down => Some(ScanCode::Down),
        VirtualKeyCode::Back => Some(ScanCode::Back),
        VirtualKeyCode::Return => Some(ScanCode::Return),
        VirtualKeyCode::Space => Some(ScanCode::Space),
        VirtualKeyCode::Compose => Some(ScanCode::Compose),
        VirtualKeyCode::Caret => Some(ScanCode::Caret),
        VirtualKeyCode::Numlock => Some(ScanCode::Numlock),
        VirtualKeyCode::Numpad0 => Some(ScanCode::Numpad0),
        VirtualKeyCode::Numpad1 => Some(ScanCode::Numpad1),
        VirtualKeyCode::Numpad2 => Some(ScanCode::Numpad2),
        VirtualKeyCode::Numpad3 => Some(ScanCode::Numpad3),
        VirtualKeyCode::Numpad4 => Some(ScanCode::Numpad4),
        VirtualKeyCode::Numpad5 => Some(ScanCode::Numpad5),
        VirtualKeyCode::Numpad6 => Some(ScanCode::Numpad6),
        VirtualKeyCode::Numpad7 => Some(ScanCode::Numpad7),
        VirtualKeyCode::Numpad8 => Some(ScanCode::Numpad8),
        VirtualKeyCode::Numpad9 => Some(ScanCode::Numpad9),
        VirtualKeyCode::NumpadAdd => Some(ScanCode::NumpadAdd),
        VirtualKeyCode::NumpadDivide => Some(ScanCode::NumpadDivide),
        VirtualKeyCode::NumpadDecimal => Some(ScanCode::NumpadDecimal),
        VirtualKeyCode::NumpadComma => Some(ScanCode::NumpadComma),
        VirtualKeyCode::NumpadEnter => Some(ScanCode::NumpadEnter),
        VirtualKeyCode::NumpadEquals => Some(ScanCode::NumpadEquals),
        VirtualKeyCode::NumpadMultiply => Some(ScanCode::NumpadMultiply),
        VirtualKeyCode::NumpadSubtract => Some(ScanCode::NumpadSubtract),
        VirtualKeyCode::AbntC1 => Some(ScanCode::AbntC1),
        VirtualKeyCode::AbntC2 => Some(ScanCode::AbntC2),
        VirtualKeyCode::Apostrophe => Some(ScanCode::Apostrophe),
        VirtualKeyCode::Apps => Some(ScanCode::Apps),
        VirtualKeyCode::Asterisk => Some(ScanCode::Asterisk),
        VirtualKeyCode::At => Some(ScanCode::At),
        VirtualKeyCode::Ax => Some(ScanCode::Ax),
        VirtualKeyCode::Backslash => Some(ScanCode::Backslash),
        VirtualKeyCode::Calculator => Some(ScanCode::Calculator),
        VirtualKeyCode::Capital => Some(ScanCode::Capital),
        VirtualKeyCode::Colon => Some(ScanCode::Colon),
        VirtualKeyCode::Comma => Some(ScanCode::Comma),
        VirtualKeyCode::Convert => Some(ScanCode::Convert),
        VirtualKeyCode::Equals => Some(ScanCode::Equals),
        VirtualKeyCode::Grave => Some(ScanCode::Grave),
        VirtualKeyCode::Kana => Some(ScanCode::Kana),
        VirtualKeyCode::Kanji => Some(ScanCode::Kanji),
        VirtualKeyCode::LAlt => Some(ScanCode::LAlt),
        VirtualKeyCode::LBracket => Some(ScanCode::LBracket),
        VirtualKeyCode::LControl => Some(ScanCode::LControl),
        VirtualKeyCode::LShift => Some(ScanCode::LShift),
        VirtualKeyCode::LWin => Some(ScanCode::LWin),
        VirtualKeyCode::Mail => Some(ScanCode::Mail),
        VirtualKeyCode::MediaSelect => Some(ScanCode::MediaSelect),
        VirtualKeyCode::MediaStop => Some(ScanCode::MediaStop),
        VirtualKeyCode::Minus => Some(ScanCode::Minus),
        VirtualKeyCode::Mute => Some(ScanCode::Mute),
        VirtualKeyCode::MyComputer => Some(ScanCode::MyComputer),
        VirtualKeyCode::NavigateForward => Some(ScanCode::NavigateForward),
        VirtualKeyCode::NavigateBackward => Some(ScanCode::NavigateBackward),
        VirtualKeyCode::NextTrack => Some(ScanCode::NextTrack),
        VirtualKeyCode::NoConvert => Some(ScanCode::NoConvert),
        VirtualKeyCode::OEM102 => Some(ScanCode::OEM102),
        VirtualKeyCode::Period => Some(ScanCode::Period),
        VirtualKeyCode::PlayPause => Some(ScanCode::PlayPause),
        VirtualKeyCode::Plus => Some(ScanCode::Plus),
        VirtualKeyCode::Power => Some(ScanCode::Power),
        VirtualKeyCode::PrevTrack => Some(ScanCode::PrevTrack),
        VirtualKeyCode::RAlt => Some(ScanCode::RAlt),
        VirtualKeyCode::RBracket => Some(ScanCode::RBracket),
        VirtualKeyCode::RControl => Some(ScanCode::RControl),
        VirtualKeyCode::RShift => Some(ScanCode::RShift),
        VirtualKeyCode::RWin => Some(ScanCode::RWin),
        VirtualKeyCode::Semicolon => Some(ScanCode::Semicolon),
        VirtualKeyCode::Slash => Some(ScanCode::Slash),
        VirtualKeyCode::Sleep => Some(ScanCode::Sleep),
        VirtualKeyCode::Stop => Some(ScanCode::Stop),
        VirtualKeyCode::Sysrq => Some(ScanCode::Sysrq),
        VirtualKeyCode::Tab => Some(ScanCode::Tab),
        VirtualKeyCode::Underline => Some(ScanCode::Underline),
        VirtualKeyCode::Unlabeled => Some(ScanCode::Unlabeled),
        VirtualKeyCode::VolumeDown => Some(ScanCode::VolumeDown),
        VirtualKeyCode::VolumeUp => Some(ScanCode::VolumeUp),
        VirtualKeyCode::Wake => Some(ScanCode::Wake),
        VirtualKeyCode::WebBack => Some(ScanCode::WebBack),
        VirtualKeyCode::WebFavorites => Some(ScanCode::WebFavorites),
        VirtualKeyCode::WebForward => Some(ScanCode::WebForward),
        VirtualKeyCode::WebHome => Some(ScanCode::WebHome),
        VirtualKeyCode::WebRefresh => Some(ScanCode::WebRefresh),
        VirtualKeyCode::WebSearch => Some(ScanCode::WebSearch),
        VirtualKeyCode::WebStop => Some(ScanCode::WebStop),
        VirtualKeyCode::Yen => Some(ScanCode::Yen),
        VirtualKeyCode::Copy => Some(ScanCode::Copy),
        VirtualKeyCode::Paste => Some(ScanCode::Paste),
        VirtualKeyCode::Cut => Some(ScanCode::Cut),
    }
}
