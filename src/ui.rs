use std::time::Instant;

use imgui::{FontConfig, FontSource};
use imgui::sys::*;
use imgui_gfx_renderer::{Renderer, Shaders};
use imgui_gfx_renderer::gfx::format::Rgba8;
use imgui_gfx_renderer::gfx::handle::RenderTargetView;
use imgui_gfx_renderer::gfx::memory::Typed;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use crate::ggez::{Context, GameResult, graphics};
use crate::ggez::GameError::RenderError;
use crate::live_debugger::LiveDebugger;
use crate::scene::Scene;
use crate::SharedGameState;

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

        imgui.style_mut().window_padding = [4.0, 6.0];
        imgui.style_mut().frame_padding = [8.0, 6.0];
        imgui.style_mut().item_spacing = [8.0, 6.0];
        imgui.style_mut().item_inner_spacing = [8.0, 6.0];
        imgui.style_mut().indent_spacing = 20.0;

        imgui.style_mut().scrollbar_size = 20.0;
        imgui.style_mut().grab_min_size = 5.0;
        imgui.style_mut().window_border_size = 0.0;
        imgui.style_mut().child_border_size = 1.0;
        imgui.style_mut().popup_border_size = 1.0;
        imgui.style_mut().frame_border_size = 1.0;
        imgui.style_mut().tab_border_size = 0.0;

        imgui.style_mut().window_rounding = 0.0;
        imgui.style_mut().child_rounding = 0.0;
        imgui.style_mut().frame_rounding = 0.0;
        imgui.style_mut().popup_rounding = 0.0;
        imgui.style_mut().scrollbar_rounding = 0.0;
        imgui.style_mut().grab_rounding = 0.0;
        imgui.style_mut().tab_rounding = 0.0;

        imgui.style_mut().window_title_align = [0.50, 0.50];
        imgui.style_mut().window_rounding = 0.0;

        let colors = &mut imgui.style_mut().colors;
        colors[ImGuiCol_Text as usize] = [0.90, 0.90, 0.90, 1.00];
        colors[ImGuiCol_TextDisabled as usize] = [0.50, 0.50, 0.50, 1.00];
        colors[ImGuiCol_WindowBg as usize] = [0.05, 0.05, 0.05, 0.60];
        colors[ImGuiCol_ChildBg as usize] = [0.05, 0.05, 0.05, 0.60];
        colors[ImGuiCol_PopupBg as usize] = [0.00, 0.00, 0.00, 0.60];
        colors[ImGuiCol_Border as usize] = [0.40, 0.40, 0.40, 1.00];
        colors[ImGuiCol_BorderShadow as usize] = [1.00, 1.00, 1.00, 0.00];
        colors[ImGuiCol_FrameBg as usize] = [0.00, 0.00, 0.00, 0.60];
        colors[ImGuiCol_FrameBgHovered as usize] = [0.84, 0.37, 0.00, 0.20];
        colors[ImGuiCol_FrameBgActive as usize] = [0.84, 0.37, 0.00, 1.00];
        colors[ImGuiCol_TitleBg as usize] = [0.06, 0.06, 0.06, 1.00];
        colors[ImGuiCol_TitleBgActive as usize] = [0.00, 0.00, 0.00, 1.00];
        colors[ImGuiCol_TitleBgCollapsed as usize] = [0.06, 0.06, 0.06, 0.40];
        colors[ImGuiCol_MenuBarBg as usize] = [0.14, 0.14, 0.14, 1.00];
        colors[ImGuiCol_ScrollbarBg as usize] = [0.14, 0.14, 0.14, 0.40];
        colors[ImGuiCol_ScrollbarGrab as usize] = [0.31, 0.31, 0.31, 0.30];
        colors[ImGuiCol_ScrollbarGrabHovered as usize] = [1.00, 1.00, 1.00, 0.30];
        colors[ImGuiCol_ScrollbarGrabActive as usize] = [1.00, 1.00, 1.00, 0.50];
        colors[ImGuiCol_CheckMark as usize] = [0.90, 0.90, 0.90, 1.00];
        colors[ImGuiCol_SliderGrab as usize] = [0.31, 0.31, 0.31, 1.00];
        colors[ImGuiCol_SliderGrabActive as usize] = [1.00, 1.00, 1.00, 0.50];
        colors[ImGuiCol_Button as usize] = [0.14, 0.14, 0.14, 1.00];
        colors[ImGuiCol_ButtonHovered as usize] = [0.84, 0.37, 0.00, 0.20];
        colors[ImGuiCol_ButtonActive as usize] = [0.84, 0.37, 0.00, 1.00];
        colors[ImGuiCol_Header as usize] = [0.14, 0.14, 0.14, 1.00];
        colors[ImGuiCol_HeaderHovered as usize] = [0.84, 0.37, 0.00, 0.20];
        colors[ImGuiCol_HeaderActive as usize] = [0.84, 0.37, 0.00, 1.00];
        colors[ImGuiCol_Separator as usize] = [0.50, 0.50, 0.43, 0.50];
        colors[ImGuiCol_SeparatorHovered as usize] = [0.75, 0.45, 0.10, 0.78];
        colors[ImGuiCol_SeparatorActive as usize] = [0.75, 0.45, 0.10, 1.00];
        colors[ImGuiCol_ResizeGrip as usize] = [0.98, 0.65, 0.26, 0.25];
        colors[ImGuiCol_ResizeGripHovered as usize] = [0.98, 0.65, 0.26, 0.67];
        colors[ImGuiCol_ResizeGripActive as usize] = [0.98, 0.65, 0.26, 0.95];
        colors[ImGuiCol_Tab as usize] = [0.17, 0.10, 0.04, 0.94];
        colors[ImGuiCol_TabHovered as usize] = [0.84, 0.37, 0.00, 0.60];
        colors[ImGuiCol_TabActive as usize] = [0.67, 0.30, 0.00, 0.68];
        colors[ImGuiCol_TabUnfocused as usize] = [0.06, 0.05, 0.05, 0.69];
        colors[ImGuiCol_TabUnfocusedActive as usize] = [0.36, 0.17, 0.03, 0.64];
        colors[ImGuiCol_PlotLines as usize] = [0.39, 0.39, 0.39, 1.00];
        colors[ImGuiCol_PlotLinesHovered as usize] = [0.35, 0.92, 1.00, 1.00];
        colors[ImGuiCol_PlotHistogram as usize] = [0.00, 0.20, 0.90, 1.00];
        colors[ImGuiCol_PlotHistogramHovered as usize] = [0.00, 0.40, 1.00, 1.00];
        colors[ImGuiCol_TextSelectedBg as usize] = [0.98, 0.65, 0.26, 0.35];
        colors[ImGuiCol_DragDropTarget as usize] = [0.00, 0.00, 1.00, 0.90];
        colors[ImGuiCol_NavHighlight as usize] = [0.98, 0.65, 0.26, 1.00];
        colors[ImGuiCol_NavWindowingHighlight as usize] = [0.00, 0.00, 0.00, 0.70];
        colors[ImGuiCol_NavWindowingDimBg as usize] = [0.20, 0.20, 0.20, 0.20];
        colors[ImGuiCol_ModalWindowDimBg as usize] = [0.20, 0.20, 0.20, 0.35];

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

    pub fn draw(&mut self, dbg: &mut LiveDebugger, state: &mut SharedGameState, ctx: &mut Context, scene: &mut Box<dyn Scene>) -> GameResult {
        {
            let io = self.imgui.io_mut();
            self.platform.prepare_frame(io, graphics::window(ctx)).map_err(|e| RenderError(e))?;

            io.update_delta_time(self.last_frame);
            self.last_frame = Instant::now();
        }
        let mut ui = self.imgui.frame();

        scene.debug_overlay_draw(dbg, state, ctx, &mut ui)?;

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
