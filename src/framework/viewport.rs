use crate::common::Rect;

/// Aspect constraint applied by [`Viewport::recompute`]. This is the framework-level primitive:
/// it doesn't know anything about game editions. The game layer owns a richer `AspectRatio` enum
/// (with "default" and parse-from-string) that resolves down to this type before being stored on
/// the viewport.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedAspect {
    /// Fill the window; canvas snaps to an integer multiple of the base resolution, leaving a thin
    /// black border where the window overshoots.
    Unrestricted,
    /// Fill the window exactly — canvas matches the window pixel-for-pixel. Always uses integer
    /// pixel scaling regardless of `ScalingMode`, since a fractional blit would defeat the intent.
    Stretch,
    /// Lock to an integer aspect ratio, with letterboxing or pillarboxing.
    Locked { w: u32, h: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScalingMode {
    /// Integer scaling only. Canvas displayed at exact `pixel_scale`; any remaining
    /// area in the window is left as letterbox/pillarbox.
    Integer,
    /// Canvas rendered at the nearest (round-half-up) integer factor but stretched
    /// to fill the viewport. The canvas is pixel-crisp; the final blit is bilinear.
    Scaled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InsetMode {
    /// Viewport fills the entire window; UI shifts to avoid cutouts. `effective_insets` pass through.
    FillScreen,
    /// Viewport shrinks into the safe area; `effective_insets` are zeroed.
    FitSafeArea,
}

/// Single source of truth for viewport geometry. Lives on [`Context`](crate::framework::context::Context).
///
/// The backend updates [`window_size`](Self::window_size) and [`raw_insets`](Self::raw_insets);
/// the game layer updates the configuration fields.
/// [`Viewport::recompute`] refreshes the computed outputs from either kind of change.
pub struct Viewport {
    // --- Raw inputs (written by the backend) ---
    /// Physical window pixel size.
    pub window_size: (u32, u32),
    /// Raw screen insets (left, top, right, bottom) in physical pixels.
    pub raw_insets: (f32, f32, f32, f32),

    // --- Computed outputs (refreshed by recompute) ---
    /// Rectangle inside the window where the game draws, in physical pixels.
    pub viewport_rect: Rect<u32>,
    /// Internal framebuffer size (always a pixel-perfect multiple of `base_height_step`).
    pub canvas_size: (u32, u32),
    /// `canvas_size` as `(f32, f32)` — kept to simplify call sites that mix floats and ints.
    pub screen_size: (f32, f32),
    /// Logical coordinate space that gameplay code draws into (== old `canvas_size`).
    pub logical_size: (f32, f32),
    /// Integer factor applied to the canvas (> 0).
    pub pixel_scale: u32,
    /// Scale from logical to viewport pixels. Equal to `pixel_scale` in Integer mode.
    pub display_scale: f32,
    /// Mirror of `display_scale` — kept for call-site compatibility with the old `ctx.viewport.scale`.
    pub scale: f32,
    /// Effective insets propagated to UI code — zero in FitSafeArea, raw in FillScreen.
    pub effective_insets: (f32, f32, f32, f32),

    // --- Configuration (written by the game on settings change) ---
    pub aspect: ResolvedAspect,
    pub scaling_mode: ScalingMode,
    pub inset_mode: InsetMode,
    /// Logical-height step that `pixel_scale=1` maps to. Always 240 for Cave Story — the logical
    /// coordinate space is shared across editions regardless of texture density.
    pub base_height_step: u32,
    /// Minimum granularity of `pixel_scale`. 1 for freeware/NXEngine (can scale 1x, 2x, 3x, …);
    /// 2 for Cave Story+ because its texture atlases are 2x density so the rendered pixels must
    /// come in pairs (2x, 4x, 6x, …). Set from `EngineConstants::texture_scale()`.
    pub texture_scale: u32,
}

impl Viewport {
    pub fn new() -> Viewport {
        let mut vp = Viewport {
            window_size: (640, 480),
            raw_insets: (0.0, 0.0, 0.0, 0.0),
            viewport_rect: Rect::new(0, 0, 640, 480),
            canvas_size: (320, 240),
            screen_size: (320.0, 240.0),
            logical_size: (320.0, 240.0),
            pixel_scale: 2,
            display_scale: 2.0,
            scale: 2.0,
            effective_insets: (0.0, 0.0, 0.0, 0.0),
            aspect: ResolvedAspect::Unrestricted,
            scaling_mode: ScalingMode::Scaled,
            inset_mode: InsetMode::FillScreen,
            base_height_step: 240,
            texture_scale: 1,
        };
        vp.recompute();
        vp
    }

    /// Re-derive all `Computed outputs` from the current inputs and configuration.
    pub fn recompute(&mut self) {
        let (win_w, win_h) = (self.window_size.0.max(1), self.window_size.1.max(1));

        // 1. Determine usable area based on inset mode.
        let (usable_origin, usable_size, effective_insets) = match self.inset_mode {
            InsetMode::FitSafeArea => {
                let (il, it, ir, ib) =
                    (self.raw_insets.0.max(0.0), self.raw_insets.1.max(0.0),
                     self.raw_insets.2.max(0.0), self.raw_insets.3.max(0.0));
                let ox = il as u32;
                let oy = it as u32;
                let uw = (win_w as i64 - il as i64 - ir as i64).max(1) as u32;
                let uh = (win_h as i64 - it as i64 - ib as i64).max(1) as u32;
                ((ox, oy), (uw, uh), (0.0, 0.0, 0.0, 0.0))
            }
            InsetMode::FillScreen => ((0, 0), (win_w, win_h), self.raw_insets),
        };

        // 2. Fit viewport into usable area based on aspect ratio.
        let (vp_w, vp_h) = match self.aspect {
            ResolvedAspect::Unrestricted | ResolvedAspect::Stretch => usable_size,
            ResolvedAspect::Locked { w, h } => {
                let aspect = w as f64 / h as f64;
                let by_width = (usable_size.0 as f64) / aspect;
                if by_width <= usable_size.1 as f64 {
                    (usable_size.0, by_width.floor().max(1.0) as u32)
                } else {
                    let by_height = (usable_size.1 as f64) * aspect;
                    (by_height.floor().max(1.0) as u32, usable_size.1)
                }
            }
        };

        let step = self.base_height_step.max(1);
        let tex = self.texture_scale.max(1);

        // 3-4. Natural vertical scale and integer pixel_scale (constrained to multiples of `tex`).
        //      Dividing by tex before rounding ensures the rounding happens in "texture-density
        //      units" — otherwise a CS+ build would visually "click" between 2x and 3x identically
        //      to freeware but only 2x and 4x would actually be available, making scaling feel uneven.
        //
        //      Stretch mode forces integer rounding (floor) regardless of scaling_mode, because a
        //      Scaled/bilinear blit would defeat the "fills the window pixel-for-pixel" intent.
        let ns_per_tex = vp_h as f32 / (step * tex) as f32;
        let k = match (self.aspect, self.scaling_mode) {
            (ResolvedAspect::Stretch, _) | (_, ScalingMode::Integer) => (ns_per_tex.floor() as u32).max(1),
            (_, ScalingMode::Scaled) => ((ns_per_tex + 0.5).floor() as u32).max(1),
        };
        let pixel_scale = k * tex;

        // 5. Canvas size.
        //    - Stretch: canvas matches the viewport exactly; logical size may be fractional.
        //    - Unrestricted: both dimensions snap to `pixel_scale` granularity so the final blit
        //      has *uniform* x/y scaling (preventing the slight squash that comes from locking
        //      height to `step*K` while letting width follow the viewport).
        //    - Locked aspect: canvas has the aspect's exact ratio; height is `step * pixel_scale`.
        let (canvas_w, canvas_h) = match self.aspect {
            ResolvedAspect::Stretch => (vp_w, vp_h),
            ResolvedAspect::Unrestricted => {
                let logical_w = ((vp_w as f32 / pixel_scale as f32).round() as u32).max(1);
                let logical_h = ((vp_h as f32 / pixel_scale as f32).round() as u32).max(1);
                (logical_w * pixel_scale, logical_h * pixel_scale)
            }
            ResolvedAspect::Locked { w, h } => {
                let canvas_h = step * pixel_scale;
                let canvas_w = ((canvas_h as u64 * w as u64) / h.max(1) as u64).max(1) as u32;
                (canvas_w, canvas_h)
            }
        };

        // 6. Viewport sizing.
        //    - Stretch: always fills the window (viewport == canvas == vp_w/vp_h, 1:1 blit).
        //    - Integer mode: viewport == canvas (1:1 blit), smaller than vp if it didn't divide evenly.
        //    - Scaled: viewport stays at vp_w/vp_h and canvas is stretched to fit (bilinear filter).
        let (final_vp_w, final_vp_h, display_scale) = match (self.aspect, self.scaling_mode) {
            (ResolvedAspect::Stretch, _) => (vp_w, vp_h, 1.0),
            (_, ScalingMode::Integer) => (canvas_w, canvas_h, 1.0),
            (_, ScalingMode::Scaled) => {
                let ds = vp_h as f32 / canvas_h as f32;
                (vp_w, vp_h, ds)
            }
        };

        // 8. Centre viewport in usable area.
        let vx = usable_origin.0 + (usable_size.0.saturating_sub(final_vp_w)) / 2;
        let vy = usable_origin.1 + (usable_size.1.saturating_sub(final_vp_h)) / 2;

        self.viewport_rect = Rect::new(vx, vy, vx + final_vp_w, vy + final_vp_h);
        self.canvas_size = (canvas_w, canvas_h);
        self.screen_size = (canvas_w as f32, canvas_h as f32);
        self.logical_size = (canvas_w as f32 / pixel_scale as f32, canvas_h as f32 / pixel_scale as f32);
        self.pixel_scale = pixel_scale;
        self.display_scale = display_scale;
        // `scale` mirrors the old `state.scale` semantics: the factor applied to logical coordinates
        // to produce canvas-pixel coordinates. Must equal `pixel_scale`, NOT the total including the
        // final blit's display_scale — game code renders into the canvas framebuffer, not the viewport.
        self.scale = pixel_scale as f32;
        self.effective_insets = effective_insets;
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}
