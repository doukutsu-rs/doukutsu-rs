use crate::fir::{FIR, FIR_STEP};

const FIR_CENTER: usize = (FIR.len() - 1) / 2;
const MAX_SPACING: usize = FIR_STEP as usize;
// Caps the filter length when heavily downsampling, past 8:1 some aliasing is accepted.
const MIN_SPACING: usize = MAX_SPACING / 8;
const MAX_HALF_TAPS: usize = FIR_CENTER.div_ceil(MIN_SPACING);

/// Number of filter phases per input sample, coefficients between phases are interpolated linearly.
const PHASES: usize = 64;

/// Silent or wrapped samples required on both sides of sample data read by the interpolators.
pub const GUARD: usize = MAX_HALF_TAPS + 10;

/// Windowed sinc coefficients for every phase of one resampling ratio.
pub struct PhaseBank {
    half: usize,
    taps: usize,
    rows: Box<[f32]>,
}

impl PhaseBank {
    /// `spacing` is the distance in FIR table entries between two input samples.
    /// FIR_STEP keeps the prototype cutoff at the input Nyquist frequency, smaller values lower it.
    fn new(spacing: usize) -> PhaseBank {
        let half = FIR_CENTER.div_ceil(spacing);
        let taps = (half * 2 + 1).next_multiple_of(8);
        let gain = spacing as f32 / FIR_STEP;
        let mut rows = vec![0.0; (PHASES + 1) * taps].into_boxed_slice();

        for (phase, row) in rows.chunks_exact_mut(taps).enumerate() {
            let frac = phase as f32 / PHASES as f32;

            // Tap j is applied to the input sample at offset j - half from the current position.
            for (j, coeff) in row.iter_mut().enumerate() {
                let x = FIR_CENTER as f32 + (frac + half as f32 - j as f32) * spacing as f32;
                *coeff = fir_at(x) * gain;
            }
        }

        PhaseBank { half, taps, rows }
    }

    /// Number of samples before the current position the filter reads.
    #[inline]
    pub fn half(&self) -> usize {
        self.half
    }

    /// Number of samples the filter reads, starting at `position - half`.
    #[inline]
    pub fn taps(&self) -> usize {
        self.taps
    }

    /// Returns the two coefficient rows surrounding `frac` and the blend factor between them.
    #[inline]
    pub fn rows(&self, frac: f32) -> (&[f32], &[f32], f32) {
        let x = frac * PHASES as f32;
        let phase = (x as usize).min(PHASES - 1);
        let start = phase * self.taps;

        (&self.rows[start..start + self.taps], &self.rows[start + self.taps..start + self.taps * 2], x - phase as f32)
    }
}

#[inline]
fn fir_at(x: f32) -> f32 {
    if x < 0.0 || x >= (FIR.len() - 1) as f32 {
        return 0.0;
    }

    let i = x as usize;
    let t = x - i as f32;
    FIR[i] + (FIR[i + 1] - FIR[i]) * t
}

/// Lazily built phase banks, one per quantized resampling ratio.
pub struct PhaseBanks {
    banks: Vec<Option<PhaseBank>>,
}

impl PhaseBanks {
    pub fn new() -> PhaseBanks {
        PhaseBanks { banks: (0..=MAX_SPACING).map(|_| None).collect() }
    }

    /// `advance` is the number of input samples consumed per output sample.
    pub fn get(&mut self, advance: f64) -> &PhaseBank {
        let ratio = if advance > 1.0 { 1.0 / advance } else { 1.0 };
        let spacing = ((FIR_STEP as f64 * ratio).round() as usize).clamp(MIN_SPACING, MAX_SPACING);

        self.banks[spacing].get_or_insert_with(|| PhaseBank::new(spacing))
    }
}
