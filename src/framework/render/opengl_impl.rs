use std::any::Any;
use std::borrow::BorrowMut;
use std::cell::{Cell, OnceCell, Ref, RefCell, RefMut, UnsafeCell};
use std::ffi::{c_void, CStr};
use std::fmt::Write;
use std::hint::unreachable_unchecked;
use std::mem;
use std::mem::MaybeUninit;
use std::ptr::null;
use std::rc::Rc;
use std::sync::Arc;

use glow::{HasContext, PixelUnpackData};

use std::collections::HashMap;

use crate::common::{Color, Rect};
use crate::framework::backend::{
    BackendIndexBuffer, BackendRenderer, BackendShader, BackendTexture, BackendVertexBuffer, BufferUsage,
    PrimitiveType, VertexData,
};
use crate::framework::context::Context;
use crate::framework::error::GameError;
use crate::framework::error::GameError::RenderError;
use crate::framework::error::GameResult;
use crate::framework::graphics::{BlendMode, IndexData, ShaderStage, SwapMode};
use crate::framework::render::effect::{BackendEffect, EffectParameter, EffectTechnique};
use crate::framework::render::vertex::{VertexDeclaration, VertexElementFormat, VertexElementUsage};
use crate::framework::util::{field_offset, return_param};

type GLResult<T = ()> = Result<T, String>;

trait GLResultExt<T> {
    fn into_game_result(self) -> GameResult<T>;
}

impl<T> GLResultExt<T> for GLResult<T> {
    fn into_game_result(self) -> GameResult<T> {
        self.map_err(|mut e| {
            e.insert_str(0, "OpenGL error: ");
            GameError::RenderError(e)
        })
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GLContextType {
    /// The context type is not known yet, because it hasn't been not created or is already disposed.
    Unknown,
    /// The context is at least an OpenGL ES 3.0 context. Must be able to use #version 300 es shaders.
    GLES2,
    /// The context is at least a (Desktop) OpenGL 3.2 Core context. Must be able to use #version 150 core shaders.
    DesktopGL2,
}

fn opengl_index_size(indices: IndexData) -> u32 {
    match indices {
        IndexData::UByte(_) => glow::UNSIGNED_BYTE,
        IndexData::UShort(_) => glow::UNSIGNED_SHORT,
        IndexData::UInt(_) => glow::UNSIGNED_INT,
    }
}

pub trait GLPlatformFunctions {
    fn get_proc_address(&self, name: &str) -> *const c_void;

    fn swap_buffers(&self);

    fn set_swap_mode(&self, mode: SwapMode);

    fn get_context_type(&self) -> GLContextType;
}

pub struct OpenGLTexture {
    width: u16,
    height: u16,
    texture_id: glow::Texture,
    framebuffer_id: Option<glow::Framebuffer>,
    context_holder: GlContextHolder,
}

impl OpenGLTexture {
    fn try_dyn_ref(texture: &dyn BackendTexture) -> GameResult<&Self> {
        texture
            .as_any()
            .downcast_ref::<Self>()
            .ok_or_else(|| RenderError("This texture was not created by OpenGL backend.".to_string()))
    }
}

impl BackendTexture for OpenGLTexture {
    fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for OpenGLTexture {
    fn drop(&mut self) {
        unsafe {
            if !self.context_holder.is_context_active() {
                return;
            }

            let gl = self.context_holder.ctx_ref();

            gl.delete_texture(self.texture_id);
            if let Some(framebuffer_id) = self.framebuffer_id {
                gl.delete_framebuffer(framebuffer_id);
            }
        }
    }
}

fn check_shader_compile_status(shader: glow::Shader, gl: &glow::Context) -> GLResult {
    unsafe {
        let is_success = gl.get_shader_compile_status(shader);

        if !is_success {
            let data = gl.get_shader_info_log(shader);
            return Err(format!("Failed to compile shader {}: {}", shader.0, data));
        }
    }

    Ok(())
}

const VERTEX_SHADER_BASIC: &str = include_str!("../shaders/opengl/vertex_basic_150.glsl");
const FRAGMENT_SHADER_TEXTURED: &str = include_str!("../shaders/opengl/fragment_textured_150.glsl");
const FRAGMENT_SHADER_COLOR: &str = include_str!("../shaders/opengl/fragment_color_150.glsl");
const FRAGMENT_SHADER_WATER: &str = include_str!("../shaders/opengl/fragment_water_150.glsl");

const VERTEX_SHADER_BASIC_GLES: &str = include_str!("../shaders/opengles/vertex_basic_300.glsl");
const FRAGMENT_SHADER_TEXTURED_GLES: &str = include_str!("../shaders/opengles/fragment_textured_300.glsl");
const FRAGMENT_SHADER_COLOR_GLES: &str = include_str!("../shaders/opengles/fragment_color_300.glsl");

macro_rules! impl_rtti {
    ($name:ident, $inner_type:ty, $create_method:ident, $delete_method:ident) => {
        struct $name<'a> {
            inner: Option<$inner_type>,
            ctx: &'a glow::Context,
        }

        impl<'a> $name<'a> {
            #[inline(always)]
            fn new(ctx: &'a glow::Context) -> GLResult<Self> {
                unsafe {
                    let inner = Some(ctx.$create_method()?);
                    Ok(Self { inner, ctx })
                }
            }

            #[inline(always)]
            fn take(mut self) -> $inner_type {
                std::mem::take(&mut self.inner).unwrap()
            }
        }

        impl<'a> Drop for $name<'a> {
            #[inline(always)]
            fn drop(&mut self) {
                if let Some(inner) = std::mem::take(&mut self.inner) {
                    unsafe {
                        self.ctx.$delete_method(inner);
                    }
                }
            }
        }
    };
}

impl_rtti!(BufferRAAI, glow::Buffer, create_buffer, delete_buffer);
impl_rtti!(TextureRAAI, glow::Texture, create_texture, delete_texture);
impl_rtti!(FramebufferRAAI, glow::Framebuffer, create_framebuffer, delete_framebuffer);

struct RenderShaderObject {
    shader: glow::Shader,
    stage: ShaderStage,
    context_holder: GlContextHolder,
}

impl RenderShaderObject {
    fn new(context: &GlContextHolder, stage: ShaderStage, source: &str) -> GLResult<Rc<RenderShaderObject>> {
        let gl = context.ctx_ref();

        unsafe {
            let shader = gl.create_shader(match stage {
                ShaderStage::Vertex => glow::VERTEX_SHADER,
                ShaderStage::Fragment => glow::FRAGMENT_SHADER,
            })?;

            gl.shader_source(shader, source);
            gl.compile_shader(shader);
            match check_shader_compile_status(shader, gl) {
                Ok(()) => Ok(Rc::new(RenderShaderObject { shader, stage, context_holder: context.clone() })),
                Err(e) => {
                    gl.delete_shader(shader);
                    Err(e)
                }
            }
        }
    }
}

impl Drop for RenderShaderObject {
    fn drop(&mut self) {
        if !self.context_holder.is_context_active() {
            return;
        }

        unsafe {
            let gl = self.context_holder.ctx_ref();
            gl.delete_shader(self.shader);
        }
    }
}

struct RenderShader {
    name: String,
    program_id: Option<glow::Program>,
    vertex_shader: Rc<RenderShaderObject>,
    fragment_shader: Rc<RenderShaderObject>,
    texture: Option<glow::UniformLocation>,
    proj_mtx: Option<glow::UniformLocation>,
    scale: Option<glow::UniformLocation>,
    time: Option<glow::UniformLocation>,
    frame_offset: Option<glow::UniformLocation>,
    position: Option<u32>,
    uv: Option<u32>,
    color: Option<u32>,
    context_holder: GlContextHolder,
}

impl RenderShader {
    fn compile(
        context: &GlContextHolder,
        vertex_shader: Rc<RenderShaderObject>,
        fragment_shader: Rc<RenderShaderObject>,
        name: String,
    ) -> GLResult<Rc<RenderShader>> {
        unsafe {
            let mut shader = RenderShader {
                name,
                program_id: None,
                vertex_shader,
                fragment_shader,
                texture: None,
                proj_mtx: None,
                scale: None,
                time: None,
                frame_offset: None,
                position: None,
                uv: None,
                color: None,
                context_holder: context.clone(),
            };

            let gl = context.ctx_ref();

            let program_id = gl.create_program()?;
            shader.program_id = Some(program_id);
            gl.attach_shader(program_id, shader.vertex_shader.shader);
            gl.attach_shader(program_id, shader.fragment_shader.shader);
            gl.link_program(program_id);
            // TODO: Error check?
            log::debug!("Linked shader '{0}', program ID: {program_id:?}", shader.name);

            shader.texture = gl.get_uniform_location(program_id, "Texture");
            shader.proj_mtx = gl.get_uniform_location(program_id, "ProjMtx");
            shader.scale = gl.get_uniform_location(program_id, "Scale");
            shader.time = gl.get_uniform_location(program_id, "Time");
            shader.frame_offset = gl.get_uniform_location(program_id, "FrameOffset");
            shader.position = gl.get_attrib_location(program_id, "Position");
            shader.uv = gl.get_attrib_location(program_id, "UV");
            shader.color = gl.get_attrib_location(program_id, "Color");
            Ok(Rc::new(shader))
        }
    }

    unsafe fn bind_attrib_pointer(&self, gl: &glow::Context) -> GLResult {
        if let None = self.program_id {
            return Err(String::from("Cannot bind attribute pointers without a shader program."));
        }

        gl.use_program(self.program_id);
        if let Some(position) = self.position {
            gl.enable_vertex_attrib_array(position);
            gl.vertex_attrib_pointer_f32(
                position,
                2,
                glow::FLOAT,
                false,
                mem::size_of::<VertexData>() as _,
                mem::offset_of!(VertexData, position) as _,
            );
        }
        if let Some(uv) = self.uv {
            gl.enable_vertex_attrib_array(uv);
            gl.vertex_attrib_pointer_f32(
                uv,
                2,
                glow::FLOAT,
                false,
                mem::size_of::<VertexData>() as _,
                mem::offset_of!(VertexData, uv) as _,
            );
        }
        if let Some(color) = self.color {
            gl.enable_vertex_attrib_array(color);
            gl.vertex_attrib_pointer_f32(
                color,
                4,
                glow::UNSIGNED_BYTE,
                true,
                mem::size_of::<VertexData>() as _,
                mem::offset_of!(VertexData, color) as _,
            );
        }
        check_gl_errors("bind_attrib_pointer", &gl);

        Ok(())
    }
}

impl Drop for RenderShader {
    fn drop(&mut self) {
        if !self.context_holder.is_context_active() {
            return;
        }

        let gl = self.context_holder.ctx_ref();
        unsafe {
            if let Some(program_id) = self.program_id {
                gl.delete_program(program_id);
            }
            self.program_id = None;
        }
    }
}

struct RenderData {
    tex_shader: Rc<RenderShader>,
    fill_shader: Rc<RenderShader>,
    fill_water_shader: Rc<RenderShader>,
    render_fbo: Option<glow::Framebuffer>,
    vao: Option<glow::VertexArray>,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    surf_framebuffer: glow::Framebuffer,
    surf_texture: glow::Texture,
    last_size: (u32, u32),
}

impl RenderData {
    fn new(context: GlContextHolder) -> GLResult<Self> {
        let gles2_mode = context.ctx_ref().version().is_embedded;

        let vshdr_basic = if gles2_mode { VERTEX_SHADER_BASIC_GLES } else { VERTEX_SHADER_BASIC };
        let fshdr_tex = if gles2_mode { FRAGMENT_SHADER_TEXTURED_GLES } else { FRAGMENT_SHADER_TEXTURED };
        let fshdr_fill = if gles2_mode { FRAGMENT_SHADER_COLOR_GLES } else { FRAGMENT_SHADER_COLOR };
        let fshdr_fill_water = if gles2_mode { FRAGMENT_SHADER_COLOR_GLES } else { FRAGMENT_SHADER_WATER };

        unsafe {
            let gl = context.ctx_ref();

            // iOS has "unusual" framebuffer setup, where we can't rely on 0 as the system provided render target.
            let render_fbo = gl.get_parameter_framebuffer(glow::FRAMEBUFFER_BINDING);

            let vshdr_basic = RenderShaderObject::new(&context, ShaderStage::Vertex, vshdr_basic)?;
            let fshdr_tex = RenderShaderObject::new(&context, ShaderStage::Fragment, fshdr_tex)?;
            let fshdr_fill = RenderShaderObject::new(&context, ShaderStage::Fragment, fshdr_fill)?;
            let fshdr_fill_water = RenderShaderObject::new(&context, ShaderStage::Fragment, fshdr_fill_water)?;

            let vao = gl.create_vertex_array().ok();
            if let Some(vao) = vao {
                gl.bind_vertex_array(Some(vao));
            }

            let mut vbo = BufferRAAI::new(gl)?;
            let mut ebo = BufferRAAI::new(gl)?;
            let mut surf_texture = TextureRAAI::new(gl)?;
            let mut surf_framebuffer = FramebufferRAAI::new(gl)?;

            let tex_shader =
                RenderShader::compile(&context, vshdr_basic.clone(), fshdr_tex, "builtin texture".to_owned())?;
            let fill_shader =
                RenderShader::compile(&context, vshdr_basic.clone(), fshdr_fill, "builtin fill".to_owned())?;
            let fill_water_shader =
                RenderShader::compile(&context, vshdr_basic.clone(), fshdr_fill_water, "builtin water".to_owned())?;

            let vbo = vbo.take();
            let ebo = ebo.take();
            let surf_texture = surf_texture.take();
            let surf_framebuffer = surf_framebuffer.take();

            gl.bind_texture(glow::TEXTURE_2D, Some(surf_texture));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as _);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as _);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as _);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as _);

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as _,
                320,
                240,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );

            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(surf_framebuffer));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(surf_texture),
                0,
            );

            gl.draw_buffers(&[glow::COLOR_ATTACHMENT0]);
            OpenGLRenderer::check_framebuffer_status(&gl);

            Ok(RenderData {
                tex_shader,
                fill_shader,
                fill_water_shader,
                render_fbo,
                vao,
                vbo,
                ebo,
                surf_framebuffer,
                surf_texture,
                last_size: (320, 240),
            })
        }
    }
}

#[derive(Clone)]
pub struct GlContextHolder {
    context: Rc<glow::Context>,
    context_active: Rc<RefCell<bool>>,
}

impl GlContextHolder {
    pub fn new(context: Rc<glow::Context>) -> GlContextHolder {
        GlContextHolder { context, context_active: Rc::new(RefCell::new(true)) }
    }

    #[inline(always)]
    pub fn ctx(&self) -> Rc<glow::Context> {
        self.context.clone()
    }

    #[inline(always)]
    pub fn ctx_ref(&self) -> &glow::Context {
        &self.context
    }

    pub(crate) fn is_context_active(&self) -> bool {
        *self.context_active.borrow()
    }

    pub(crate) fn renderer_dropped(&self) {
        self.context_active.replace(false);
    }
}

pub struct OpenGLRenderer {
    platform: RefCell<Box<dyn GLPlatformFunctions>>,
    gl: OnceCell<GlContextHolder>,
    render_data: RefCell<Option<RenderData>>,
    def_matrix: [[f32; 4]; 4],
    curr_matrix: [[f32; 4]; 4],
}

impl OpenGLRenderer {
    pub fn new(platform: Box<dyn GLPlatformFunctions>) -> OpenGLRenderer {
        OpenGLRenderer {
            platform: RefCell::new(platform),
            gl: OnceCell::new(),
            render_data: RefCell::new(None),
            def_matrix: [[0.0; 4]; 4],
            curr_matrix: [[0.0; 4]; 4],
        }
    }

    fn get_context_holder(&self) -> &GlContextHolder {
        self.gl.get_or_init(|| {
            let gl_context = {
                let platform = self.platform.borrow();
                let mut context = unsafe { glow::Context::from_loader_function(|ptr| platform.get_proc_address(ptr)) };
                {
                    let glow::Version { major, minor, is_embedded, vendor_info, .. } = context.version();
                    let emb = if *is_embedded { " ES" } else { "" };
                    log::info!("OpenGL{emb} version {major}.{minor} ({vendor_info})");
                }
                OpenGLRenderer::enable_debug_output(&mut context);
                Rc::new(context)
            };
            GlContextHolder::new(gl_context)
        })
    }

    #[inline]
    fn get_context(&self) -> &glow::Context {
        self.get_context_holder().ctx_ref()
    }

    fn get_render_data(&self) -> GameResult<RefMut<'_, RenderData>> {
        let needs_init = self.render_data.borrow().is_none();
        if needs_init {
            let context = self.get_context_holder().clone();
            let render_data = RenderData::new(context).into_game_result()?;
            self.render_data.borrow_mut().replace(render_data);
        }

        Ok(RefMut::map(self.render_data.borrow_mut(), |f| unsafe { f.as_mut().unwrap_unchecked() }))
    }
}

impl BackendRenderer for OpenGLRenderer {
    fn renderer_name(&self) -> String {
        let context = self.get_context();
        let version = context.version();
        let mut s = String::with_capacity(128);
        s.push_str("OpenGL ");
        if version.is_embedded {
            s.push_str("ES ");
        }
        write!(s, "{}.{}", version.major, version.minor);
        if let Some(revision) = version.revision {
            write!(s, ".{}", revision);
        }
        s.push(' ');
        s.push_str(&version.vendor_info);
        s
    }

    fn clear(&mut self, color: Color) {
        let gl = self.get_context();
        unsafe {
            gl.clear_color(color.r, color.g, color.b, color.a);
            gl.clear(glow::COLOR_BUFFER_BIT);
            check_gl_errors("clear", &gl);
        }
    }

    fn present(&mut self) -> GameResult {
        unsafe {
            let gl = self.get_context();

            let (surf_texture) = {
                let render_data = self.get_render_data()?;
                gl.bind_framebuffer(glow::FRAMEBUFFER, render_data.render_fbo);
                gl.clear_color(0.0, 0.0, 0.0, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

                let matrix =
                    [[2.0f32, 0.0, 0.0, 0.0], [0.0, -2.0, 0.0, 0.0], [0.0, 0.0, -1.0, 0.0], [-1.0, 1.0, 0.0, 1.0]];

                gl.use_program(render_data.tex_shader.program_id);
                gl.uniform_matrix_4_f32_slice(render_data.tex_shader.proj_mtx.as_ref(), false, matrix.as_flattened());

                (render_data.surf_texture)
            };

            let color = (255, 255, 255, 255);
            let vertices = [
                VertexData { position: (0.0, 1.0), uv: (0.0, 0.0), color },
                VertexData { position: (0.0, 0.0), uv: (0.0, 1.0), color },
                VertexData { position: (1.0, 0.0), uv: (1.0, 1.0), color },
                VertexData { position: (1.0, 1.0), uv: (1.0, 0.0), color },
            ];
            let indices = [0u8, 1, 2, 0, 2, 3];

            self.draw_immediate_tex_id(
                glow::TRIANGLES,
                &vertices,
                Some(IndexData::UByte(&indices)),
                Some(surf_texture),
                BackendShader::Texture,
                0,
            )?;
            check_gl_errors("present", &gl);

            self.platform.borrow().swap_buffers();
        }

        Ok(())
    }

    fn set_swap_mode(&mut self, mode: SwapMode) -> GameResult {
        self.platform.borrow().set_swap_mode(mode);
        Ok(())
    }

    fn prepare_draw(&mut self, width: f32, height: f32) -> GameResult {
        self.def_matrix = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, 2.0 / -height, 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];
        self.curr_matrix = self.def_matrix;

        let gl = self.get_context();

        unsafe {
            let mut render_data = self.get_render_data()?;
            let (width_u, height_u) = (width as u32, height as u32);
            if render_data.last_size != (width_u, height_u) {
                render_data.last_size = (width_u, height_u);
                gl.bind_framebuffer(glow::FRAMEBUFFER, render_data.render_fbo);
                gl.bind_texture(glow::TEXTURE_2D, Some(render_data.surf_texture));

                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA as _,
                    width_u as _,
                    height_u as _,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    PixelUnpackData::Slice(None),
                );

                gl.bind_texture(glow::TEXTURE_2D, None);
            }

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(render_data.surf_framebuffer));

            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.clear(glow::COLOR_BUFFER_BIT);

            gl.active_texture(glow::TEXTURE0);
            gl.blend_equation(glow::FUNC_ADD);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            gl.viewport(0, 0, width_u as _, height_u as _);

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

            gl.use_program(render_data.fill_shader.program_id);
            gl.uniform_matrix_4_f32_slice(
                render_data.fill_shader.proj_mtx.as_ref(),
                false,
                self.curr_matrix.as_flattened(),
            );
            gl.use_program(render_data.fill_water_shader.program_id);
            gl.uniform_1_i32(render_data.fill_water_shader.texture.as_ref(), 0);
            gl.uniform_matrix_4_f32_slice(
                render_data.fill_water_shader.proj_mtx.as_ref(),
                false,
                self.curr_matrix.as_flattened(),
            );
            gl.use_program(render_data.tex_shader.program_id);
            gl.uniform_1_i32(render_data.tex_shader.texture.as_ref(), 0);
            gl.uniform_matrix_4_f32_slice(
                render_data.tex_shader.proj_mtx.as_ref(),
                false,
                self.curr_matrix.as_flattened(),
            );
        }

        check_gl_errors("prepare_draw", &gl);

        Ok(())
    }

    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        let gl = self.get_context();
        unsafe {
            let current_texture_id = gl.get_parameter_texture(glow::TEXTURE_BINDING_2D);
            let current_framebuffer_id = gl.get_parameter_framebuffer(glow::FRAMEBUFFER_BINDING);

            let texture_id = TextureRAAI::new(&gl).into_game_result()?;
            let framebuffer_id = FramebufferRAAI::new(&gl).into_game_result()?;

            let texture_id = texture_id.take();
            let framebuffer_id = framebuffer_id.take();

            gl.bind_texture(glow::TEXTURE_2D, Some(texture_id));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as _);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as _);

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as _,
                width as _,
                height as _,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                PixelUnpackData::Slice(None),
            );

            gl.bind_texture(glow::TEXTURE_2D, current_texture_id);
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer_id));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(texture_id),
                0,
            );
            gl.draw_buffers(&[glow::COLOR_ATTACHMENT0]);

            gl.viewport(0, 0, width as _, height as _);
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
            OpenGLRenderer::check_framebuffer_status(&gl);

            gl.bind_framebuffer(glow::FRAMEBUFFER, current_framebuffer_id);

            check_gl_errors("create_texture_mutable", &gl);

            Ok(Box::new(OpenGLTexture {
                texture_id,
                framebuffer_id: Some(framebuffer_id),
                width,
                height,
                context_holder: self.get_context_holder().clone(),
            }))
        }
    }

    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        let gl = self.get_context();
        unsafe {
            let current_texture_id = gl.get_parameter_texture(glow::TEXTURE_BINDING_2D);

            let texture_id = TextureRAAI::new(&gl).into_game_result()?;
            let texture_id = texture_id.take();

            gl.bind_texture(glow::TEXTURE_2D, Some(texture_id));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as _);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as _);

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as _,
                width as _,
                height as _,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                PixelUnpackData::Slice(Some(data)),
            );

            gl.bind_texture(glow::TEXTURE_2D, current_texture_id);

            check_gl_errors("create_texture", &gl);

            Ok(Box::new(OpenGLTexture {
                texture_id,
                framebuffer_id: None,
                width,
                height,
                context_holder: self.get_context_holder().clone(),
            }))
        }
    }

    fn set_blend_mode(&mut self, blend: BlendMode) -> GameResult {
        let gl = self.get_context();
        match blend {
            BlendMode::Add => unsafe {
                gl.enable(glow::BLEND);
                gl.blend_equation(glow::FUNC_ADD);
                gl.blend_func(glow::ONE, glow::ONE);
            },
            BlendMode::Alpha => unsafe {
                gl.enable(glow::BLEND);
                gl.blend_equation(glow::FUNC_ADD);
                gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            },
            BlendMode::Multiply => unsafe {
                gl.enable(glow::BLEND);
                gl.blend_equation(glow::FUNC_ADD);
                gl.blend_func_separate(glow::ZERO, glow::SRC_COLOR, glow::ZERO, glow::SRC_ALPHA);
            },
            BlendMode::None => unsafe {
                gl.disable(glow::BLEND);
            },
        }

        check_gl_errors("set_blend_mode", &gl);

        Ok(())
    }

    fn set_render_target(&mut self, texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        unsafe {
            if let Some(texture) = texture {
                let gl_texture = OpenGLTexture::try_dyn_ref(texture.as_ref())?;

                self.curr_matrix = [
                    [2.0 / (gl_texture.width as f32), 0.0, 0.0, 0.0],
                    [0.0, 2.0 / (gl_texture.height as f32), 0.0, 0.0],
                    [0.0, 0.0, -1.0, 0.0],
                    [-1.0, -1.0, 0.0, 1.0],
                ];

                let gl = self.get_context();
                let render_data = self.get_render_data()?;

                gl.use_program(render_data.fill_shader.program_id);
                gl.uniform_matrix_4_f32_slice(
                    render_data.fill_shader.proj_mtx.as_ref(),
                    false,
                    self.curr_matrix.as_flattened(),
                );
                gl.use_program(render_data.fill_water_shader.program_id);
                gl.uniform_matrix_4_f32_slice(
                    render_data.fill_water_shader.proj_mtx.as_ref(),
                    false,
                    self.curr_matrix.as_flattened(),
                );
                gl.use_program(render_data.tex_shader.program_id);
                gl.uniform_1_i32(render_data.tex_shader.texture.as_ref(), 0);
                gl.uniform_matrix_4_f32_slice(
                    render_data.tex_shader.proj_mtx.as_ref(),
                    false,
                    self.curr_matrix.as_flattened(),
                );

                gl.bind_framebuffer(glow::FRAMEBUFFER, gl_texture.framebuffer_id);
                gl.viewport(0, 0, gl_texture.width as _, gl_texture.height as _);
            } else {
                self.curr_matrix = self.def_matrix;

                let gl = self.get_context();
                let render_data = self.get_render_data()?;

                gl.use_program(render_data.fill_shader.program_id);
                gl.uniform_matrix_4_f32_slice(
                    render_data.fill_shader.proj_mtx.as_ref(),
                    false,
                    self.curr_matrix.as_flattened(),
                );
                gl.use_program(render_data.fill_water_shader.program_id);
                gl.uniform_matrix_4_f32_slice(
                    render_data.fill_water_shader.proj_mtx.as_ref(),
                    false,
                    self.curr_matrix.as_flattened(),
                );
                gl.use_program(render_data.tex_shader.program_id);
                gl.uniform_1_i32(render_data.tex_shader.texture.as_ref(), 0);
                gl.uniform_matrix_4_f32_slice(
                    render_data.tex_shader.proj_mtx.as_ref(),
                    false,
                    self.curr_matrix.as_flattened(),
                );
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(render_data.surf_framebuffer));
                gl.viewport(0, 0, render_data.last_size.0 as _, render_data.last_size.1 as _);
            }
        }

        check_gl_errors("set_render_target", &self.get_context());

        Ok(())
    }

    fn draw_rect(&mut self, rect: Rect<isize>, color: Color) -> GameResult {
        unsafe {
            let gl = self.get_context();
            let color = color.to_rgba();
            let mut uv = (0.0, 0.0);

            let vertices = [
                VertexData { position: (rect.left as _, rect.bottom as _), uv, color },
                VertexData { position: (rect.left as _, rect.top as _), uv, color },
                VertexData { position: (rect.right as _, rect.top as _), uv, color },
                VertexData { position: (rect.right as _, rect.bottom as _), uv, color },
            ];
            let indices = [0, 1, 2, 0, 2, 3];

            self.draw_immediate_tex_id(
                glow::TRIANGLES,
                &vertices,
                Some(IndexData::UByte(&indices)),
                None,
                BackendShader::Fill,
                0,
            )?;

            check_gl_errors("draw_rect", &gl);

            Ok(())
        }
    }

    fn draw_outline_rect(&mut self, _rect: Rect<isize>, _line_width: usize, _color: Color) -> GameResult {
        Ok(())
    }

    fn set_clip_rect(&mut self, rect: Option<Rect>) -> GameResult {
        let gl = self.get_context();
        let render_data = self.get_render_data()?;
        unsafe {
            if let Some(rect) = &rect {
                gl.enable(glow::SCISSOR_TEST);
                gl.scissor(
                    rect.left as i32,
                    render_data.last_size.1 as i32 - rect.bottom as i32,
                    rect.width() as i32,
                    rect.height() as i32,
                );
            } else {
                gl.disable(glow::SCISSOR_TEST);
            }
        }

        check_gl_errors("set_clip_rect", &gl);

        Ok(())
    }

    fn draw_triangles(
        &mut self,
        vertices: &[VertexData],
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult {
        self.draw_arrays(glow::TRIANGLES, vertices, texture, shader, 0)
    }

    fn draw_triangles_indexed(
        &mut self,
        vertices: &[VertexData],
        indices: IndexData,
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult {
        self.draw_elements(glow::TRIANGLES, vertices, indices, texture, shader, 0)
    }

    fn create_vertex_buffer(
        &mut self,
        decl: VertexDeclaration,
        count: usize,
        usage: BufferUsage,
    ) -> GameResult<Box<dyn BackendVertexBuffer>> {
        let context = self.get_context_holder().clone();
        let gl = context.ctx_ref();
        unsafe {
            let buffer_id = gl.create_buffer().map_err(|e| RenderError(e))?;
            let byte_size = count * decl.stride as usize;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer_id));
            gl.buffer_data_size(glow::ARRAY_BUFFER, byte_size as i32, OpenGLVertexBuffer::gl_usage(usage));
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            Ok(Box::new(OpenGLVertexBuffer {
                buffer_id,
                vertex_count: count,
                stride: decl.stride as usize,
                usage,
                context_holder: context,
            }))
        }
    }

    fn create_index_buffer(&mut self, count: usize, usage: BufferUsage) -> GameResult<Box<dyn BackendIndexBuffer>> {
        let context = self.get_context_holder().clone();
        let gl = context.ctx_ref();
        unsafe {
            let buffer_id = gl.create_buffer().map_err(|e| RenderError(e))?;
            // Allocate for u16 by default; will be resized on set_data if needed
            let byte_size = count * 2;
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(buffer_id));
            gl.buffer_data_size(glow::ELEMENT_ARRAY_BUFFER, byte_size as i32, OpenGLVertexBuffer::gl_usage(usage));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

            Ok(Box::new(OpenGLIndexBuffer {
                buffer_id,
                index_count: count,
                index_type: glow::UNSIGNED_SHORT,
                usage,
                context_holder: context,
            }))
        }
    }

    fn create_effect_from_glsl(
        &mut self,
        vertex_src: &str,
        fragment_src: &str,
        name: &str,
    ) -> GameResult<Box<dyn BackendEffect>> {
        let context = self.get_context_holder().clone();
        let gl = context.ctx_ref();

        unsafe {
            let vs = gl.create_shader(glow::VERTEX_SHADER).map_err(|e| RenderError(e))?;
            gl.shader_source(vs, vertex_src);
            gl.compile_shader(vs);
            if let Err(e) = check_shader_compile_status(vs, gl) {
                gl.delete_shader(vs);
                return Err(RenderError(e));
            }

            let fs = gl.create_shader(glow::FRAGMENT_SHADER).map_err(|e| RenderError(e))?;
            gl.shader_source(fs, fragment_src);
            gl.compile_shader(fs);
            if let Err(e) = check_shader_compile_status(fs, gl) {
                gl.delete_shader(vs);
                gl.delete_shader(fs);
                return Err(RenderError(e));
            }

            let program_id = gl.create_program().map_err(|e| {
                gl.delete_shader(vs);
                gl.delete_shader(fs);
                RenderError(e)
            })?;
            gl.attach_shader(program_id, vs);
            gl.attach_shader(program_id, fs);
            gl.link_program(program_id);

            // Shaders can be deleted after linking
            gl.delete_shader(vs);
            gl.delete_shader(fs);

            // Enumerate all active uniforms as parameters
            let uniform_count = gl.get_active_uniforms(program_id);
            let mut parameters = Vec::new();
            for i in 0..uniform_count {
                if let Some(uniform) = gl.get_active_uniform(program_id, i) {
                    if let Some(location) = gl.get_uniform_location(program_id, &uniform.name) {
                        parameters.push(OpenGLEffectParameter {
                            name: uniform.name.clone(),
                            location,
                            value: None,
                            dirty: false,
                        });
                    }
                }
            }

            let param_indices: HashMap<String, usize> =
                parameters.iter().enumerate().map(|(i, p)| (p.name.clone(), i)).collect();

            let technique = OpenGLEffectTechnique { name: name.to_string(), program_id, parameters: param_indices };

            Ok(Box::new(OpenGLEffect {
                techniques: vec![technique],
                parameters,
                current_technique: 0,
                context_holder: context,
            }))
        }
    }

    fn scene_texture(&self) -> Option<&dyn BackendTexture> {
        // TODO: wrap surf_texture as a BackendTexture and return it
        None
    }

    fn draw_primitives(
        &mut self,
        effect: &mut dyn BackendEffect,
        vb: &dyn BackendVertexBuffer,
        decl: &VertexDeclaration,
        prim_type: PrimitiveType,
        start: usize,
        count: usize,
    ) -> GameResult {
        let gl_vb = vb
            .as_any()
            .downcast_ref::<OpenGLVertexBuffer>()
            .ok_or_else(|| RenderError("Vertex buffer was not created by OpenGL backend.".to_string()))?;
        let gl_effect = effect
            .as_any()
            .downcast_ref::<OpenGLEffect>()
            .ok_or_else(|| RenderError("Effect was not created by OpenGL backend.".to_string()))?;

        let gl = self.get_context();
        let technique = &gl_effect.techniques[gl_effect.current_technique];

        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(gl_vb.buffer_id));
            bind_vertex_declaration(&gl, technique.program_id, decl);

            let vertex_count = primitive_count_to_vertex_count(prim_type, count);
            gl.draw_arrays(gl_primitive_type(prim_type), start as i32, vertex_count as i32);

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
        Ok(())
    }

    fn draw_indexed_primitives(
        &mut self,
        effect: &mut dyn BackendEffect,
        vb: &dyn BackendVertexBuffer,
        ib: &dyn BackendIndexBuffer,
        decl: &VertexDeclaration,
        prim_type: PrimitiveType,
        start_index: usize,
        count: usize,
    ) -> GameResult {
        let gl_vb = vb
            .as_any()
            .downcast_ref::<OpenGLVertexBuffer>()
            .ok_or_else(|| RenderError("Vertex buffer was not created by OpenGL backend.".to_string()))?;
        let gl_ib = ib
            .as_any()
            .downcast_ref::<OpenGLIndexBuffer>()
            .ok_or_else(|| RenderError("Index buffer was not created by OpenGL backend.".to_string()))?;
        let gl_effect = effect
            .as_any()
            .downcast_ref::<OpenGLEffect>()
            .ok_or_else(|| RenderError("Effect was not created by OpenGL backend.".to_string()))?;

        let gl = self.get_context();
        let technique = &gl_effect.techniques[gl_effect.current_technique];

        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(gl_vb.buffer_id));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(gl_ib.buffer_id));
            bind_vertex_declaration(&gl, technique.program_id, decl);

            let index_count = primitive_count_to_vertex_count(prim_type, count);
            let byte_offset = start_index
                * match gl_ib.index_type {
                    glow::UNSIGNED_BYTE => 1,
                    glow::UNSIGNED_SHORT => 2,
                    glow::UNSIGNED_INT => 4,
                    _ => 2,
                };
            gl.draw_elements(gl_primitive_type(prim_type), index_count as i32, gl_ib.index_type, byte_offset as i32);

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl OpenGLRenderer {
    fn enable_debug_output(gl: &mut glow::Context) {
        #[cfg(debug_assertions)]
        unsafe {
            if gl.supports_debug() {
                gl.enable(glow::DEBUG_OUTPUT);
                gl.enable(glow::DEBUG_OUTPUT_SYNCHRONOUS);
                gl.debug_message_callback(|source, type_, id, severity, message| {
                    let type_str = match type_ {
                        glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DEPRECATED_BEHAVIOR",
                        glow::DEBUG_TYPE_ERROR => "ERROR",
                        glow::DEBUG_TYPE_MARKER => "MARKER",
                        glow::DEBUG_TYPE_OTHER => "OTHER",
                        glow::DEBUG_TYPE_PERFORMANCE => "PERFORMANCE",
                        glow::DEBUG_TYPE_POP_GROUP => "POP_GROUP",
                        glow::DEBUG_TYPE_PORTABILITY => "PORTABILITY",
                        glow::DEBUG_TYPE_PUSH_GROUP => "PUSH_GROUP",
                        glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "UNDEFINED_BEHAVIOR",
                        _ => "UNKNOWN",
                    };

                    if severity == glow::DEBUG_SEVERITY_NOTIFICATION {
                        return; // too spammy
                    }

                    let severity_str = match severity {
                        glow::DEBUG_SEVERITY_NOTIFICATION => "NOTIFICATION",
                        glow::DEBUG_SEVERITY_HIGH => "HIGH",
                        glow::DEBUG_SEVERITY_MEDIUM => "MEDIUM",
                        glow::DEBUG_SEVERITY_LOW => "LOW",
                        _ => "UNKNOWN",
                    };
                    log::debug!("GLDebugOutput(type={type_str}, id={id}, severity={severity_str}): {message}");
                });
            }
        }
    }

    fn check_framebuffer_status(gl: &glow::Context) {
        unsafe {
            let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
            let status_str = match status {
                glow::FRAMEBUFFER_COMPLETE => return,
                glow::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "FRAMEBUFFER_INCOMPLETE_ATTACHMENT",
                glow::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => "FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT",
                glow::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => "FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER",
                glow::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => "FRAMEBUFFER_INCOMPLETE_READ_BUFFER",
                glow::FRAMEBUFFER_UNSUPPORTED => "FRAMEBUFFER_UNSUPPORTED",
                _ => "UNKNOWN",
            };
            log::warn!("Framebuffer status: {:#x} ({})", status, status_str);
        }
    }

    fn draw_arrays(
        &mut self,
        vert_type: u32,
        vertices: &[VertexData],
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
        first_vertex: u32,
    ) -> GameResult<()> {
        if vertices.is_empty() {
            return Ok(());
        }

        let texture_id = if let Some(texture) = texture {
            let gl_texture = OpenGLTexture::try_dyn_ref(texture.as_ref())?;

            Some(gl_texture.texture_id)
        } else {
            None
        };

        unsafe { self.draw_immediate_tex_id(vert_type, vertices, None, texture_id, shader, first_vertex) }
    }

    fn draw_elements(
        &mut self,
        vert_type: u32,
        vertices: &[VertexData],
        indices: IndexData,
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
        first_index: u32,
    ) -> GameResult<()> {
        if vertices.is_empty() || indices.is_empty() {
            return Ok(());
        }

        let texture_id = if let Some(texture) = texture {
            let gl_texture = OpenGLTexture::try_dyn_ref(texture.as_ref())?;
            Some(gl_texture.texture_id)
        } else {
            None
        };

        unsafe { self.draw_immediate_tex_id(vert_type, vertices, Some(indices), texture_id, shader, first_index) }
    }

    unsafe fn draw_immediate_tex_id(
        &self,
        vert_type: u32,
        vertices: &[VertexData],
        indices: Option<IndexData>,
        mut texture: Option<glow::Texture>,
        shader: BackendShader,
        first: u32,
    ) -> GameResult<()> {
        let gl = self.get_context();
        let render_data = self.get_render_data()?;

        // Upload vertex data before setting up attrib pointers, as macOS GL validates
        // offsets against the current buffer size in glVertexAttribPointer.
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(render_data.vbo));
        let vertices_slice =
            std::slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * mem::size_of::<VertexData>());
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_slice, glow::STREAM_DRAW);

        match shader {
            BackendShader::Fill => {
                render_data.fill_shader.bind_attrib_pointer(&gl).into_game_result()?;
            }
            BackendShader::Texture => {
                render_data.tex_shader.bind_attrib_pointer(&gl).into_game_result()?;
            }
            BackendShader::WaterFill(scale, t, frame_pos) => {
                render_data.fill_water_shader.bind_attrib_pointer(&gl).into_game_result()?;
                gl.uniform_1_f32(render_data.fill_water_shader.scale.as_ref(), scale);
                gl.uniform_1_f32(render_data.fill_water_shader.time.as_ref(), t);
                gl.uniform_2_f32(render_data.fill_water_shader.frame_offset.as_ref(), frame_pos.0, frame_pos.1);
                texture = Some(render_data.surf_texture);
            }
        }

        gl.bind_texture(glow::TEXTURE_2D, texture);

        if let Some(indices) = indices {
            let index_slice = indices.as_bytes_slice();
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(render_data.ebo));

            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices.as_bytes_slice(), glow::STREAM_DRAW);

            gl.draw_elements(vert_type, indices.len() as _, opengl_index_size(indices), (first as usize) as _);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        } else {
            gl.draw_arrays(vert_type, first as _, vertices.len() as _);
        }

        gl.bind_texture(glow::TEXTURE_2D, None);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);

        Ok(())
    }
}

pub struct OpenGLVertexBuffer {
    buffer_id: glow::Buffer,
    vertex_count: usize,
    stride: usize,
    usage: BufferUsage,
    context_holder: GlContextHolder,
}

impl OpenGLVertexBuffer {
    fn gl_usage(usage: BufferUsage) -> u32 {
        match usage {
            BufferUsage::Static => glow::STATIC_DRAW,
            BufferUsage::Dynamic => glow::DYNAMIC_DRAW,
            BufferUsage::Stream => glow::STREAM_DRAW,
        }
    }
}

impl BackendVertexBuffer for OpenGLVertexBuffer {
    fn set_data_raw(&mut self, data: &[u8], offset: usize) -> GameResult {
        let gl = self.context_holder.ctx_ref();
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.buffer_id));
            if offset == 0 {
                // Full re-upload: orphan and reallocate. This is the idiomatic
                // path for stream buffers and lets the driver avoid sync stalls.
                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, data, Self::gl_usage(self.usage));
            } else {
                gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, offset as i32, data);
            }
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
        self.vertex_count = data.len() / self.stride;
        Ok(())
    }

    fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for OpenGLVertexBuffer {
    fn drop(&mut self) {
        if !self.context_holder.is_context_active() {
            return;
        }
        unsafe {
            self.context_holder.ctx_ref().delete_buffer(self.buffer_id);
        }
    }
}

pub struct OpenGLIndexBuffer {
    buffer_id: glow::Buffer,
    index_count: usize,
    index_type: u32,
    usage: BufferUsage,
    context_holder: GlContextHolder,
}

impl BackendIndexBuffer for OpenGLIndexBuffer {
    fn set_data(&mut self, data: IndexData, offset: usize) -> GameResult {
        let gl = self.context_holder.ctx_ref();
        self.index_type = opengl_index_size(data);
        self.index_count = data.len();
        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.buffer_id));
            let bytes = data.as_bytes_slice();
            if offset == 0 {
                gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, bytes, OpenGLVertexBuffer::gl_usage(self.usage));
            } else {
                gl.buffer_sub_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, offset as i32, bytes);
            }
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        }
        Ok(())
    }

    fn index_count(&self) -> usize {
        self.index_count
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for OpenGLIndexBuffer {
    fn drop(&mut self) {
        if !self.context_holder.is_context_active() {
            return;
        }
        unsafe {
            self.context_holder.ctx_ref().delete_buffer(self.buffer_id);
        }
    }
}

enum EffectParameterValue {
    Float(f32),
    Float2([f32; 2]),
    Float3([f32; 3]),
    Float4([f32; 4]),
    Matrix([[f32; 4]; 4]),
    Int(i32),
    Texture(glow::Texture),
}

pub struct OpenGLEffectParameter {
    name: String,
    location: glow::UniformLocation,
    value: Option<EffectParameterValue>,
    dirty: bool,
}

impl EffectParameter for OpenGLEffectParameter {
    fn set_float(&mut self, value: f32) -> GameResult {
        self.value = Some(EffectParameterValue::Float(value));
        self.dirty = true;
        Ok(())
    }

    fn set_float2(&mut self, value: [f32; 2]) -> GameResult {
        self.value = Some(EffectParameterValue::Float2(value));
        self.dirty = true;
        Ok(())
    }

    fn set_float3(&mut self, value: [f32; 3]) -> GameResult {
        self.value = Some(EffectParameterValue::Float3(value));
        self.dirty = true;
        Ok(())
    }

    fn set_float4(&mut self, value: [f32; 4]) -> GameResult {
        self.value = Some(EffectParameterValue::Float4(value));
        self.dirty = true;
        Ok(())
    }

    fn set_matrix(&mut self, value: &[[f32; 4]; 4]) -> GameResult {
        self.value = Some(EffectParameterValue::Matrix(*value));
        self.dirty = true;
        Ok(())
    }

    fn set_int(&mut self, value: i32) -> GameResult {
        self.value = Some(EffectParameterValue::Int(value));
        self.dirty = true;
        Ok(())
    }

    fn set_texture(&mut self, texture: &dyn BackendTexture) -> GameResult {
        let gl_texture = texture
            .as_any()
            .downcast_ref::<OpenGLTexture>()
            .ok_or_else(|| RenderError("Texture was not created by OpenGL backend.".to_string()))?;
        self.value = Some(EffectParameterValue::Texture(gl_texture.texture_id));
        self.dirty = true;
        Ok(())
    }
}

struct OpenGLEffectTechnique {
    name: String,
    program_id: glow::Program,
    parameters: HashMap<String, usize>, // maps param name -> index in effect's params vec
}

impl EffectTechnique for OpenGLEffectTechnique {
    fn name(&self) -> &str {
        &self.name
    }

    fn pass_count(&self) -> usize {
        1 // TODO: multiple passes?
    }
}

pub struct OpenGLEffect {
    techniques: Vec<OpenGLEffectTechnique>,
    parameters: Vec<OpenGLEffectParameter>,
    current_technique: usize,
    context_holder: GlContextHolder,
}

impl OpenGLEffect {
    fn upload_dirty_params(&mut self) {
        let gl = self.context_holder.ctx_ref();
        let mut texture_unit = 0u32;

        for param in &mut self.parameters {
            if !param.dirty {
                continue;
            }
            param.dirty = false;

            unsafe {
                match &param.value {
                    Some(EffectParameterValue::Float(v)) => {
                        gl.uniform_1_f32(Some(&param.location), *v);
                    }
                    Some(EffectParameterValue::Float2(v)) => {
                        gl.uniform_2_f32(Some(&param.location), v[0], v[1]);
                    }
                    Some(EffectParameterValue::Float3(v)) => {
                        gl.uniform_3_f32(Some(&param.location), v[0], v[1], v[2]);
                    }
                    Some(EffectParameterValue::Float4(v)) => {
                        gl.uniform_4_f32(Some(&param.location), v[0], v[1], v[2], v[3]);
                    }
                    Some(EffectParameterValue::Matrix(v)) => {
                        gl.uniform_matrix_4_f32_slice(Some(&param.location), false, v.as_flattened());
                    }
                    Some(EffectParameterValue::Int(v)) => {
                        gl.uniform_1_i32(Some(&param.location), *v);
                    }
                    Some(EffectParameterValue::Texture(tex_id)) => {
                        gl.active_texture(glow::TEXTURE0 + texture_unit);
                        gl.bind_texture(glow::TEXTURE_2D, Some(*tex_id));
                        gl.uniform_1_i32(Some(&param.location), texture_unit as i32);
                        texture_unit += 1;
                    }
                    None => {}
                }
            }
        }
    }
}

impl BackendEffect for OpenGLEffect {
    fn technique_count(&self) -> usize {
        self.techniques.len()
    }

    fn current_technique(&self) -> usize {
        self.current_technique
    }

    fn set_current_technique(&mut self, index: usize) -> GameResult {
        if index >= self.techniques.len() {
            return Err(RenderError(format!("Technique index {} out of range", index)));
        }
        self.current_technique = index;
        Ok(())
    }

    fn technique_name(&self, index: usize) -> Option<&str> {
        self.techniques.get(index).map(|t| t.name.as_str())
    }

    fn pass_count(&self) -> usize {
        1 // TODO: multiple passes?
    }

    fn begin_pass(&mut self, _pass_index: usize) -> GameResult {
        let gl = self.context_holder.ctx_ref();
        let technique = &self.techniques[self.current_technique];
        unsafe {
            gl.use_program(Some(technique.program_id));
        }
        // mark all params dirty so they get re-uploaded for this program
        for param in &mut self.parameters {
            param.dirty = true;
        }
        self.upload_dirty_params();
        Ok(())
    }

    fn end_pass(&mut self) -> GameResult {
        let gl = self.context_holder.ctx_ref();
        unsafe {
            gl.use_program(None);
        }
        Ok(())
    }

    fn parameter(&self, name: &str) -> Option<&dyn EffectParameter> {
        self.parameters.iter().find(|p| p.name == name).map(|p| p as &dyn EffectParameter)
    }

    fn parameter_mut(&mut self, name: &str) -> Option<&mut dyn EffectParameter> {
        self.parameters.iter_mut().find(|p| p.name == name).map(|p| p as &mut dyn EffectParameter)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for OpenGLEffect {
    fn drop(&mut self) {
        if !self.context_holder.is_context_active() {
            return;
        }
        let gl = self.context_holder.ctx_ref();
        unsafe {
            for technique in &self.techniques {
                gl.delete_program(technique.program_id);
            }
        }
    }
}

unsafe fn bind_vertex_declaration(gl: &glow::Context, program: glow::Program, decl: &VertexDeclaration) {
    for element in &decl.elements {
        let attr_name = match element.usage {
            VertexElementUsage::Position => "Position",
            VertexElementUsage::Normal => "Normal",
            VertexElementUsage::TextureCoordinate => "UV",
            VertexElementUsage::Color => "Color",
            VertexElementUsage::Tangent => "Tangent",
            VertexElementUsage::Binormal => "Binormal",
        };

        if let Some(location) = gl.get_attrib_location(program, attr_name) {
            gl.enable_vertex_attrib_array(location);

            let (component_count, gl_type, normalized) = match element.format {
                VertexElementFormat::Float1 => (1, glow::FLOAT, false),
                VertexElementFormat::Float2 => (2, glow::FLOAT, false),
                VertexElementFormat::Float3 => (3, glow::FLOAT, false),
                VertexElementFormat::Float4 => (4, glow::FLOAT, false),
                VertexElementFormat::Color => (4, glow::UNSIGNED_BYTE, true),
                VertexElementFormat::Byte4 => (4, glow::UNSIGNED_BYTE, false),
                VertexElementFormat::Short2 => (2, glow::SHORT, false),
                VertexElementFormat::Short4 => (4, glow::SHORT, false),
            };

            gl.vertex_attrib_pointer_f32(
                location,
                component_count,
                gl_type,
                normalized,
                decl.stride as i32,
                element.offset as i32,
            );
        }
    }
}

fn gl_primitive_type(prim_type: PrimitiveType) -> u32 {
    match prim_type {
        PrimitiveType::TriangleList => glow::TRIANGLES,
        PrimitiveType::TriangleStrip => glow::TRIANGLE_STRIP,
        PrimitiveType::LineList => glow::LINES,
        PrimitiveType::LineStrip => glow::LINE_STRIP,
    }
}

fn primitive_count_to_vertex_count(prim_type: PrimitiveType, prim_count: usize) -> usize {
    match prim_type {
        PrimitiveType::TriangleList => prim_count * 3,
        PrimitiveType::TriangleStrip => prim_count + 2,
        PrimitiveType::LineList => prim_count * 2,
        PrimitiveType::LineStrip => prim_count + 1,
    }
}

impl Drop for OpenGLRenderer {
    fn drop(&mut self) {
        let context = self.gl.get_mut();
        if let Some(context) = context {
            context.renderer_dropped();
        }
    }
}

fn macos_drain_errors(gl: &glow::Context) {
    let _ = gl;
    #[cfg(target_os = "macos")]
    unsafe {
        while gl.get_error() != glow::NO_ERROR {}
    }
}

fn check_gl_errors(hint: &str, gl: &glow::Context) {
    let _ = hint;
    loop {
        // drain GL errors

        let error = unsafe { gl.get_error() };
        if error == glow::NO_ERROR {
            break;
        }

        #[cfg(debug_assertions)]
        {
            use std::borrow::Cow;
            let name = match error {
                glow::INVALID_ENUM => Cow::Borrowed("INVALID_ENUM"),
                glow::INVALID_FRAMEBUFFER_OPERATION => Cow::Borrowed("INVALID_FRAMEBUFFER_OPERATION"),
                glow::INVALID_OPERATION => Cow::Borrowed("INVALID_OPERATION"),
                glow::INVALID_VALUE => Cow::Borrowed("INVALID_VALUE"),
                glow::OUT_OF_MEMORY => Cow::Borrowed("OUT_OF_MEMORY"),
                _ => Cow::Owned(error.to_string()),
            };

            log::error!("GL error: {name} {error:#x} ({hint})");
            // panic!();
        }
    }
}

