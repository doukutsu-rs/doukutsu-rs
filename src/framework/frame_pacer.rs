//! Backend-agnostic frame pacing.
//!
//! Maintains an anchored tick clock (sim-side) and an anchored present clock (display-side),
//! both expressed as `anchor + period * index` so deadlines stay drift-free regardless of how
//! many frames have run. Re-anchors on speed changes, large clock drift (suspend/resume,
//! NTP step-back, non-monotonic `Instant`), and on a sub-tick high-water index.
//!
//! Also tracks an EMA of swap+present latency so interpolation can predict the actual
//! display moment, and so VRR pacing can lead the deadline by the measured latency.
use std::time::{Duration, Instant};

/// Maximum ticks of clock lag we'll absorb via catch-up before re-anchoring.
const MAX_LAG_TICKS: u32 = 4;
/// If a deadline is more than this far in the future relative to `now`, treat it
/// as a clock regression / suspend-resume artifact and re-anchor.
const MAX_LEAD: Duration = Duration::from_secs(1);
/// Sub-tick re-anchor floor — re-anchor when index reaches this, to keep
/// `index * period` from growing without bound. ~52 days at 60 Hz.
const REANCHOR_INDEX: u32 = 1 << 25;
/// Slack subtracted from the present deadline before sleeping; covers OS scheduler wake-up jitter.
const SCHEDULER_MARGIN: Duration = Duration::from_micros(500);
/// Hard cap on the spin-wait window that bridges the residual sleep error.
const MAX_SPIN: Duration = Duration::from_millis(1);
/// Maximum sim ticks to run per `advance_ticks` call (death-spiral guard).
pub const MAX_CATCHUP: u32 = 10;

pub struct FramePacer {
    tick_anchor: Instant,
    tick_index: u32,
    tick_delta: Duration,
    last_tick_instant: Instant,
    present_anchor: Instant,
    present_index: u32,
    present_period: Duration,
    swap_latency_ema: Duration,
}

impl FramePacer {
    pub fn new() -> Self {
        let now = Instant::now();
        let default_delta = Duration::from_nanos(1_000_000_000 / 60);
        Self {
            tick_anchor: now,
            tick_index: 0,
            tick_delta: default_delta,
            last_tick_instant: now,
            present_anchor: now,
            present_index: 0,
            present_period: default_delta,
            swap_latency_ema: Duration::ZERO,
        }
    }

    /// Reset all anchors to `now`. Call on focus gain, scene change, or after a known stall.
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.tick_anchor = now;
        self.tick_index = 0;
        self.last_tick_instant = now;
        self.present_anchor = now;
        self.present_index = 0;
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

    /// Update the sim tick period. Phase-preserving re-anchor if the period changed.
    pub fn set_tick_delta(&mut self, delta: Duration) {
        if delta != self.tick_delta && !delta.is_zero() {
            self.tick_delta = delta;
            self.reanchor_ticks_phased(self.last_tick_instant);
        }
    }

    /// Update the VRR present period. Phase-preserving re-anchor if it changed.
    pub fn set_present_period(&mut self, period: Duration) {
        if period != self.present_period && !period.is_zero() {
            self.present_period = period;
            self.reanchor_present_phased(Instant::now());
        }
    }

    /// Compute how many sim ticks should run this frame. Advances internal counters
    /// and `last_tick_instant`. Caller is expected to invoke its tick callback this
    /// many times. Returns 0..=MAX_CATCHUP.
    pub fn advance_ticks(&mut self) -> u32 {
        if self.tick_delta.is_zero() {
            return 0;
        }
        let now = Instant::now();
        let mut deadline = self.tick_deadline(1);

        let lag_cap = self.tick_delta.checked_mul(MAX_LAG_TICKS).unwrap_or(MAX_LEAD);
        if now.saturating_duration_since(deadline) > lag_cap
            || deadline.saturating_duration_since(now) > MAX_LEAD
        {
            self.tick_anchor = now;
            self.tick_index = 0;
            self.last_tick_instant = now;
            deadline = self.tick_deadline(1);
        }

        let mut loops = 0u32;
        while now >= deadline && loops < MAX_CATCHUP {
            self.tick_index = self.tick_index.saturating_add(1);
            self.last_tick_instant = deadline;
            loops += 1;
            deadline = self.tick_deadline(1);
        }

        if self.tick_index >= REANCHOR_INDEX {
            self.reanchor_ticks_phased(self.last_tick_instant);
        }

        loops
    }

    /// Block until just before the next present deadline. Uses recorded swap latency
    /// to lead the deadline so the frame appears at the intended instant.
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
            self.present_anchor = now;
            self.present_index = 0;
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
        if self.present_index >= REANCHOR_INDEX {
            self.reanchor_present_phased(Instant::now());
        }
    }

    /// Record measured swap+present latency. Call after the swap-buffers / finalize call.
    pub fn record_swap_latency(&mut self, measured: Duration) {
        // Cap at one tick so a single stall can't poison the lead.
        let cap = if self.tick_delta.is_zero() { Duration::from_millis(50) } else { self.tick_delta };
        let measured = measured.min(cap);
        self.swap_latency_ema = (self.swap_latency_ema * 7 + measured) / 8;
    }

    /// Predicted instant the next presented frame will appear on the display.
    pub fn predicted_present_instant(&self) -> Instant {
        Instant::now() + self.swap_latency_ema
    }

    /// Interpolation alpha in [0, 1] — fraction of the current tick at the predicted
    /// present instant. Returns 0 if no tick has run yet or `tick_delta` is zero.
    pub fn interpolation_alpha(&self) -> f64 {
        if self.tick_delta.is_zero() {
            return 0.0;
        }
        let predicted = self.predicted_present_instant();
        let since = predicted.saturating_duration_since(self.last_tick_instant);
        let alpha = (since.as_nanos() as f64) / (self.tick_delta.as_nanos() as f64);
        alpha.clamp(0.0, 1.0)
    }

    fn tick_deadline(&self, idx_offset: u32) -> Instant {
        let idx = self.tick_index.saturating_add(idx_offset);
        self.tick_delta
            .checked_mul(idx)
            .and_then(|d| self.tick_anchor.checked_add(d))
            .unwrap_or(self.tick_anchor)
    }

    fn present_deadline(&self, idx_offset: u32) -> Instant {
        let idx = self.present_index.saturating_add(idx_offset);
        self.present_period
            .checked_mul(idx)
            .and_then(|d| self.present_anchor.checked_add(d))
            .unwrap_or(self.present_anchor)
    }

    fn reanchor_ticks_phased(&mut self, now: Instant) {
        let phase_ns = if self.tick_delta.is_zero() {
            0
        } else {
            now.saturating_duration_since(self.tick_anchor).as_nanos() % self.tick_delta.as_nanos()
        };
        self.tick_anchor = now.checked_sub(Duration::from_nanos(phase_ns as u64)).unwrap_or(now);
        self.tick_index = 0;
    }

    fn reanchor_present_phased(&mut self, now: Instant) {
        let phase_ns = if self.present_period.is_zero() {
            0
        } else {
            now.saturating_duration_since(self.present_anchor).as_nanos() % self.present_period.as_nanos()
        };
        self.present_anchor = now.checked_sub(Duration::from_nanos(phase_ns as u64)).unwrap_or(now);
        self.present_index = 0;
    }
}

impl Default for FramePacer {
    fn default() -> Self {
        Self::new()
    }
}
