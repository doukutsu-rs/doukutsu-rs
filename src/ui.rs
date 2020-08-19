use std::time::Instant;

use ggez::{Context, GameResult, graphics};
use ggez::GameError::RenderError;
use imgui::{FontConfig, FontSource};
use imgui_gfx_renderer::{Renderer, Shaders};
use imgui_gfx_renderer::gfx::format::Rgba8;
use imgui_gfx_renderer::gfx::handle::RenderTargetView;
use imgui_gfx_renderer::gfx::memory::Typed;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use crate::scene::Scene;
use crate::GameContext;
use crate::game_state::GameState;

mod types {
    pub type Device = gfx_device_gl::Device;
    pub type Factory = gfx_device_gl::Factory;
    pub type Resources = gfx_device_gl::Resources;
}

pub struct UI {
    pub imgui: imgui::Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer<Rgba8, types::Resources>,
    main_color: RenderTargetView<types::Resources, Rgba8>,
    last_frame: Instant,
}

impl UI {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        imgui.fonts().add_font(&[
            FontSource::DefaultFontData {
                config: Some(FontConfig::default()),
            },
        ]);
        imgui.style_mut().use_dark_colors();

        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), graphics::window(ctx), HiDpiMode::Rounded);

        let (factory, dev, _, _, color) = graphics::gfx_objects(ctx);
        let shaders = {
            let version = dev.get_info().shading_language;
            if version.is_embedded {
                if version.major >= 3 {
                    Shaders::GlSlEs300
                } else {
                    Shaders::GlSlEs100
                }
            } else if version.major >= 4 {
                Shaders::GlSl400
            } else if version.major >= 3 {
                if version.minor >= 2 {
                    Shaders::GlSl150
                } else {
                    Shaders::GlSl130
                }
            } else {
                Shaders::GlSl110
            }
        };
        let renderer = Renderer::init(&mut imgui, factory, shaders)
            .map_err(|e| RenderError(e.to_string()))?;

        Ok(Self {
            imgui,
            platform,
            renderer,
            main_color: RenderTargetView::new(color),
            last_frame: Instant::now(),
        })
    }

    pub fn handle_events(&mut self, ctx: &mut Context, event: &winit::Event) {
        self.platform.handle_event(self.imgui.io_mut(), graphics::window(ctx), &event);
    }

    pub fn draw(&mut self, state: &mut GameState, game_ctx: &mut GameContext, ctx: &mut Context, scene: &mut Box<dyn Scene>) -> GameResult {
        {
            let io = self.imgui.io_mut();
            self.platform.prepare_frame(io, graphics::window(ctx)).map_err(|e| RenderError(e))?;

            io.update_delta_time(self.last_frame);
            self.last_frame = Instant::now();
        }
        let mut ui = self.imgui.frame();

        scene.overlay_draw(state, game_ctx, ctx, &mut ui)?;

        self.platform.prepare_render(&ui, graphics::window(ctx));
        let draw_data = ui.render();
        let (factory, dev, encoder, _, _) = graphics::gfx_objects(ctx);
        self.renderer
            .render(factory, encoder, &mut self.main_color, draw_data)
            .map_err(|e| RenderError(e.to_string()))?;

        encoder.flush(dev);

        Ok(())
    }
}
