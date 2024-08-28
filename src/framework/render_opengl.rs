use std::any::Any;
use std::borrow::BorrowMut;
use std::cell::{Cell, RefCell, UnsafeCell};
use std::ffi::{c_void, CStr};
use std::hint::unreachable_unchecked;
use std::mem;
use std::mem::MaybeUninit;
use std::ptr::null;
use std::rc::Rc;
use std::sync::Arc;

use imgui::{DrawCmd, DrawCmdParams, DrawData, DrawIdx, DrawVert, TextureId, Ui};

use super::backend::{BackendRenderer, BackendShader, BackendTexture, SpriteBatchCommand, VertexData};
use super::context::Context;
use super::error::GameError;
use super::error::GameError::RenderError;
use super::error::GameResult;
use super::gl;
use super::gl::types::*;
use super::graphics::BlendMode;
use super::graphics::SwapMode;
use super::util::{field_offset, return_param};
use crate::common::{Color, Rect};
use crate::game::GAME_SUSPENDED;

pub trait GLPlatformFunctions {
    fn get_proc_address(&self, name: &str) -> *const c_void;

    fn swap_buffers(&self);

    fn set_swap_mode(&self, mode: SwapMode);
}

pub struct GLContext {
    pub gles2_mode: bool,
    pub platform: Box<dyn GLPlatformFunctions>,
}

pub struct OpenGLTexture {
    width: u16,
    height: u16,
    texture_id: u32,
    framebuffer_id: u32,
    shader: RenderShader,
    vbo: GLuint,
    vertices: Vec<VertexData>,
    gl: Rc<Gl>,
}

impl BackendTexture for OpenGLTexture {
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
            SpriteBatchCommand::DrawRectFlipTinted(mut src, dest, flip_x, flip_y, color) => {
                if flip_x {
                    std::mem::swap(&mut src.left, &mut src.right);
                }

                if flip_y {
                    std::mem::swap(&mut src.top, &mut src.bottom);
                }

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
        if self.vertices.is_empty() {
            return Ok(());
        }

        unsafe {
            let gl = self.gl.as_ref();
            if self.texture_id == 0 {
                return Ok(());
            }

            if gl.gl.BindSampler.is_loaded() {
                gl.gl.BindSampler(0, 0);
            }

            gl.gl.Enable(gl::TEXTURE_2D);
            gl.gl.Enable(gl::BLEND);
            gl.gl.Disable(gl::DEPTH_TEST);

            self.shader.bind_attrib_pointer(gl, self.vbo);

            gl.gl.BindTexture(gl::TEXTURE_2D, self.texture_id);
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
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for OpenGLTexture {
    fn drop(&mut self) {
        unsafe {
            let gl = self.gl.as_ref();
            if !*gl.context_active.borrow() {
                return;
            }

            if self.texture_id != 0 {
                let texture_id = &self.texture_id;
                gl.gl.DeleteTextures(1, texture_id as *const _);
            }

            // if self.framebuffer_id != 0 {
            //     let framebuffer_id = &self.framebuffer_id;
            //     gl.gl.DeleteFramebuffers(1, framebuffer_id as *const _);
            // }
        }
    }
}

fn check_shader_compile_status(shader: u32, gl: &Gl) -> GameResult {
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
            return Err(GameError::RenderError(format!("Failed to compile shader {}: {}", shader, data)));
        }
    }

    Ok(())
}

const VERTEX_SHADER_BASIC: &str = include_str!("shaders/opengl/vertex_basic_110.glsl");
const FRAGMENT_SHADER_TEXTURED: &str = include_str!("shaders/opengl/fragment_textured_110.glsl");
const FRAGMENT_SHADER_COLOR: &str = include_str!("shaders/opengl/fragment_color_110.glsl");
const FRAGMENT_SHADER_WATER: &str = include_str!("shaders/opengl/fragment_water_110.glsl");

const VERTEX_SHADER_BASIC_GLES: &str = include_str!("shaders/opengles/vertex_basic_100.glsl");
const FRAGMENT_SHADER_TEXTURED_GLES: &str = include_str!("shaders/opengles/fragment_textured_100.glsl");
const FRAGMENT_SHADER_COLOR_GLES: &str = include_str!("shaders/opengles/fragment_color_100.glsl");

#[derive(Copy, Clone)]
struct RenderShader {
    program_id: GLuint,
    texture: GLint,
    proj_mtx: GLint,
    scale: GLint,
    time: GLint,
    frame_offset: GLint,
    position: GLuint,
    uv: GLuint,
    color: GLuint,
}

impl Default for RenderShader {
    fn default() -> Self {
        Self {
            program_id: 0,
            texture: 0,
            proj_mtx: 0,
            scale: 0,
            time: 0,
            frame_offset: 0,
            position: 0,
            uv: 0,
            color: 0,
        }
    }
}

impl RenderShader {
    fn compile(gl: &Gl, vertex_shader: &str, fragment_shader: &str) -> GameResult<RenderShader> {
        let mut shader = RenderShader::default();
        unsafe {
            shader.program_id = gl.gl.CreateProgram();

            unsafe fn cleanup(shader: &mut RenderShader, gl: &Gl, vert: GLuint, frag: GLuint) {
                if vert != 0 {
                    gl.gl.DeleteShader(vert);
                }

                if frag != 0 {
                    gl.gl.DeleteShader(frag);
                }

                if shader.program_id != 0 {
                    gl.gl.DeleteProgram(shader.program_id);
                    shader.program_id = 0;
                }

                *shader = RenderShader::default();
            }

            let vert_shader = gl.gl.CreateShader(gl::VERTEX_SHADER);
            let frag_shader = gl.gl.CreateShader(gl::FRAGMENT_SHADER);

            let vert_sources = [vertex_shader.as_ptr() as *const GLchar];
            let frag_sources = [fragment_shader.as_ptr() as *const GLchar];
            let vert_sources_len = [vertex_shader.len() as GLint - 1];
            let frag_sources_len = [fragment_shader.len() as GLint - 1];

            gl.gl.ShaderSource(vert_shader, 1, vert_sources.as_ptr(), vert_sources_len.as_ptr());
            gl.gl.ShaderSource(frag_shader, 1, frag_sources.as_ptr(), frag_sources_len.as_ptr());

            gl.gl.CompileShader(vert_shader);
            gl.gl.CompileShader(frag_shader);

            if let Err(e) = check_shader_compile_status(vert_shader, gl) {
                cleanup(&mut shader, gl, vert_shader, frag_shader);
                return Err(e);
            }

            if let Err(e) = check_shader_compile_status(frag_shader, gl) {
                cleanup(&mut shader, gl, vert_shader, frag_shader);
                return Err(e);
            }

            gl.gl.AttachShader(shader.program_id, vert_shader);
            gl.gl.AttachShader(shader.program_id, frag_shader);
            gl.gl.LinkProgram(shader.program_id);

            shader.texture = gl.gl.GetUniformLocation(shader.program_id, b"Texture\0".as_ptr() as _);
            shader.proj_mtx = gl.gl.GetUniformLocation(shader.program_id, b"ProjMtx\0".as_ptr() as _);
            shader.scale = gl.gl.GetUniformLocation(shader.program_id, b"Scale\0".as_ptr() as _) as _;
            shader.time = gl.gl.GetUniformLocation(shader.program_id, b"Time\0".as_ptr() as _) as _;
            shader.frame_offset = gl.gl.GetUniformLocation(shader.program_id, b"FrameOffset\0".as_ptr() as _) as _;
            shader.position = gl.gl.GetAttribLocation(shader.program_id, b"Position\0".as_ptr() as _) as _;
            shader.uv = gl.gl.GetAttribLocation(shader.program_id, b"UV\0".as_ptr() as _) as _;
            shader.color = gl.gl.GetAttribLocation(shader.program_id, b"Color\0".as_ptr() as _) as _;
        }

        Ok(shader)
    }

    unsafe fn bind_attrib_pointer(&self, gl: &Gl, vbo: GLuint) -> GameResult {
        gl.gl.UseProgram(self.program_id);
        gl.gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl.gl.EnableVertexAttribArray(self.position);
        gl.gl.EnableVertexAttribArray(self.uv);
        gl.gl.EnableVertexAttribArray(self.color);

        gl.gl.VertexAttribPointer(
            self.position,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<VertexData>() as _,
            field_offset::<VertexData, _, _>(|v| &v.position) as _,
        );

        gl.gl.VertexAttribPointer(
            self.uv,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<VertexData>() as _,
            field_offset::<VertexData, _, _>(|v| &v.uv) as _,
        );

        gl.gl.VertexAttribPointer(
            self.color,
            4,
            gl::UNSIGNED_BYTE,
            gl::TRUE,
            mem::size_of::<VertexData>() as _,
            field_offset::<VertexData, _, _>(|v| &v.color) as _,
        );

        Ok(())
    }
}

struct RenderData {
    initialized: bool,
    tex_shader: RenderShader,
    fill_shader: RenderShader,
    fill_water_shader: RenderShader,
    render_fbo: GLint,
    vbo: GLuint,
    ebo: GLuint,
    font_texture: GLuint,
    font_tex_size: (f32, f32),
    surf_framebuffer: GLuint,
    surf_texture: GLuint,
    last_size: (u32, u32),
}

impl RenderData {
    fn new() -> Self {
        RenderData {
            initialized: false,
            tex_shader: RenderShader::default(),
            fill_shader: RenderShader::default(),
            fill_water_shader: RenderShader::default(),
            render_fbo: 0,
            vbo: 0,
            ebo: 0,
            font_texture: 0,
            font_tex_size: (1.0, 1.0),
            surf_framebuffer: 0,
            surf_texture: 0,
            last_size: (320, 240),
        }
    }

    fn init(&mut self, gles2_mode: bool, imgui: &mut imgui::Context, gl: &Gl) {
        self.initialized = true;

        let vshdr_basic = if gles2_mode { VERTEX_SHADER_BASIC_GLES } else { VERTEX_SHADER_BASIC };
        let fshdr_tex = if gles2_mode { FRAGMENT_SHADER_TEXTURED_GLES } else { FRAGMENT_SHADER_TEXTURED };
        let fshdr_fill = if gles2_mode { FRAGMENT_SHADER_COLOR_GLES } else { FRAGMENT_SHADER_COLOR };
        let fshdr_fill_water = if gles2_mode { FRAGMENT_SHADER_COLOR_GLES } else { FRAGMENT_SHADER_WATER };

        unsafe {
            // iOS has "unusual" framebuffer setup, where we can't rely on 0 as the system provided render target.
            self.render_fbo = return_param(|x| gl.gl.GetIntegerv(gl::FRAMEBUFFER_BINDING, x));

            self.tex_shader = RenderShader::compile(gl, vshdr_basic, fshdr_tex).unwrap_or_else(|e| {
                log::error!("Failed to compile texture shader: {}", e);
                RenderShader::default()
            });
            self.fill_shader = RenderShader::compile(gl, vshdr_basic, fshdr_fill).unwrap_or_else(|e| {
                log::error!("Failed to compile fill shader: {}", e);
                RenderShader::default()
            });
            self.fill_water_shader = RenderShader::compile(gl, vshdr_basic, fshdr_fill_water).unwrap_or_else(|e| {
                log::error!("Failed to compile fill water shader: {}", e);
                RenderShader::default()
            });

            self.vbo = return_param(|x| gl.gl.GenBuffers(1, x));
            self.ebo = return_param(|x| gl.gl.GenBuffers(1, x));

            self.font_texture = return_param(|x| gl.gl.GenTextures(1, x));
            gl.gl.BindTexture(gl::TEXTURE_2D, self.font_texture);
            gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);

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

            let texture_id = return_param(|x| gl.gl.GenTextures(1, x));

            gl.gl.BindTexture(gl::TEXTURE_2D, texture_id);
            gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
            gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as _);
            gl.gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as _);

            gl.gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as _,
                320 as _,
                240 as _,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                null() as _,
            );

            gl.gl.BindTexture(gl::TEXTURE_2D, 0 as _);

            self.surf_texture = texture_id;

            let framebuffer_id = return_param(|x| gl.gl.GenFramebuffers(1, x));

            gl.gl.BindFramebuffer(gl::FRAMEBUFFER, framebuffer_id);
            gl.gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture_id, 0);
            let draw_buffers = [gl::COLOR_ATTACHMENT0];
            gl.gl.DrawBuffers(1, draw_buffers.as_ptr() as _);

            self.surf_framebuffer = framebuffer_id;
        }
    }
}

pub struct Gl {
    pub gl: gl::Gles2,
    pub context_active: RefCell<bool>,
}

impl Gl {
    pub fn ensure_compatibility(&self) -> GameResult {
        fn ensure_function_loaded(ptr: &gl::FnPtr) -> GameResult {
            if !ptr.is_loaded() {
                return Err(RenderError("Created context does not provide required OpenGL functionality!".to_string()));
            }

            Ok(())
        }

        ensure_function_loaded(&self.gl.ActiveTexture)?;
        ensure_function_loaded(&self.gl.AttachShader)?;
        ensure_function_loaded(&self.gl.BindBuffer)?;
        ensure_function_loaded(&self.gl.BindFramebuffer)?;
        // ensure_function_loaded(&self.gl.BindSampler)?; // optional
        ensure_function_loaded(&self.gl.BindTexture)?;
        ensure_function_loaded(&self.gl.BlendEquation)?;
        ensure_function_loaded(&self.gl.BlendFunc)?;
        ensure_function_loaded(&self.gl.BlendFuncSeparate)?;
        ensure_function_loaded(&self.gl.BufferData)?;
        ensure_function_loaded(&self.gl.Clear)?;
        ensure_function_loaded(&self.gl.ClearColor)?;
        ensure_function_loaded(&self.gl.CompileShader)?;
        ensure_function_loaded(&self.gl.CreateProgram)?;
        ensure_function_loaded(&self.gl.CreateShader)?;
        ensure_function_loaded(&self.gl.DeleteFramebuffers)?;
        ensure_function_loaded(&self.gl.DeleteProgram)?;
        ensure_function_loaded(&self.gl.DeleteShader)?;
        ensure_function_loaded(&self.gl.DeleteTextures)?;
        ensure_function_loaded(&self.gl.Disable)?;
        ensure_function_loaded(&self.gl.DrawArrays)?;
        ensure_function_loaded(&self.gl.DrawBuffers)?;
        ensure_function_loaded(&self.gl.DrawElements)?;
        ensure_function_loaded(&self.gl.Enable)?;
        ensure_function_loaded(&self.gl.EnableVertexAttribArray)?;
        ensure_function_loaded(&self.gl.FramebufferTexture2D)?;
        ensure_function_loaded(&self.gl.GenBuffers)?;
        ensure_function_loaded(&self.gl.GenFramebuffers)?;
        ensure_function_loaded(&self.gl.GenTextures)?;
        ensure_function_loaded(&self.gl.GetAttribLocation)?;
        ensure_function_loaded(&self.gl.GetIntegerv)?;
        ensure_function_loaded(&self.gl.GetShaderInfoLog)?;
        ensure_function_loaded(&self.gl.GetShaderiv)?;
        ensure_function_loaded(&self.gl.GetUniformLocation)?;
        ensure_function_loaded(&self.gl.LinkProgram)?;
        ensure_function_loaded(&self.gl.Scissor)?;
        ensure_function_loaded(&self.gl.ShaderSource)?;
        ensure_function_loaded(&self.gl.TexImage2D)?;
        ensure_function_loaded(&self.gl.TexParameteri)?;
        ensure_function_loaded(&self.gl.Uniform1f)?;
        ensure_function_loaded(&self.gl.Uniform1i)?;
        ensure_function_loaded(&self.gl.Uniform2f)?;
        ensure_function_loaded(&self.gl.UniformMatrix4fv)?;
        ensure_function_loaded(&self.gl.UseProgram)?;
        ensure_function_loaded(&self.gl.VertexAttribPointer)?;
        ensure_function_loaded(&self.gl.Viewport)?;

        Ok(())
    }

    fn get_version(&self) -> String {
        let p = unsafe { self.gl.GetString(gl::VERSION) };
        if p.is_null() {
            "unknown".to_owned()
        } else {
            let data = unsafe { CStr::from_ptr(p as *const _).to_bytes().to_vec() };
            String::from_utf8_lossy(&data).to_string()
        }
    }
}

pub fn load_gl(gl_context: &GLContext) -> Rc<Gl> {
    Rc::new(Gl {
        gl: unsafe { gl::Gles2::load_with(|ptr| gl_context.platform.get_proc_address(ptr)) },
        context_active: RefCell::new(true),
    })
}

pub struct OpenGLRenderer {
    refs: GLContext,
    gl: Rc<Gl>,
    imgui: Rc<RefCell<imgui::Context>>,
    render_data: RenderData,
    def_matrix: [[f32; 4]; 4],
    curr_matrix: [[f32; 4]; 4],
}

impl OpenGLRenderer {
    pub const RENDERER_ID: &'static str = "opengl2";

    pub fn new(refs: GLContext, mut imgui: imgui::Context) -> GameResult<Box<dyn BackendRenderer>> {
        let gl = load_gl(&refs);
        gl.ensure_compatibility()?;

        log::info!("OpenGL version: {}", gl.get_version());

        let mut render_data = RenderData::new();
        render_data.init(refs.gles2_mode, &mut imgui, &gl);

        Ok(Box::new(OpenGLRenderer {
            refs,
            gl,
            imgui: Rc::new(RefCell::new(imgui)),
            render_data,
            def_matrix: [[0.0; 4]; 4],
            curr_matrix: [[0.0; 4]; 4],
        }))
    }
}

impl BackendRenderer for OpenGLRenderer {
    fn renderer_name(&self) -> String {
        if self.refs.gles2_mode {
            "*COMPATIBILITY* OpenGL ES 2.0".to_string()
        } else {
            "*COMPATIBILITY* OpenGL 2.1".to_string()
        }
    }

    fn clear(&mut self, color: Color) {
        unsafe {
            let gl = &self.gl;
            gl.gl.ClearColor(color.r, color.g, color.b, color.a);
            gl.gl.Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn present(&mut self) -> GameResult {
        {
            let mutex = GAME_SUSPENDED.lock().unwrap();
            if *mutex {
                return Ok(());
            }
        }

        unsafe {
            let gl = &self.gl;
            gl.gl.BindFramebuffer(gl::FRAMEBUFFER, self.render_data.render_fbo as _);
            gl.gl.ClearColor(0.0, 0.0, 0.0, 1.0);
            gl.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            let matrix = [[2.0f32, 0.0, 0.0, 0.0], [0.0, -2.0, 0.0, 0.0], [0.0, 0.0, -1.0, 0.0], [-1.0, 1.0, 0.0, 1.0]];

            self.render_data.tex_shader.bind_attrib_pointer(&gl, self.render_data.vbo);
            gl.gl.UniformMatrix4fv(self.render_data.tex_shader.proj_mtx, 1, gl::FALSE, matrix.as_ptr() as _);

            let color = (255, 255, 255, 255);
            let vertices = [
                VertexData { position: (0.0, 1.0), uv: (0.0, 0.0), color },
                VertexData { position: (0.0, 0.0), uv: (0.0, 1.0), color },
                VertexData { position: (1.0, 0.0), uv: (1.0, 1.0), color },
                VertexData { position: (0.0, 1.0), uv: (0.0, 0.0), color },
                VertexData { position: (1.0, 0.0), uv: (1.0, 1.0), color },
                VertexData { position: (1.0, 1.0), uv: (1.0, 0.0), color },
            ];

            self.draw_arrays_tex_id(gl::TRIANGLES, &vertices, self.render_data.surf_texture, BackendShader::Texture)?;

            self.refs.platform.swap_buffers();
        }

        Ok(())
    }

    fn set_swap_mode(&mut self, mode: SwapMode) -> GameResult {
        self.refs.platform.set_swap_mode(mode);
        Ok(())
    }

    fn prepare_draw(&mut self, width: f32, height: f32) -> GameResult {
        unsafe {
            let gl = &self.gl;
            let (width_u, height_u) = (width as u32, height as u32);
            if self.render_data.last_size != (width_u, height_u) {
                self.render_data.last_size = (width_u, height_u);
                gl.gl.BindFramebuffer(gl::FRAMEBUFFER, self.render_data.render_fbo as _);
                gl.gl.BindTexture(gl::TEXTURE_2D, self.render_data.surf_texture);

                gl.gl.TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as _,
                    width_u as _,
                    height_u as _,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    null() as _,
                );

                gl.gl.BindTexture(gl::TEXTURE_2D, 0 as _);
            }

            gl.gl.BindFramebuffer(gl::FRAMEBUFFER, self.render_data.surf_framebuffer);
            gl.gl.ClearColor(0.0, 0.0, 0.0, 0.0);
            gl.gl.Clear(gl::COLOR_BUFFER_BIT);

            gl.gl.ActiveTexture(gl::TEXTURE0);
            gl.gl.BlendEquation(gl::FUNC_ADD);
            gl.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            gl.gl.Viewport(0, 0, width_u as _, height_u as _);

            self.def_matrix = [
                [2.0 / width, 0.0, 0.0, 0.0],
                [0.0, 2.0 / -height, 0.0, 0.0],
                [0.0, 0.0, -1.0, 0.0],
                [-1.0, 1.0, 0.0, 1.0],
            ];
            self.curr_matrix = self.def_matrix;

            gl.gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            gl.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl.gl.UseProgram(self.render_data.fill_shader.program_id);
            gl.gl.UniformMatrix4fv(self.render_data.fill_shader.proj_mtx, 1, gl::FALSE, self.curr_matrix.as_ptr() as _);
            gl.gl.UseProgram(self.render_data.fill_water_shader.program_id);
            gl.gl.Uniform1i(self.render_data.fill_water_shader.texture, 0);
            gl.gl.UniformMatrix4fv(
                self.render_data.fill_water_shader.proj_mtx,
                1,
                gl::FALSE,
                self.curr_matrix.as_ptr() as _,
            );
            gl.gl.UseProgram(self.render_data.tex_shader.program_id);
            gl.gl.Uniform1i(self.render_data.tex_shader.texture, 0);
            gl.gl.UniformMatrix4fv(self.render_data.tex_shader.proj_mtx, 1, gl::FALSE, self.curr_matrix.as_ptr() as _);
        }

        Ok(())
    }

    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        unsafe {
            let gl = &self.gl;
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
                null() as _,
            );

            gl.gl.BindTexture(gl::TEXTURE_2D, current_texture_id);

            let framebuffer_id = return_param(|x| gl.gl.GenFramebuffers(1, x));

            gl.gl.BindFramebuffer(gl::FRAMEBUFFER, framebuffer_id);
            gl.gl.FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture_id, 0);
            let draw_buffers = [gl::COLOR_ATTACHMENT0];
            gl.gl.DrawBuffers(1, draw_buffers.as_ptr() as _);

            gl.gl.Viewport(0, 0, width as _, height as _);
            gl.gl.ClearColor(0.0, 0.0, 0.0, 0.0);
            gl.gl.Clear(gl::COLOR_BUFFER_BIT);
            gl.gl.BindFramebuffer(gl::FRAMEBUFFER, self.render_data.render_fbo as _);

            // todo error checking: glCheckFramebufferStatus()

            Ok(Box::new(OpenGLTexture {
                texture_id,
                framebuffer_id,
                width,
                height,
                vertices: Vec::new(),
                shader: self.render_data.tex_shader,
                vbo: self.render_data.vbo,
                gl: gl.clone(),
            }))
        }
    }

    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        unsafe {
            let gl = &self.gl;
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

            Ok(Box::new(OpenGLTexture {
                texture_id,
                framebuffer_id: 0,
                width,
                height,
                vertices: Vec::new(),
                shader: self.render_data.tex_shader,
                vbo: self.render_data.vbo,
                gl: gl.clone(),
            }))
        }
    }

    fn set_blend_mode(&mut self, blend: BlendMode) -> GameResult {
        let gl = &self.gl;
        match blend {
            BlendMode::Add => unsafe {
                gl.gl.Enable(gl::BLEND);
                gl.gl.BlendEquation(gl::FUNC_ADD);
                gl.gl.BlendFunc(gl::ONE, gl::ONE);
            },
            BlendMode::Alpha => unsafe {
                gl.gl.Enable(gl::BLEND);
                gl.gl.BlendEquation(gl::FUNC_ADD);
                gl.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            },
            BlendMode::Multiply => unsafe {
                gl.gl.Enable(gl::BLEND);
                gl.gl.BlendEquation(gl::FUNC_ADD);
                gl.gl.BlendFuncSeparate(gl::ZERO, gl::SRC_COLOR, gl::ZERO, gl::SRC_ALPHA);
            },
            BlendMode::None => unsafe {
                gl.gl.Disable(gl::BLEND);
            },
        }

        Ok(())
    }

    fn set_render_target(&mut self, texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        unsafe {
            let gl = &self.gl;
            if let Some(texture) = texture {
                let gl_texture = texture
                    .as_any()
                    .downcast_ref::<OpenGLTexture>()
                    .ok_or_else(|| RenderError("This texture was not created by OpenGL backend.".to_string()))?;

                self.curr_matrix = [
                    [2.0 / (gl_texture.width as f32), 0.0, 0.0, 0.0],
                    [0.0, 2.0 / (gl_texture.height as f32), 0.0, 0.0],
                    [0.0, 0.0, -1.0, 0.0],
                    [-1.0, -1.0, 0.0, 1.0],
                ];

                gl.gl.UseProgram(self.render_data.fill_shader.program_id);
                gl.gl.UniformMatrix4fv(
                    self.render_data.fill_shader.proj_mtx,
                    1,
                    gl::FALSE,
                    self.curr_matrix.as_ptr() as _,
                );
                gl.gl.UseProgram(self.render_data.fill_water_shader.program_id);
                gl.gl.UniformMatrix4fv(
                    self.render_data.fill_water_shader.proj_mtx,
                    1,
                    gl::FALSE,
                    self.curr_matrix.as_ptr() as _,
                );
                gl.gl.UseProgram(self.render_data.tex_shader.program_id);
                gl.gl.Uniform1i(self.render_data.tex_shader.texture, 0);
                gl.gl.UniformMatrix4fv(
                    self.render_data.tex_shader.proj_mtx,
                    1,
                    gl::FALSE,
                    self.curr_matrix.as_ptr() as _,
                );

                gl.gl.BindFramebuffer(gl::FRAMEBUFFER, gl_texture.framebuffer_id);
                gl.gl.Viewport(0, 0, gl_texture.width as _, gl_texture.height as _);
            } else {
                self.curr_matrix = self.def_matrix;

                gl.gl.UseProgram(self.render_data.fill_shader.program_id);
                gl.gl.UniformMatrix4fv(
                    self.render_data.fill_shader.proj_mtx,
                    1,
                    gl::FALSE,
                    self.curr_matrix.as_ptr() as _,
                );
                gl.gl.UseProgram(self.render_data.fill_water_shader.program_id);
                gl.gl.UniformMatrix4fv(
                    self.render_data.fill_water_shader.proj_mtx,
                    1,
                    gl::FALSE,
                    self.curr_matrix.as_ptr() as _,
                );
                gl.gl.UseProgram(self.render_data.tex_shader.program_id);
                gl.gl.Uniform1i(self.render_data.tex_shader.texture, 0);
                gl.gl.UniformMatrix4fv(
                    self.render_data.tex_shader.proj_mtx,
                    1,
                    gl::FALSE,
                    self.curr_matrix.as_ptr() as _,
                );
                gl.gl.BindFramebuffer(gl::FRAMEBUFFER, self.render_data.surf_framebuffer);
                gl.gl.Viewport(0, 0, self.render_data.last_size.0 as _, self.render_data.last_size.1 as _);
            }
        }

        Ok(())
    }

    fn draw_rect(&mut self, rect: Rect<isize>, color: Color) -> GameResult {
        unsafe {
            let gl = &self.gl;
            let color = color.to_rgba();
            let mut uv = self.render_data.font_tex_size;
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

            self.render_data.fill_shader.bind_attrib_pointer(&gl, self.render_data.vbo);

            gl.gl.BindTexture(gl::TEXTURE_2D, self.render_data.font_texture);
            gl.gl.BindBuffer(gl::ARRAY_BUFFER, self.render_data.vbo);
            gl.gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * mem::size_of::<VertexData>()) as _,
                vertices.as_ptr() as _,
                gl::STREAM_DRAW,
            );

            gl.gl.DrawArrays(gl::TRIANGLES, 0, vertices.len() as _);

            gl.gl.BindTexture(gl::TEXTURE_2D, 0);
            gl.gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        Ok(())
    }

    fn draw_outline_rect(&mut self, _rect: Rect<isize>, _line_width: usize, _color: Color) -> GameResult {
        Ok(())
    }

    fn set_clip_rect(&mut self, rect: Option<Rect>) -> GameResult {
        unsafe {
            let gl = &self.gl;
            if let Some(rect) = &rect {
                gl.gl.Enable(gl::SCISSOR_TEST);
                gl.gl.Scissor(
                    rect.left as GLint,
                    self.render_data.last_size.1 as GLint - rect.bottom as GLint,
                    rect.width() as GLint,
                    rect.height() as GLint,
                );
            } else {
                gl.gl.Disable(gl::SCISSOR_TEST);
            }
        }

        Ok(())
    }

    fn imgui(&self) -> GameResult<Rc<RefCell<imgui::Context>>> {
        Ok(self.imgui.clone())
    }

    fn imgui_texture_id(&self, texture: &Box<dyn BackendTexture>) -> GameResult<TextureId> {
        let gl_texture = texture
            .as_any()
            .downcast_ref::<OpenGLTexture>()
            .ok_or_else(|| RenderError("This texture was not created by OpenGL backend.".to_string()))?;

        Ok(TextureId::new(gl_texture.texture_id as usize))
    }

    fn prepare_imgui(&mut self, _ui: &Ui) -> GameResult {
        Ok(())
    }

    fn render_imgui(&mut self, draw_data: &DrawData) -> GameResult {
        // https://github.com/michaelfairley/rust-imgui-opengl-renderer

        unsafe {
            let gl = &self.gl;
            gl.gl.ActiveTexture(gl::TEXTURE0);
            gl.gl.Enable(gl::BLEND);
            gl.gl.BlendEquation(gl::FUNC_ADD);
            gl.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl.gl.Disable(gl::CULL_FACE);
            gl.gl.Disable(gl::DEPTH_TEST);
            gl.gl.Enable(gl::SCISSOR_TEST);

            let imgui = self.imgui()?;
            let [width, height] = imgui.borrow().io().display_size;
            let [scale_w, scale_h] = imgui.borrow().io().display_framebuffer_scale;

            let fb_width = width * scale_w;
            let fb_height = height * scale_h;

            gl.gl.Viewport(0, 0, fb_width as _, fb_height as _);
            let matrix = [
                [2.0 / width as f32, 0.0, 0.0, 0.0],
                [0.0, 2.0 / -(height as f32), 0.0, 0.0],
                [0.0, 0.0, -1.0, 0.0],
                [-1.0, 1.0, 0.0, 1.0],
            ];

            gl.gl.UseProgram(self.render_data.tex_shader.program_id);
            gl.gl.Uniform1i(self.render_data.tex_shader.texture, 0);
            gl.gl.UniformMatrix4fv(self.render_data.tex_shader.proj_mtx, 1, gl::FALSE, matrix.as_ptr() as _);

            if gl.gl.BindSampler.is_loaded() {
                gl.gl.BindSampler(0, 0);
            }

            gl.gl.BindBuffer(gl::ARRAY_BUFFER, self.render_data.vbo);
            gl.gl.EnableVertexAttribArray(self.render_data.tex_shader.position);
            gl.gl.EnableVertexAttribArray(self.render_data.tex_shader.uv);
            gl.gl.EnableVertexAttribArray(self.render_data.tex_shader.color);

            gl.gl.VertexAttribPointer(
                self.render_data.tex_shader.position,
                2,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<DrawVert>() as _,
                field_offset::<DrawVert, _, _>(|v| &v.pos) as _,
            );

            gl.gl.VertexAttribPointer(
                self.render_data.tex_shader.uv,
                2,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<DrawVert>() as _,
                field_offset::<DrawVert, _, _>(|v| &v.uv) as _,
            );

            gl.gl.VertexAttribPointer(
                self.render_data.tex_shader.color,
                4,
                gl::UNSIGNED_BYTE,
                gl::TRUE,
                mem::size_of::<DrawVert>() as _,
                field_offset::<DrawVert, _, _>(|v| &v.col) as _,
            );

            for draw_list in draw_data.draw_lists() {
                let vtx_buffer = draw_list.vtx_buffer();
                let idx_buffer = draw_list.idx_buffer();

                gl.gl.BindBuffer(gl::ARRAY_BUFFER, self.render_data.vbo);
                gl.gl.BufferData(
                    gl::ARRAY_BUFFER,
                    (vtx_buffer.len() * mem::size_of::<DrawVert>()) as _,
                    vtx_buffer.as_ptr() as _,
                    gl::STREAM_DRAW,
                );

                gl.gl.BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.render_data.ebo);
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

            gl.gl.Disable(gl::SCISSOR_TEST);
        }

        Ok(())
    }

    fn draw_triangle_list(
        &mut self,
        vertices: &[VertexData],
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult<()> {
        self.draw_arrays(gl::TRIANGLES, vertices, texture, shader)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl OpenGLRenderer {
    fn draw_arrays(
        &mut self,
        vert_type: GLenum,
        vertices: &[VertexData],
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult<()> {
        if vertices.is_empty() {
            return Ok(());
        }

        let texture_id = if let Some(texture) = texture {
            let gl_texture = texture
                .as_any()
                .downcast_ref::<OpenGLTexture>()
                .ok_or_else(|| RenderError("This texture was not created by OpenGL backend.".to_string()))?;

            gl_texture.texture_id
        } else {
            0
        };

        unsafe { self.draw_arrays_tex_id(vert_type, vertices, texture_id, shader) }
    }

    unsafe fn draw_arrays_tex_id(
        &mut self,
        vert_type: GLenum,
        vertices: &[VertexData],
        mut texture: u32,
        shader: BackendShader,
    ) -> GameResult<()> {
        let gl = &self.gl;
        match shader {
            BackendShader::Fill => {
                self.render_data.fill_shader.bind_attrib_pointer(&gl, self.render_data.vbo)?;
            }
            BackendShader::Texture => {
                self.render_data.tex_shader.bind_attrib_pointer(&gl, self.render_data.vbo)?;
            }
            BackendShader::WaterFill(scale, t, frame_pos) => {
                self.render_data.fill_water_shader.bind_attrib_pointer(&gl, self.render_data.vbo)?;
                gl.gl.Uniform1f(self.render_data.fill_water_shader.scale, scale);
                gl.gl.Uniform1f(self.render_data.fill_water_shader.time, t);
                gl.gl.Uniform2f(self.render_data.fill_water_shader.frame_offset, frame_pos.0, frame_pos.1);
                texture = self.render_data.surf_texture;
            }
        }

        gl.gl.BindTexture(gl::TEXTURE_2D, texture);
        gl.gl.BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<VertexData>()) as _,
            vertices.as_ptr() as _,
            gl::STREAM_DRAW,
        );

        gl.gl.DrawArrays(vert_type, 0, vertices.len() as _);

        gl.gl.BindTexture(gl::TEXTURE_2D, 0);
        gl.gl.BindBuffer(gl::ARRAY_BUFFER, 0);

        Ok(())
    }
}

impl Drop for OpenGLRenderer {
    fn drop(&mut self) {
        self.gl.context_active.replace(false);
    }
}
