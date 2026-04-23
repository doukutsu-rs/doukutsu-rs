use crate::common::Rect;

/// Framework-level aspect policy. The game layer resolves its richer
/// `AspectRatio` (edition-aware, parse-from-string) down to this before
/// writing it to the viewport.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedAspect {
    Unrestricted,
    Stretch,
    Locked { w: u32, h: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScalingMode {
    Integer,
    Scaled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InsetMode {
    FillScreen,
    FitSafeArea,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DebugOverlayMode {
    Viewported,
    Unconstrained,
}

/// Single source of truth for viewport geometry. Lives on
/// [`Context`](crate::framework::context::Context).
///
/// Backends write `window_size`, `dpi_scale`, `raw_insets`; the game layer
/// writes the configuration fields. [`Viewport::recompute`] refreshes the
/// computed outputs from either kind of change.
pub struct Viewport {
    // --- Raw inputs (written by the backend) ---
    pub window_size: (u32, u32),
    /// `drawable_size / window.size()` — `(1.0, 1.0)` off HiDPI.
    pub dpi_scale: (f32, f32),
    /// left, top, right, bottom, in physical pixels.
    pub raw_insets: (f32, f32, f32, f32),

    // --- Computed outputs (refreshed by recompute) ---
    pub viewport_rect: Rect<u32>,
    pub canvas_size: (u32, u32),
    pub screen_size: (f32, f32),
    pub logical_size: (f32, f32),
    pub pixel_scale: u32,
    pub display_scale: f32,
    /// Factor applied to logical coords to reach canvas-pixel coords. Mirrors
    /// `pixel_scale` — NOT the final blit's `display_scale` — since gameplay
    /// renders into the canvas framebuffer, not the viewport.
    pub scale: f32,
    pub effective_insets: (f32, f32, f32, f32),

    // --- Configuration (written by the game) ---
    pub aspect: ResolvedAspect,
    pub scaling_mode: ScalingMode,
    pub inset_mode: InsetMode,
    pub debug_overlay_mode: DebugOverlayMode,
    pub base_height_step: u32,
    /// Minimum granularity of `pixel_scale`. 1 for freeware/NXEngine, 2 for
    /// CS+ whose 2x-density atlases require even multiples.
    pub texture_scale: u32,
}

impl Viewport {
    pub fn new() -> Viewport {
        let mut vp = Viewport {
            window_size: (640, 480),
            dpi_scale: (1.0, 1.0),
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
            debug_overlay_mode: DebugOverlayMode::Unconstrained,
            base_height_step: 240,
            texture_scale: 1,
        };
        vp.recompute();
        vp
    }

    pub fn recompute(&mut self) {
        let win = (self.window_size.0.max(1), self.window_size.1.max(1));
        let (usable_origin, usable_size, effective_insets) = self.compute_usable_area(win);
        let vp = fit_viewport(self.aspect, usable_size);
        let pixel_scale = self.compute_pixel_scale(vp.1);
        let Layout { canvas, final_vp, display_scale } = self.compute_layout(vp, pixel_scale);

        let (fw, fh) = final_vp;
        let vx = usable_origin.0 + (usable_size.0.saturating_sub(fw)) / 2;
        let vy = usable_origin.1 + (usable_size.1.saturating_sub(fh)) / 2;

        self.viewport_rect = Rect::new(vx, vy, vx + fw, vy + fh);
        self.canvas_size = canvas;
        self.screen_size = (canvas.0 as f32, canvas.1 as f32);
        self.logical_size = (canvas.0 as f32 / pixel_scale as f32, canvas.1 as f32 / pixel_scale as f32);
        self.pixel_scale = pixel_scale;
        self.display_scale = display_scale;
        self.scale = pixel_scale as f32;
        self.effective_insets = effective_insets;
    }

    pub fn logical_to_physical(&self, lx: f32, ly: f32) -> (f32, f32) {
        (lx * self.dpi_scale.0, ly * self.dpi_scale.1)
    }

    /// Physical window coords → canvas logical coords. Letterbox/pillarbox
    /// pixels produce out-of-range results; callers filter as needed.
    pub fn window_to_canvas(&self, px: f32, py: f32) -> (f32, f32) {
        let r = self.viewport_rect;
        let rw = (r.right.saturating_sub(r.left)).max(1) as f32;
        let rh = (r.bottom.saturating_sub(r.top)).max(1) as f32;
        (
            (px - r.left as f32) * self.logical_size.0 / rw,
            (py - r.top as f32) * self.logical_size.1 / rh,
        )
    }

    pub fn window_to_overlay(&self, px: f32, py: f32) -> (f32, f32) {
        match self.debug_overlay_mode {
            DebugOverlayMode::Viewported => self.window_to_canvas(px, py),
            DebugOverlayMode::Unconstrained => (px, py),
        }
    }

    pub fn overlay_display_size(&self) -> (f32, f32) {
        match self.debug_overlay_mode {
            DebugOverlayMode::Viewported => self.logical_size,
            DebugOverlayMode::Unconstrained => (self.window_size.0 as f32, self.window_size.1 as f32),
        }
    }

    pub fn overlay_framebuffer_scale(&self) -> (f32, f32) {
        match self.debug_overlay_mode {
            DebugOverlayMode::Viewported => {
                (self.display_scale * self.dpi_scale.0, self.display_scale * self.dpi_scale.1)
            }
            DebugOverlayMode::Unconstrained => (1.0, 1.0),
        }
    }

    pub fn overlay_viewport_rect(&self) -> Option<Rect<u32>> {
        match self.debug_overlay_mode {
            DebugOverlayMode::Viewported => Some(self.viewport_rect),
            DebugOverlayMode::Unconstrained => None,
        }
    }

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

    /// Divide by `tex` before rounding so CS+ (2x-only scales) steps through
    /// 2×/4×/6× without skipping available scales unevenly. Stretch forces a
    /// floor regardless of `scaling_mode` — a fractional final blit would
    /// defeat "fills the window pixel-for-pixel".
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

    fn compute_layout(&self, (vp_w, vp_h): (u32, u32), pixel_scale: u32) -> Layout {
        let step = self.base_height_step.max(1);
        match self.aspect {
            ResolvedAspect::Stretch => Layout {
                canvas: (vp_w, vp_h),
                final_vp: (vp_w, vp_h),
                display_scale: 1.0,
            },

            // Height locked to canonical 240p; width follows the window so world-space
            // sprites stay at their intended size. Scaled mode stretches by vp_h/canvas_h
            // and leaves a horizontal bar.
            ResolvedAspect::Unrestricted => {
                let canvas_h = step * pixel_scale;
                match self.scaling_mode {
                    ScalingMode::Integer => {
                        let logical_w = ((vp_w as f32 / pixel_scale as f32).round() as u32).max(1);
                        let canvas_w = logical_w * pixel_scale;
                        Layout { canvas: (canvas_w, canvas_h), final_vp: (canvas_w, canvas_h), display_scale: 1.0 }
                    }
                    ScalingMode::Scaled => {
                        // Floor to avoid overshoot: canvas_w * ds ≤ vp_w.
                        let target_w = vp_w as f32 * canvas_h as f32 / vp_h as f32;
                        let logical_w = ((target_w / pixel_scale as f32).floor() as u32).max(1);
                        let canvas_w = logical_w * pixel_scale;
                        let ds = vp_h as f32 / canvas_h as f32;
                        let final_w = (canvas_w as f32 * ds).round() as u32;
                        Layout { canvas: (canvas_w, canvas_h), final_vp: (final_w, vp_h), display_scale: ds }
                    }
                }
            }

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

struct Layout {
    canvas: (u32, u32),
    final_vp: (u32, u32),
    display_scale: f32,
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn vp_with_rect(vp_rect: Rect<u32>, logical: (f32, f32), window: (u32, u32)) -> Viewport {
        let mut vp = Viewport::new();
        vp.window_size = window;
        vp.viewport_rect = vp_rect;
        vp.logical_size = logical;
        vp.screen_size = logical;
        vp
    }

    #[test]
    fn canvas_transform_maps_vp_corners() {
        let vp = vp_with_rect(Rect::new(50, 0, 370, 240), (320.0, 240.0), (420, 240));

        let (x, y) = vp.window_to_canvas(50.0, 0.0);
        assert!((x - 0.0).abs() < 1e-3);
        assert!((y - 0.0).abs() < 1e-3);

        let (x, y) = vp.window_to_canvas(370.0, 240.0);
        assert!((x - 320.0).abs() < 1e-3);
        assert!((y - 240.0).abs() < 1e-3);
    }

    #[test]
    fn canvas_transform_letterbox_clicks_are_out_of_range() {
        let vp = vp_with_rect(Rect::new(100, 0, 420, 240), (320.0, 240.0), (520, 240));
        let (x, _y) = vp.window_to_canvas(0.0, 0.0);
        assert!(x < 0.0, "letterbox click should map outside canvas, got x={}", x);
    }

    #[test]
    fn overlay_mode_switches_transform() {
        let mut vp = vp_with_rect(Rect::new(50, 0, 370, 240), (320.0, 240.0), (420, 240));
        vp.debug_overlay_mode = DebugOverlayMode::Unconstrained;
        let (x, y) = vp.window_to_overlay(123.0, 45.0);
        assert_eq!((x, y), (123.0, 45.0));

        vp.debug_overlay_mode = DebugOverlayMode::Viewported;
        let (x, _y) = vp.window_to_overlay(50.0, 0.0);
        assert!((x - 0.0).abs() < 1e-3);
    }

    #[test]
    fn dpi_scale_logical_to_physical() {
        let mut vp = Viewport::new();
        vp.dpi_scale = (2.0, 2.0);
        assert_eq!(vp.logical_to_physical(100.0, 50.0), (200.0, 100.0));
    }
}
