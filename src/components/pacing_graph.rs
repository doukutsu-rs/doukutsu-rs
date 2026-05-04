use std::time::Duration;

use crate::{
    common::Rect,
    framework::{context::Context, error::GameResult, frame_pacer::FramePacer, graphics},
    game::{settings::VSyncMode, shared_game_state::SharedGameState},
    graphics::font::Font,
};

pub struct PacingGraph;

impl PacingGraph {
    pub fn draw(&self, pacer: &FramePacer, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if !state.settings.pacing_debug {
            return Ok(());
        }

        let canvas_w = ctx.viewport.logical_size.0;
        let chart_w = (canvas_w - 16.0).clamp(40.0, 120.0);
        let chart_h = 40.0_f32;
        let x = 8.0_f32;
        let y = 24.0;
        let scale = state.scale;

        // VRR modes pace against `present_period`; everything else has no explicit
        // target so we lean on the measured EMA (tick_delta until it converges).
        let display_target: Duration = match state.settings.vsync_mode {
            VSyncMode::VRRTickSync1x
            | VSyncMode::VRRTickSync2x
            | VSyncMode::VRRTickSync3x
            | VSyncMode::VRRTickSyncAuto => pacer.present_period(),
            _ => {
                let m = pacer.measured_period();
                if m.is_zero() {
                    pacer.tick_delta()
                } else {
                    m
                }
            }
        };

        let clip = Rect::new_size(
            (x * scale) as isize,
            (y * scale) as isize,
            (chart_w * scale).ceil() as isize,
            (chart_h * scale).ceil() as isize,
        );
        graphics::set_clip_rect(ctx, Some(clip))?;
        let r = pacer.draw_debug_chart(ctx, x, y, chart_w, chart_h, scale, display_target);
        graphics::set_clip_rect(ctx, None)?;
        r?;

        let (avg_ns, max_jitter_ns, lat_us) = pacer.debug_stats_against(display_target);
        let avg_ms = avg_ns as f32 / 1_000_000.0;
        let jit_ms = max_jitter_ns as f32 / 1_000_000.0;
        let lat_ms = lat_us as f32 / 1_000.0;
        let target_ms = display_target.as_nanos() as f32 / 1_000_000.0;

        let target_ns = display_target.as_nanos().max(1) as f32;
        let jit_rel = (max_jitter_ns as f32) / target_ns;
        let stable = (80, 220, 80, 255);
        let warn = (240, 200, 80, 255);
        let bad = (240, 80, 80, 255);
        let swap_c = (120, 180, 255, 255);
        let header_c = (200, 200, 200, 255);

        let segments: [(&str, (u8, u8, u8, u8)); 4] = [
            (&format!("tgt {:.2}", target_ms), header_c),
            (&format!("avg {:.2}", avg_ms), stable),
            (
                &format!("jit {:.2}", jit_ms),
                if jit_rel < 0.10 {
                    stable
                } else if jit_rel < 0.30 {
                    warn
                } else {
                    bad
                },
            ),
            (&format!("swap {:.2}", lat_ms), swap_c),
        ];
        let label_y = y + chart_h + 1.0;
        let mut cur_x = x;
        for (text, color) in segments.iter() {
            let builder = state.font.builder().scale(0.5);
            let w = builder.compute_width(text) * 0.5;
            state.font.builder().position(cur_x, label_y).scale(0.5).shadow(true).color(*color).draw(
                text,
                ctx,
                &state.constants,
                &mut state.texture_set,
            )?;
            cur_x += w + 4.0;
        }

        Ok(())
    }
}
