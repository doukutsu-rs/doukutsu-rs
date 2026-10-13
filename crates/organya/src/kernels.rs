use std::f32::consts::PI;

use wide::f32x8;

use crate::polyphase::PhaseBank;
use crate::stuff::cubic_interp;
use crate::InterpolationMode;

const LANES: usize = 8;

/// A run of frames rendered from planar sample data, `right` is None for mono samples.
/// `base` must leave at least `polyphase::GUARD` readable samples on both sides of the played range.
pub struct Run<'a> {
    pub left: &'a [f32],
    pub right: Option<&'a [f32]>,
    pub base: usize,
    pub position: f64,
    pub advance: f64,
    /// Length the position loops at, `f64::INFINITY` for runs that end before reaching their end.
    pub wrap: f64,
}

impl Run<'_> {
    #[inline(always)]
    fn step(&self, mut pos: f64) -> f64 {
        pos += self.advance;
        while pos >= self.wrap {
            pos -= self.wrap;
        }
        pos
    }
}

/// Interpolates `gains.len()` frames from `run`, adding `sample * gain * pan` into the outputs.
pub fn mix(
    mode: InterpolationMode,
    bank: Option<&PhaseBank>,
    run: &Run,
    gains: &[f32],
    pan: (f32, f32),
    out_l: &mut [f32],
    out_r: &mut [f32],
) {
    match mode {
        InterpolationMode::Nearest => mix_with::<Nearest>(run, gains, pan, out_l, out_r),
        InterpolationMode::Linear => mix_with::<Linear>(run, gains, pan, out_l, out_r),
        InterpolationMode::Cosine => mix_with::<Cosine>(run, gains, pan, out_l, out_r),
        InterpolationMode::Cubic => mix_with::<Cubic>(run, gains, pan, out_l, out_r),
        InterpolationMode::Polyphase => {
            let bank = bank.expect("polyphase interpolation requires a phase bank");
            mix_polyphase(bank, run, gains, pan, out_l, out_r)
        }
    }
}

trait Interpolator {
    /// Interpolates 8 frames, `idx` holds the index of the sample at or before each frame's position.
    fn interpolate(data: &[f32], idx: &[usize; LANES], frac: f32x8) -> f32x8;
}

#[inline(always)]
fn gather(data: &[f32], idx: &[usize; LANES], offset: usize) -> f32x8 {
    f32x8::new(std::array::from_fn(|lane| data[idx[lane] + offset]))
}

struct Nearest;

impl Interpolator for Nearest {
    #[inline(always)]
    fn interpolate(data: &[f32], idx: &[usize; LANES], _frac: f32x8) -> f32x8 {
        gather(data, idx, 0)
    }
}

struct Linear;

impl Interpolator for Linear {
    #[inline(always)]
    fn interpolate(data: &[f32], idx: &[usize; LANES], frac: f32x8) -> f32x8 {
        let s1 = gather(data, idx, 0);
        let s2 = gather(data, idx, 1);
        (s2 - s1).mul_add(frac, s1)
    }
}

struct Cosine;

impl Interpolator for Cosine {
    #[inline(always)]
    fn interpolate(data: &[f32], idx: &[usize; LANES], frac: f32x8) -> f32x8 {
        let s1 = gather(data, idx, 0);
        let s2 = gather(data, idx, 1);
        let r = (f32x8::ONE - (frac * f32x8::splat(PI)).cos()) * f32x8::HALF;
        (s2 - s1).mul_add(r, s1)
    }
}

struct Cubic;

impl Interpolator for Cubic {
    #[inline(always)]
    fn interpolate(data: &[f32], idx: &[usize; LANES], frac: f32x8) -> f32x8 {
        let idx = idx.map(|i| i - 1);
        let sp = gather(data, &idx, 0);
        let s1 = gather(data, &idx, 1);
        let s2 = gather(data, &idx, 2);
        let sn = gather(data, &idx, 3);
        cubic_interp(s1, s2, sp, sn, frac)
    }
}

fn mix_with<I: Interpolator>(
    run: &Run,
    gains: &[f32],
    (pan_l, pan_r): (f32, f32),
    out_l: &mut [f32],
    out_r: &mut [f32],
) {
    let pan_l = f32x8::splat(pan_l);
    let pan_r = f32x8::splat(pan_r);

    let mut pos = run.position;

    // The last block may extend past the run, its extra lanes read guard samples and get zero gain.
    for (block, gains) in gains.chunks(LANES).enumerate() {
        let start = block * LANES;
        let mut idx = [0; LANES];
        let mut frac = [0.0; LANES];
        for lane in 0..LANES {
            let ipos = pos as usize;
            idx[lane] = run.base + ipos;
            frac[lane] = (pos - ipos as f64) as f32;
            pos = run.step(pos);
        }

        let gain = f32x8::new(pad_lanes(gains));
        let frac = f32x8::new(frac);

        let sl = I::interpolate(run.left, &idx, frac) * gain;
        let sr = match run.right {
            Some(right) => I::interpolate(right, &idx, frac) * gain,
            None => sl,
        };

        let end = start + gains.len();
        accumulate(&mut out_l[start..end], sl * pan_l);
        accumulate(&mut out_r[start..end], sr * pan_r);
    }
}

// Full blocks take the fixed size paths, a variable length copy compiles to a memcpy call.
#[inline(always)]
fn pad_lanes(values: &[f32]) -> [f32; LANES] {
    match values.try_into() {
        Ok(values) => values,
        Err(_) => {
            let mut padded = [0.0; LANES];
            padded[..values.len()].copy_from_slice(values);
            padded
        }
    }
}

#[inline(always)]
fn accumulate(out: &mut [f32], v: f32x8) {
    match <&mut [f32; LANES]>::try_from(&mut *out) {
        Ok(out) => *out = (f32x8::new(*out) + v).to_array(),
        Err(_) => {
            for (out, v) in out.iter_mut().zip(v.to_array()) {
                *out += v;
            }
        }
    }
}

fn mix_polyphase(
    bank: &PhaseBank,
    run: &Run,
    gains: &[f32],
    (pan_l, pan_r): (f32, f32),
    out_l: &mut [f32],
    out_r: &mut [f32],
) {
    let half = bank.half();
    let taps = bank.taps();
    let mut pos = run.position;

    for (k, &gain) in gains.iter().enumerate() {
        let ipos = pos as usize;
        let frac = (pos - ipos as f64) as f32;
        pos = run.step(pos);
        let start = run.base + ipos - half;
        let (row0, row1, t) = bank.rows(frac);

        let left = &run.left[start..start + taps];
        let (d0, d1) = dot2(row0, row1, left);
        let sl = (d1 - d0).mul_add(t, d0) * gain;

        let sr = match run.right {
            Some(right) => {
                let (d0, d1) = dot2(row0, row1, &right[start..start + taps]);
                (d1 - d0).mul_add(t, d0) * gain
            }
            None => sl,
        };

        out_l[k] += sl * pan_l;
        out_r[k] += sr * pan_r;
    }
}

/// Dot products of `window` with both coefficient rows, all three are a multiple of 8 long.
#[inline(always)]
fn dot2(row0: &[f32], row1: &[f32], window: &[f32]) -> (f32, f32) {
    let (row0, _) = row0.as_chunks::<LANES>();
    let (row1, _) = row1.as_chunks::<LANES>();
    let (window, _) = window.as_chunks::<LANES>();

    let mut acc0 = f32x8::ZERO;
    let mut acc1 = f32x8::ZERO;
    for ((&c0, &c1), &x) in row0.iter().zip(row1).zip(window) {
        let x = f32x8::new(x);
        acc0 = f32x8::new(c0).mul_add(x, acc0);
        acc1 = f32x8::new(c1).mul_add(x, acc1);
    }

    (acc0.reduce_add(), acc1.reduce_add())
}
