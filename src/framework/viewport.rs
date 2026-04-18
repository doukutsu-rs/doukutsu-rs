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
        let win = (self.window_size.0.max(1), self.window_size.1.max(1));
        let (usable_origin, usable_size, effective_insets) = self.compute_usable_area(win);
        let vp = fit_viewport(self.aspect, usable_size);
        let pixel_scale = self.compute_pixel_scale(vp.1);
        let Layout { canvas, final_vp, display_scale } = self.compute_layout(vp, pixel_scale);

        // Centre the final viewport within the usable area.
        let (fw, fh) = final_vp;
        let vx = usable_origin.0 + (usable_size.0.saturating_sub(fw)) / 2;
        let vy = usable_origin.1 + (usable_size.1.saturating_sub(fh)) / 2;

        self.viewport_rect = Rect::new(vx, vy, vx + fw, vy + fh);
        self.canvas_size = canvas;
        self.screen_size = (canvas.0 as f32, canvas.1 as f32);
        self.logical_size = (canvas.0 as f32 / pixel_scale as f32, canvas.1 as f32 / pixel_scale as f32);
        self.pixel_scale = pixel_scale;
        self.display_scale = display_scale;
        // `scale` mirrors the old `state.scale` semantics: the factor applied to logical coordinates
        // to produce canvas-pixel coordinates. Must equal `pixel_scale`, NOT the total including the
        // final blit's `display_scale` — game code renders into the canvas framebuffer, not the viewport.
        self.scale = pixel_scale as f32;
        self.effective_insets = effective_insets;
    }

    /// Subtract inset bars from the window (or not) based on `inset_mode`, returning the
    /// drawable region and the inset values that UI code should still apply.
    fn compute_usable_area(
        &self,
        (win_w, win_h): (u32, u32),
    ) -> ((u32, u32), (u32, u32), (f32, f32, f32, f32)) {
        match self.inset_mode {
            InsetMode::FillScreen => ((0, 0), (win_w, win_h), self.raw_insets),
            InsetMode::FitSafeArea => {
                let (il, it, ir, ib) = (
                    self.raw_insets.0.max(0.0),
                    self.raw_insets.1.max(0.0),
                    self.raw_insets.2.max(0.0),
                    self.raw_insets.3.max(0.0),
                );
                let origin = (il as u32, it as u32);
                let size = (
                    (win_w as i64 - il as i64 - ir as i64).max(1) as u32,
                    (win_h as i64 - it as i64 - ib as i64).max(1) as u32,
                );
                (origin, size, (0.0, 0.0, 0.0, 0.0))
            }
        }
    }

    /// Pick an integer pixel scale constrained to a multiple of `texture_scale`. Dividing by
    /// `tex` before rounding keeps the steps feeling consistent on CS+ (which can only do 2×,
    /// 4×, 6× …) — otherwise rounding against raw pixels would skip available scales unevenly.
    ///
    /// Stretch aspect forces a floor (integer-only) regardless of `scaling_mode`, because any
    /// fractional final blit would defeat the "fills the window pixel-for-pixel" intent.
    fn compute_pixel_scale(&self, vp_h: u32) -> u32 {
        let step = self.base_height_step.max(1);
        let tex = self.texture_scale.max(1);
        let ns_per_tex = vp_h as f32 / (step * tex) as f32;
        let k = match (self.aspect, self.scaling_mode) {
            (ResolvedAspect::Stretch, _) | (_, ScalingMode::Integer) => ns_per_tex.floor() as u32,
            (_, ScalingMode::Scaled) => (ns_per_tex + 0.5).floor() as u32,
        };
        k.max(1) * tex
    }

    /// Work out canvas size and the final viewport the canvas is blitted into. The three aspect
    /// modes have fundamentally different strategies, so they're handled as separate branches —
    /// collapsing them would require threading more state into helpers than it would save.
    fn compute_layout(&self, (vp_w, vp_h): (u32, u32), pixel_scale: u32) -> Layout {
        let step = self.base_height_step.max(1);
        match self.aspect {
            // Canvas == window, blit 1:1. Logical size may be fractional.
            ResolvedAspect::Stretch => Layout {
                canvas: (vp_w, vp_h),
                final_vp: (vp_w, vp_h),
                display_scale: 1.0,
            },

            // Height locked to canonical 240p; width follows the window so sprites in world-space
            // stay at their intended size. Integer mode renders 1:1; Scaled mode uniformly
            // stretches by `vp_h/canvas_h` and leaves a thin horizontal bar.
            ResolvedAspect::Unrestricted => {
                let canvas_h = step * pixel_scale;
                match self.scaling_mode {
                    ScalingMode::Integer => {
                        let logical_w = ((vp_w as f32 / pixel_scale as f32).round() as u32).max(1);
                        let canvas_w = logical_w * pixel_scale;
                        Layout { canvas: (canvas_w, canvas_h), final_vp: (canvas_w, canvas_h), display_scale: 1.0 }
                    }
                    ScalingMode::Scaled => {
                        // Pick canvas_w so `canvas_w * ds <= vp_w`, `ds = vp_h / canvas_h`, and
                        // canvas_w is a multiple of pixel_scale. Floor to avoid overshoot.
                        let target_w = vp_w as f32 * canvas_h as f32 / vp_h as f32;
                        let logical_w = ((target_w / pixel_scale as f32).floor() as u32).max(1);
                        let canvas_w = logical_w * pixel_scale;
                        let ds = vp_h as f32 / canvas_h as f32;
                        let final_w = (canvas_w as f32 * ds).round() as u32;
                        Layout { canvas: (canvas_w, canvas_h), final_vp: (final_w, vp_h), display_scale: ds }
                    }
                }
            }

            // Canvas matches the aspect exactly; vp already fits the aspect (from `fit_viewport`)
            // so the canvas→viewport scale is uniform in both modes.
            ResolvedAspect::Locked { w, h } => {
                let canvas_h = step * pixel_scale;
                let canvas_w = ((canvas_h as u64 * w as u64) / h.max(1) as u64).max(1) as u32;
                match self.scaling_mode {
                    ScalingMode::Integer => Layout {
                        canvas: (canvas_w, canvas_h),
                        final_vp: (canvas_w, canvas_h),
                        display_scale: 1.0,
                    },
                    ScalingMode::Scaled => {
                        let ds = vp_h as f32 / canvas_h as f32;
                        let final_w = (canvas_w as f32 * ds).round() as u32;
                        Layout { canvas: (canvas_w, canvas_h), final_vp: (final_w, vp_h), display_scale: ds }
                    }
                }
            }
        }
    }
}

/// Intermediate value returned by [`Viewport::compute_layout`].
struct Layout {
    canvas: (u32, u32),
    final_vp: (u32, u32),
    display_scale: f32,
}

/// Fit a rectangle of the requested aspect into `usable_size`. Unrestricted and Stretch both
/// take the full usable area; Locked letterboxes/pillarboxes as needed.
fn fit_viewport(aspect: ResolvedAspect, usable_size: (u32, u32)) -> (u32, u32) {
    match aspect {
        ResolvedAspect::Unrestricted | ResolvedAspect::Stretch => usable_size,
        ResolvedAspect::Locked { w, h } => {
            let ratio = w as f64 / h as f64;
            let by_width_height = (usable_size.0 as f64) / ratio;
            if by_width_height <= usable_size.1 as f64 {
                (usable_size.0, by_width_height.floor().max(1.0) as u32)
            } else {
                let by_height_width = (usable_size.1 as f64) * ratio;
                (by_height_width.floor().max(1.0) as u32, usable_size.1)
            }
        }
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}
