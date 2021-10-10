use std::cell::RefCell;
use std::mem;

use imgui::DrawData;

use crate::common::{Color, Rect};
use crate::framework::backend::{
    Backend, BackendEventLoop, BackendRenderer, BackendShader, BackendTexture, SpriteBatchCommand, VertexData,
};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::BlendMode;
use crate::Game;

pub struct NullBackend;

impl NullBackend {
    pub fn new() -> GameResult<Box<dyn Backend>> {
        Ok(Box::new(NullBackend))
    }
}

impl Backend for NullBackend {
    fn create_event_loop(&self) -> GameResult<Box<dyn BackendEventLoop>> {
        Ok(Box::new(NullEventLoop))
    }
}

pub struct NullEventLoop;

impl BackendEventLoop for NullEventLoop {
    fn run(&mut self, game: &mut Game, ctx: &mut Context) {
        let state_ref = unsafe { &mut *game.state.get() };

        ctx.screen_size = (640.0, 480.0);
        state_ref.handle_resize(ctx).unwrap();

        loop {
            game.update(ctx).unwrap();

            if state_ref.shutdown {
                log::info!("Shutting down...");
                break;
            }

            if state_ref.next_scene.is_some() {
                mem::swap(&mut game.scene, &mut state_ref.next_scene);
                state_ref.next_scene = None;
                game.scene.as_mut().unwrap().init(state_ref, ctx).unwrap();
                game.loops = 0;
                state_ref.frame_time = 0.0;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));

            //game.draw(ctx).unwrap();
        }
    }

    fn new_renderer(&self) -> GameResult<Box<dyn BackendRenderer>> {
        let mut imgui = imgui::Context::create();
        imgui.io_mut().display_size = [640.0, 480.0];
        imgui.fonts().build_alpha8_texture();

        Ok(Box::new(NullRenderer(RefCell::new(imgui))))
    }
}

pub struct NullTexture(u16, u16);

impl BackendTexture for NullTexture {
    fn dimensions(&self) -> (u16, u16) {
        (self.0, self.1)
    }

    fn add(&mut self, _command: SpriteBatchCommand) {}

    fn clear(&mut self) {}

    fn draw(&mut self) -> GameResult<()> {
        Ok(())
    }
}

pub struct NullRenderer(RefCell<imgui::Context>);

impl BackendRenderer for NullRenderer {
    fn renderer_name(&self) -> String {
        "Null".to_owned()
    }

    fn clear(&mut self, _color: Color) {}

    fn present(&mut self) -> GameResult {
        Ok(())
    }

    fn create_texture_mutable(&mut self, width: u16, height: u16) -> GameResult<Box<dyn BackendTexture>> {
        Ok(Box::new(NullTexture(width, height)))
    }

    fn create_texture(&mut self, width: u16, height: u16, _data: &[u8]) -> GameResult<Box<dyn BackendTexture>> {
        Ok(Box::new(NullTexture(width, height)))
    }

    fn set_blend_mode(&mut self, _blend: BlendMode) -> GameResult {
        Ok(())
    }

    fn set_render_target(&mut self, _texture: Option<&Box<dyn BackendTexture>>) -> GameResult {
        Ok(())
    }

    fn draw_rect(&mut self, _rect: Rect<isize>, _color: Color) -> GameResult {
        Ok(())
    }

    fn draw_outline_rect(&mut self, _rect: Rect<isize>, _line_width: usize, _color: Color) -> GameResult {
        Ok(())
    }

    fn imgui(&self) -> GameResult<&mut imgui::Context> {
        unsafe { Ok(&mut *self.0.as_ptr()) }
    }

    fn render_imgui(&mut self, _draw_data: &DrawData) -> GameResult {
        Ok(())
    }

    fn draw_triangle_list(
        &mut self,
        _vertices: Vec<VertexData>,
        _texture: Option<&Box<dyn BackendTexture>>,
        _shader: BackendShader,
    ) -> GameResult<()> {
        Ok(())
    }
}
