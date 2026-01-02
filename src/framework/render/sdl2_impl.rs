use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::rc::Rc;

use sdl2::controller::GameController;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use sdl2::mouse::{Cursor, SystemCursor};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Texture, TextureCreator, TextureQuery, WindowCanvas};
use sdl2::rwops::RWops;
use sdl2::surface::Surface;
use sdl2::video::{FullscreenType, GLProfile, Window, WindowContext};
use sdl2::{controller, keyboard, pixels, EventPump, GameControllerSubsystem, Sdl, VideoSubsystem};

use crate::common::{Color, Rect};
use crate::framework::backend::{BackendRenderer, BackendShader, BackendTexture, SpriteBatchCommand, VertexData};
use crate::framework::backend_sdl2::SDL2Context;
use crate::framework::error::{GameError, GameResult};
use crate::framework::graphics::{BlendMode, IndexData};

pub struct SDL2Renderer {
    refs: Rc<RefCell<SDL2Context>>,
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
        let canvas = refs.window.canvas();

        canvas.set_draw_color(to_sdl(color));
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        canvas.clear();
    }

    fn present(&mut self) -> GameResult {
        let mut refs = self.refs.borrow_mut();
        let canvas = refs.window.canvas();

        canvas.present();

        Ok(())
    }

    fn prepare_draw(&mut self, width: f32, height: f32) -> GameResult {
        let mut refs = self.refs.borrow_mut();
        let canvas = refs.window.canvas();

        canvas.set_clip_rect(Some(sdl2::rect::Rect::new(0, 0, width as u32, height as u32)));

        Ok(())
    }

    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        let mut refs = self.refs.borrow_mut();

        let texture = refs
            .window
            .texture_creator()
            .create_texture_target(PixelFormatEnum::RGBA32, width as u32, height as u32)
            .map_err(|e| GameError::RenderError(e.to_string()))?;

        Ok(Box::new(SDL2Texture { refs: self.refs.clone(), texture: Some(texture), width, height, commands: vec![] }))
    }

    fn create_texture(&mut self, width: u16, height: u16, data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        let mut refs = self.refs.borrow_mut();

        let mut texture = refs
            .window
            .texture_creator()
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
            BlendMode::None => sdl2::render::BlendMode::None,
        };

        Ok(())
    }

    fn set_render_target(&mut self, texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        let renderer = self.refs.borrow_mut().window.canvas().raw();

        match texture {
            Some(texture) => {
                let sdl2_texture = texture
                    .as_any()
                    .downcast_ref::<SDL2Texture>()
                    .ok_or(GameError::RenderError("This texture was not created by SDL2 backend.".to_string()))?;

                unsafe {
                    if let Some(target) = &sdl2_texture.texture {
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
        let blend = refs.blend_mode;
        let canvas = refs.window.canvas();

        let (r, g, b, a) = color.to_rgba();

        canvas.set_draw_color(pixels::Color::RGBA(r, g, b, a));
        canvas.set_blend_mode(blend);
        canvas
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
        let blend = refs.blend_mode;
        let canvas = refs.window.canvas();

        let (r, g, b, a) = color.to_rgba();

        canvas.set_draw_color(pixels::Color::RGBA(r, g, b, a));
        canvas.set_blend_mode(blend);

        match line_width {
            0 => {} // no-op
            1 => {
                canvas
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

                canvas.fill_rects(&rects).map_err(|e| GameError::RenderError(e.to_string()))?;
            }
        }

        Ok(())
    }

    fn set_clip_rect(&mut self, rect: Option<Rect>) -> GameResult {
        let mut refs = self.refs.borrow_mut();
        let canvas = refs.window.canvas();

        if let Some(rect) = &rect {
            canvas.set_clip_rect(Some(sdl2::rect::Rect::new(
                rect.left as i32,
                rect.top as i32,
                rect.width() as u32,
                rect.height() as u32,
            )));
        } else {
            canvas.set_clip_rect(None);
        }

        Ok(())
    }

    fn draw_triangles(
        &mut self,
        vertices: &[VertexData],
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult<()> {
        self.render_geometry(vertices, None, texture, shader)
    }

    fn draw_triangles_indexed(
        &mut self,
        vertices: &[VertexData],
        indices: IndexData,
        texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult {
        self.render_geometry(vertices, Some(indices), texture, shader)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl SDL2Renderer {
    pub fn new(refs: Rc<RefCell<SDL2Context>>) -> Self {
        SDL2Renderer { refs }
    }

    fn render_geometry(
        &mut self,
        vertices: &[VertexData],
        indices: Option<IndexData>,
        mut texture: Option<&Box<dyn BackendTexture>>,
        shader: BackendShader,
    ) -> GameResult {
        let mut refs = self.refs.borrow_mut();
        if shader == BackendShader::Fill {
            texture = None;
        } else if let BackendShader::WaterFill(..) = shader {
            texture = None;
        }

        let texture_ptr = if let Some(texture) = texture {
            SDL2Texture::try_dyn_ref(texture.as_ref())?.texture.as_ref().map_or(null_mut(), |t| t.raw())
        } else {
            null_mut::<sdl2_sys::SDL_Texture>()
        };

        unsafe {
            const VERTEX_DATA_STRIDE: i32 = std::mem::size_of::<VertexData>() as _;
            let first_vertex = vertices.as_ptr();
            let position_off = &raw const (*first_vertex).position as *const f32;
            let color_off = &raw const (*first_vertex).color as *const sdl2_sys::SDL_Color;
            let uv_off = &raw const (*first_vertex).uv as *const f32;

            let (num_indices, indices, size_indices) = if let Some(indices) = indices {
                let count = indices.len() as i32;
                let pointer = indices.as_bytes_slice().as_ptr() as *const c_void;
                let size = indices.element_size() as i32;

                (count, pointer, size)
            } else {
                (0, null(), 0)
            };

            sdl2_sys::SDL_RenderGeometryRaw(
                refs.window.canvas().raw(),
                texture_ptr,
                position_off,
                VERTEX_DATA_STRIDE,
                color_off,
                VERTEX_DATA_STRIDE,
                uv_off,
                VERTEX_DATA_STRIDE,
                vertices.len() as i32,
                indices,
                num_indices,
                size_indices,
            );
        }

        Ok(())
    }
}

struct SDL2Texture {
    refs: Rc<RefCell<SDL2Context>>,
    texture: Option<Texture>,
    width: u16,
    height: u16,
    commands: Vec<SpriteBatchCommand>,
}

impl SDL2Texture {
    fn try_dyn_ref(texture: &dyn BackendTexture) -> GameResult<&Self> {
        texture
            .as_any()
            .downcast_ref::<Self>()
            .ok_or_else(|| GameError::RenderError("This texture was not created by SDL2 backend.".to_string()))
    }
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
        match &mut self.texture {
            None => Ok(()),
            Some(texture) => {
                let mut refs = self.refs.borrow_mut();
                let blend = refs.blend_mode;
                let canvas = refs.window.canvas();
                for command in &self.commands {
                    match command {
                        SpriteBatchCommand::DrawRect(src, dest) => {
                            texture.set_color_mod(255, 255, 255);
                            texture.set_alpha_mod(255);
                            texture.set_blend_mode(blend);

                            canvas
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
                            texture.set_blend_mode(blend);

                            canvas
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
                            texture.set_blend_mode(blend);

                            canvas
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
                        SpriteBatchCommand::DrawRectFlipTinted(src, dest, flip_x, flip_y, color) => {
                            let (r, g, b, a) = color.to_rgba();
                            texture.set_color_mod(r, g, b);
                            texture.set_alpha_mod(a);
                            texture.set_blend_mode(blend);

                            canvas
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
        std::mem::swap(&mut self.texture, &mut texture_opt);

        if let Some(texture) = texture_opt {
            unsafe {
                texture.destroy();
            }
        }
    }
}
