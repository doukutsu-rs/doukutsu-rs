//! Extensions for [glutin](https://crates.io/crates/glutin) to initialize & update old school
//! [gfx](https://crates.io/crates/gfx). _An alternative to gfx_window_glutin_.
//!
//! # Example
//! ```no_run
//! use old_school_gfx_glutin_ext::*;
//!
//! type ColorFormat = gfx::format::Srgba8;
//! type DepthFormat = gfx::format::DepthStencil;
//!
//! # fn main() -> Result<(), glutin::CreationError> {
//! # let event_loop = glutin::event_loop::EventLoop::new();
//! # let window_config = glutin::window::WindowBuilder::new();
//! // Initialize
//! let (window_ctx, mut device, mut factory, mut main_color, mut main_depth) =
//!     glutin::ContextBuilder::new()
//!         .with_gfx_color_depth::<ColorFormat, DepthFormat>()
//!         .build_windowed(window_config, &event_loop)?
//!         .init_gfx::<ColorFormat, DepthFormat>();
//!
//! # let new_size = glutin::dpi::PhysicalSize::new(1, 1);
//! // Update, ie after a resize
//! window_ctx.update_gfx(&mut main_color, &mut main_depth);
//! # Ok(()) }
//! ```

#![allow(unsafe_code)]
use gfx_core::{
    format::{ChannelType, DepthFormat, Format, RenderFormat},
    handle::{DepthStencilView, RawDepthStencilView, RawRenderTargetView, RenderTargetView},
    memory::Typed,
    texture,
};
use gfx_device_gl::Resources as R;
use glutin::{NotCurrent, PossiblyCurrent};

type GfxInitTuple<Color, Depth> = (
    glutin::WindowedContext<PossiblyCurrent>,
    gfx_device_gl::Device,
    gfx_device_gl::Factory,
    RenderTargetView<R, Color>,
    DepthStencilView<R, Depth>,
);

pub trait ContextBuilderExt {
    /// Calls `with_pixel_format` & `with_srgb` according to the color format.
    fn with_gfx_color<Color: RenderFormat>(self) -> Self;
    /// Calls `with_pixel_format` & `with_srgb` according to the color format.
    fn with_gfx_color_raw(self, color_format: Format) -> Self;
    /// Calls `with_depth_buffer` & `with_stencil_buffer` according to the depth format.
    fn with_gfx_depth<Depth: DepthFormat>(self) -> Self;
    /// Calls `with_depth_buffer` & `with_stencil_buffer` according to the depth format.
    fn with_gfx_depth_raw(self, ds_format: Format) -> Self;
    /// Calls `with_gfx_color` & `with_gfx_depth`.
    fn with_gfx_color_depth<Color: RenderFormat, Depth: DepthFormat>(self) -> Self;
}

impl ContextBuilderExt for glutin::ContextBuilder<'_, NotCurrent> {
    fn with_gfx_color<Color: RenderFormat>(self) -> Self {
        self.with_gfx_color_raw(Color::get_format())
    }

    fn with_gfx_color_raw(self, Format(surface, channel): Format) -> Self {
        let color_total_bits = surface.get_total_bits();
        let alpha_bits = surface.get_alpha_stencil_bits();

        self.with_pixel_format(color_total_bits - alpha_bits, alpha_bits)
    }

    fn with_gfx_depth<Depth: DepthFormat>(self) -> Self {
        self.with_gfx_depth_raw(Depth::get_format())
    }

    fn with_gfx_depth_raw(self, Format(surface, _): Format) -> Self {
        let depth_total_bits = surface.get_total_bits();
        let stencil_bits = surface.get_alpha_stencil_bits();

        self.with_depth_buffer(depth_total_bits - stencil_bits)
            .with_stencil_buffer(stencil_bits)
    }

    fn with_gfx_color_depth<Color: RenderFormat, Depth: DepthFormat>(self) -> Self {
        self.with_gfx_color::<Color>().with_gfx_depth::<Depth>()
    }
}

pub trait WindowInitExt {
    /// Make the context current, creates the gfx device, factory and views.
    fn init_gfx<Color: RenderFormat, Depth: DepthFormat>(self) -> GfxInitTuple<Color, Depth>;
    /// Make the context current, creates the gfx device, factory and views.
    fn init_gfx_raw(
        self,
        color_format: Format,
        ds_format: Format,
    ) -> (
        glutin::WindowedContext<PossiblyCurrent>,
        gfx_device_gl::Device,
        gfx_device_gl::Factory,
        RawRenderTargetView<R>,
        RawDepthStencilView<R>,
    );
}

impl WindowInitExt for glutin::WindowedContext<NotCurrent> {
    fn init_gfx<Color: RenderFormat, Depth: DepthFormat>(self) -> GfxInitTuple<Color, Depth> {
        let (window, device, factory, color_view, ds_view) =
            self.init_gfx_raw(Color::get_format(), Depth::get_format());
        (
            window,
            device,
            factory,
            Typed::new(color_view),
            Typed::new(ds_view),
        )
    }

    fn init_gfx_raw(
        self,
        color_format: Format,
        ds_format: Format,
    ) -> (
        glutin::WindowedContext<PossiblyCurrent>,
        gfx_device_gl::Device,
        gfx_device_gl::Factory,
        RawRenderTargetView<R>,
        RawDepthStencilView<R>,
    ) {
        let window = unsafe { self.make_current().unwrap() };
        let (device, factory) =
            gfx_device_gl::create(|s| window.get_proc_address(s) as *const std::os::raw::c_void);

        let dim = get_window_dimensions(&window);
        let (color_view, ds_view) =
            gfx_device_gl::create_main_targets_raw(dim, color_format.0, ds_format.0);

        (window, device, factory, color_view, ds_view)
    }
}

pub trait WindowUpdateExt {
    /// Recreates the views if the dimensions have changed.
    fn update_gfx<Color: RenderFormat, Depth: DepthFormat>(
        &self,
        color_view: &mut RenderTargetView<R, Color>,
        ds_view: &mut DepthStencilView<R, Depth>,
    );
    /// Return new main target views if the window resolution has changed from the old dimensions.
    fn updated_views_raw(
        &self,
        old_dimensions: texture::Dimensions,
        color_format: Format,
        ds_format: Format,
    ) -> Option<(RawRenderTargetView<R>, RawDepthStencilView<R>)>;
}

impl WindowUpdateExt for glutin::WindowedContext<PossiblyCurrent> {
    fn update_gfx<Color: RenderFormat, Depth: DepthFormat>(
        &self,
        color_view: &mut RenderTargetView<R, Color>,
        ds_view: &mut DepthStencilView<R, Depth>,
    ) {
        let dim = color_view.get_dimensions();
        debug_assert_eq!(dim, ds_view.get_dimensions());
        if let Some((cv, dv)) =
        self.updated_views_raw(dim, Color::get_format(), Depth::get_format())
        {
            *color_view = Typed::new(cv);
            *ds_view = Typed::new(dv);
        }
    }

    fn updated_views_raw(
        &self,
        old_dimensions: texture::Dimensions,
        color_format: Format,
        ds_format: Format,
    ) -> Option<(RawRenderTargetView<R>, RawDepthStencilView<R>)> {
        let dim = get_window_dimensions(self);
        if dim != old_dimensions {
            Some(gfx_device_gl::create_main_targets_raw(
                dim,
                color_format.0,
                ds_format.0,
            ))
        } else {
            None
        }
    }
}

fn get_window_dimensions(ctx: &glutin::WindowedContext<PossiblyCurrent>) -> texture::Dimensions {
    let window = ctx.window();
    let (width, height) = {
        let size = window.inner_size();
        (size.width as _, size.height as _)
    };
    let aa = ctx.get_pixel_format().multisampling.unwrap_or(0) as texture::NumSamples;

    (width, height, 1, aa.into())
}
