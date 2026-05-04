//! Tick + present clocks share a single master anchor (`deadline = anchor + period * index`)
//! so an integer ratio between the two rates produces a deterministic per-frame tick count
//! instead of bobbling between e.g. 0 and 2 as the clocks drift independently.
use std::time::{Duration, Instant};

use crate::common::Color;
use crate::framework::backend::{BackendShader, VertexData};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::framework::graphics::IndexData;

const MAX_LAG_TICKS: u32 = 4;
const MAX_LEAD: Duration = Duration::from_secs(1);
/// ~52 days at 60Hz — re-anchor below this to keep `index * period` from growing without bound.
const REANCHOR_INDEX: u32 = 1 << 25;
const SCHEDULER_MARGIN: Duration = Duration::from_micros(500);
const MAX_SPIN: Duration = Duration::from_millis(1);
pub const MAX_CATCHUP: u32 = 10;

pub const HISTORY: usize = 240;

#[derive(Copy, Clone, Default)]
pub struct FrameSample {
    pub interval_ns: u64,
    pub swap_latency_ns: u64,
    pub catchup_loops: u32,
}

pub struct FramePacer {
    master_anchor: Instant,

    tick_index: u32,
    tick_delta: Duration,
    last_tick_instant: Instant,

    present_index: u32,
    present_period: Duration,

    swap_latency_ema: Duration,
    measured_period_ema: Duration,
    last_present: Option<Instant>,

    history: [FrameSample; HISTORY],
    history_head: usize,
    history_filled: usize,
}

impl FramePacer {
    pub fn new() -> Self {
        let now = Instant::now();
        let default_delta = Duration::from_nanos(1_000_000_000 / 60);
        Self {
            master_anchor: now,
            tick_index: 0,
            tick_delta: default_delta,
            last_tick_instant: now,
            present_index: 0,
            present_period: default_delta,
            swap_latency_ema: Duration::ZERO,
            measured_period_ema: Duration::ZERO,
            last_present: None,
            history: [FrameSample::default(); HISTORY],
            history_head: 0,
            history_filled: 0,
        }
    }

    pub fn reset(&mut self) {
        let now = Instant::now();
        self.master_anchor = now;
        self.tick_index = 0;
        self.last_tick_instant = now;
        self.present_index = 0;
        self.last_present = None;
    }

    pub fn record_present(&mut self, loops: u32) {
        let now = Instant::now();
        let interval_ns = match self.last_present {
            Some(prev) => now.saturating_duration_since(prev).as_nanos().min(u64::MAX as u128) as u64,
            None => 0,
        };
        if interval_ns > 0 {
            let m = Duration::from_nanos(interval_ns).min(Duration::from_millis(250));
            self.measured_period_ema = (self.measured_period_ema * 7 + m) / 8;
        }
        self.last_present = Some(now);
        let swap_latency_ns = self.swap_latency_ema.as_nanos().min(u64::MAX as u128) as u64;
        self.history[self.history_head] = FrameSample { interval_ns, swap_latency_ns, catchup_loops: loops };
        self.history_head = (self.history_head + 1) % HISTORY;
        if self.history_filled < HISTORY {
            self.history_filled += 1;
        }
    }

    pub fn history_iter(&self) -> impl Iterator<Item = FrameSample> + '_ {
        let len = self.history_filled;
        let head = self.history_head;
        let start = (head + HISTORY - len) % HISTORY;
        (0..len).map(move |i| self.history[(start + i) % HISTORY])
    }

    pub fn present_period(&self) -> Duration {
        self.present_period
    }

    pub fn measured_period(&self) -> Duration {
        self.measured_period_ema
    }

    pub fn tick_delta(&self) -> Duration {
        self.tick_delta
    }

    pub fn last_tick_instant(&self) -> Instant {
        self.last_tick_instant
    }

    pub fn swap_latency(&self) -> Duration {
        self.swap_latency_ema
    }

    /// Time the next-up sim-tick deadline will fire.
    pub fn next_tick_instant(&self) -> Instant {
        self.tick_deadline(1)
    }

    /// Time the next present is scheduled to occur.
    pub fn next_present_instant(&self) -> Instant {
        self.present_deadline(1)
    }

    pub fn set_tick_delta(&mut self, delta: Duration) {
        if delta == self.tick_delta || delta.is_zero() {
            return;
        }
        self.tick_delta = delta;
        let elapsed_ns = Instant::now().saturating_duration_since(self.master_anchor).as_nanos();
        let delta_ns = delta.as_nanos().max(1);
        let idx = (elapsed_ns / delta_ns).min(u32::MAX as u128) as u32;
        self.tick_index = idx;
        self.last_tick_instant = self
            .tick_delta
            .checked_mul(idx)
            .and_then(|d| self.master_anchor.checked_add(d))
            .unwrap_or(self.master_anchor);
    }

    pub fn set_present_period(&mut self, period: Duration) {
        if period == self.present_period || period.is_zero() {
            return;
        }
        self.present_period = period;
        let elapsed_ns = Instant::now().saturating_duration_since(self.master_anchor).as_nanos();
        let period_ns = period.as_nanos().max(1);
        let idx = (elapsed_ns / period_ns).min(u32::MAX as u128) as u32;
        self.present_index = idx;
    }

    /// Runs sim ticks whose deadlines are <= `target`. Returns 0..=MAX_CATCHUP.
    pub fn advance_ticks_until(&mut self, target: Instant) -> u32 {
        if self.tick_delta.is_zero() {
            return 0;
        }
        let mut deadline = self.tick_deadline(1);

        let now = Instant::now();
        let lag_cap = self.tick_delta.checked_mul(MAX_LAG_TICKS).unwrap_or(MAX_LEAD);
        if now.saturating_duration_since(deadline) > lag_cap
            || deadline.saturating_duration_since(now) > MAX_LEAD
        {
            self.hard_reanchor(now);
            deadline = self.tick_deadline(1);
        }

        let mut loops = 0u32;
        while deadline <= target && loops < MAX_CATCHUP {
            self.tick_index = self.tick_index.saturating_add(1);
            self.last_tick_instant = deadline;
            loops += 1;
            deadline = self.tick_deadline(1);
        }

        if self.tick_index >= REANCHOR_INDEX || self.present_index >= REANCHOR_INDEX {
            self.phased_reanchor();
        }

        loops
    }

    pub fn wait_for_present(&mut self) {
        if self.present_period.is_zero() {
            return;
        }
        let now = Instant::now();
        let mut deadline = self.present_deadline(1);

        let lag_cap = self.present_period.checked_mul(MAX_LAG_TICKS).unwrap_or(MAX_LEAD);
        if now.saturating_duration_since(deadline) > lag_cap
            || deadline.saturating_duration_since(now) > MAX_LEAD
        {
            self.hard_reanchor(now);
            deadline = self.present_deadline(1);
        }

        let lead = self.swap_latency_ema.min(self.present_period / 2);
        let target = deadline.checked_sub(lead).unwrap_or(deadline);
        let target_minus_margin = target.checked_sub(SCHEDULER_MARGIN).unwrap_or(target);

        let now2 = Instant::now();
        if target_minus_margin > now2 {
            std::thread::sleep(target_minus_margin - now2);
        }
        let spin_until = {
            let cap = Instant::now() + MAX_SPIN;
            if target < cap { target } else { cap }
        };
        while Instant::now() < spin_until {
            std::hint::spin_loop();
        }

        self.present_index = self.present_index.saturating_add(1);
        if self.present_index >= REANCHOR_INDEX || self.tick_index >= REANCHOR_INDEX {
            self.phased_reanchor();
        }
    }

    /// Caller MUST skip this in V-Sync — the blocking swap measures vblank wait,
    /// not present latency, and would poison the EMA used by VRR pacing.
    pub fn record_swap_latency(&mut self, measured: Duration) {
        let cap = if self.tick_delta.is_zero() { Duration::from_millis(50) } else { self.tick_delta };
        self.swap_latency_ema = (self.swap_latency_ema * 7 + measured.min(cap)) / 8;
    }

    pub fn predicted_present_instant(&self) -> Instant {
        Instant::now() + self.swap_latency_ema
    }

    pub fn interpolation_alpha(&self) -> f64 {
        if self.tick_delta.is_zero() {
            return 0.0;
        }
        let predicted = self.predicted_present_instant();
        let since = predicted.saturating_duration_since(self.last_tick_instant);
        let alpha = (since.as_nanos() as f64) / (self.tick_delta.as_nanos() as f64);
        alpha.clamp(0.0, 1.0)
    }

    /// Coords are logical px; `pixel_scale` lifts them into the canvas-pixel space.
    pub fn draw_debug_chart(
        &self,
        ctx: &mut Context,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        pixel_scale: f32,
        target_period: Duration,
    ) -> GameResult {
        let bg = Color::from_rgba(0, 0, 0, 160);
        let mid_line = Color::from_rgba(255, 255, 255, 110);
        let bar_ok = Color::from_rgba(80, 220, 80, 230);
        let bar_warn = Color::from_rgba(240, 200, 80, 230);
        let bar_bad = Color::from_rgba(240, 80, 80, 230);
        let latency_color = Color::from_rgba(120, 180, 255, 220);
        let loops_color = Color::from_rgba(255, 160, 80, 230);

        let chart_h = (height * 0.7).max(24.0);
        let loops_strip_y = y + chart_h + 2.0;
        let loops_h = (y + height) - loops_strip_y;
        let mid_y = y + chart_h * 0.5;

        let target_ns = target_period.as_nanos().max(1) as f32;
        let scale_ns = target_ns * 2.0;

        let count = self.history_filled.min(HISTORY);
        let bar_w = if count > 0 { width / count as f32 } else { 1.0 };

        let mut vertices: Vec<VertexData> = Vec::with_capacity(4 * (2 + 3 * count));
        let mut indices: Vec<u16> = Vec::with_capacity(6 * (2 + 3 * count));

        let mut push_quad = |vertices: &mut Vec<VertexData>,
                             indices: &mut Vec<u16>,
                             lx: f32,
                             ly: f32,
                             rx: f32,
                             by: f32,
                             color: Color| {
            if rx <= lx || by <= ly || vertices.len() + 4 > u16::MAX as usize {
                return;
            }
            let base = vertices.len() as u16;
            let c = color.to_rgba();
            let (lx, ly, rx, by) =
                (lx * pixel_scale, ly * pixel_scale, rx * pixel_scale, by * pixel_scale);
            vertices.push(VertexData { position: (lx, by), uv: (0.0, 0.0), color: c });
            vertices.push(VertexData { position: (lx, ly), uv: (0.0, 0.0), color: c });
            vertices.push(VertexData { position: (rx, ly), uv: (0.0, 0.0), color: c });
            vertices.push(VertexData { position: (rx, by), uv: (0.0, 0.0), color: c });
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        };

        push_quad(&mut vertices, &mut indices, x, y, x + width, y + height, bg);
        push_quad(&mut vertices, &mut indices, x, mid_y, x + width, mid_y + 1.0, mid_line);

        for (i, sample) in self.history_iter().enumerate() {
            let bx = x + i as f32 * bar_w;
            let bx2 = bx + bar_w;
            if sample.interval_ns == 0 {
                continue;
            }
            let signed = sample.interval_ns as i64 - target_ns as i64;
            let abs = signed.unsigned_abs() as f32;
            let h_pixels = ((abs / scale_ns) * (chart_h * 0.5)).min(chart_h * 0.5);
            let rel = abs / target_ns;
            let color = if rel < 0.10 { bar_ok } else if rel < 0.30 { bar_warn } else { bar_bad };
            let (top, bot) =
                if signed >= 0 { (mid_y - h_pixels, mid_y) } else { (mid_y, mid_y + h_pixels) };
            push_quad(&mut vertices, &mut indices, bx, top, bx2, bot, color);

            let lat_h = ((sample.swap_latency_ns as f32 / target_ns) * (chart_h * 0.5)).min(chart_h * 0.5);
            if lat_h >= 1.0 {
                let ly = y + chart_h - lat_h;
                push_quad(&mut vertices, &mut indices, bx, ly, bx2, ly + 1.0, latency_color);
            }

            if loops_h > 0.0 && sample.catchup_loops > 0 {
                let lh = ((sample.catchup_loops as f32 / MAX_CATCHUP as f32) * loops_h).min(loops_h);
                let ly = y + height - lh;
                push_quad(&mut vertices, &mut indices, bx, ly, bx2, y + height, loops_color);
            }
        }

        if !indices.is_empty() {
            graphics::draw_triangles_indexed(
                ctx,
                &vertices,
                IndexData::UShort(&indices),
                None,
                BackendShader::Fill,
            )?;
        }

        Ok(())
    }

    /// Stats over the recorded history compared against `target_period`:
    /// (avg_interval_ns, max_abs_jitter_ns, swap_latency_ema_us).
    pub fn debug_stats_against(&self, target_period: Duration) -> (u64, u64, u64) {
        let target_ns = target_period.as_nanos().max(1) as i64;
        let mut sum: u128 = 0;
        let mut count: u128 = 0;
        let mut max_jitter: u64 = 0;
        for s in self.history_iter() {
            if s.interval_ns == 0 {
                continue;
            }
            sum += s.interval_ns as u128;
            count += 1;
            let j = (s.interval_ns as i64 - target_ns).unsigned_abs();
            if j > max_jitter {
                max_jitter = j;
            }
        }
        let avg = if count > 0 { (sum / count) as u64 } else { 0 };
        let lat_us = (self.swap_latency_ema.as_nanos() / 1000).min(u64::MAX as u128) as u64;
        (avg, max_jitter, lat_us)
    }

    fn tick_deadline(&self, idx_offset: u32) -> Instant {
        let idx = self.tick_index.saturating_add(idx_offset);
        self.tick_delta
            .checked_mul(idx)
            .and_then(|d| self.master_anchor.checked_add(d))
            .unwrap_or(self.master_anchor)
    }

    fn present_deadline(&self, idx_offset: u32) -> Instant {
        let idx = self.present_index.saturating_add(idx_offset);
        self.present_period
            .checked_mul(idx)
            .and_then(|d| self.master_anchor.checked_add(d))
            .unwrap_or(self.master_anchor)
    }

    fn hard_reanchor(&mut self, now: Instant) {
        self.master_anchor = now;
        self.tick_index = 0;
        self.present_index = 0;
        self.last_tick_instant = now;
    }

    fn phased_reanchor(&mut self) {
        let now = Instant::now();
        // Snap to the larger period: if the two are integer-related the smaller phase
        // stays aligned automatically.
        let snap = self.tick_delta.max(self.present_period);
        let phase_ns = if snap.is_zero() {
            0
        } else {
            now.saturating_duration_since(self.master_anchor).as_nanos() % snap.as_nanos()
        };
        self.master_anchor = now.checked_sub(Duration::from_nanos(phase_ns as u64)).unwrap_or(now);
        self.tick_index = 0;
        self.present_index = 0;
        // last_tick_instant stays at its real value (close to `now`).
    }
}

impl Default for FramePacer {
    fn default() -> Self {
        Self::new()
    }
}
