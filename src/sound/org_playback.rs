use std::cmp::min;
use std::hint::unreachable_unchecked;
use std::mem::MaybeUninit;
use std::sync::Arc;

use crate::sound::fir::FIR;
use crate::sound::fir::FIR_STEP;
use crate::sound::organya::{Song as Organya, Version};
use crate::sound::stuff::*;
use crate::sound::wav::*;
use crate::sound::wave_bank::SoundBank;
use crate::sound::InterpolationMode;

#[derive(Clone)]
pub struct FIRData {
    cache: Vec<f32>,
    pos: usize,
}

impl FIRData {
    pub fn new() -> Self {
        FIRData { cache: Vec::new(), pos: 0 }
    }

    pub fn ensure_initialized(&mut self) {
        if self.cache.is_empty() {
            self.cache.resize(FIR.len() * 8, 0.0);
        }
    }
}

pub(crate) struct OrgPlaybackEngine {
    song: Organya,
    lengths: [u8; 8],
    swaps: [usize; 8],
    keys: [u8; 8],
    /// Octave 0 Track 0 Swap 0
    /// Octave 0 Track 1 Swap 0
    /// ...
    /// Octave 1 Track 0 Swap 0
    /// ...
    /// Octave 0 Track 0 Swap 1
    /// octave * 8 + track + swap
    /// 128..136: Drum Tracks
    track_buffers: [RenderBuffer; 136],
    output_format: WavFormat,
    play_pos: i32,
    frames_this_tick: usize,
    frames_per_tick: usize,
    pub loops: usize,
    pub interpolation: InterpolationMode,
}

#[derive(Clone)]
pub struct SavedOrganyaPlaybackState {
    song: Organya,
    play_pos: i32,
}

impl OrgPlaybackEngine {
    pub fn new() -> Self {
        let mut buffers: [MaybeUninit<RenderBuffer>; 136] = unsafe { MaybeUninit::uninit().assume_init() };

        buffers.fill_with(|| MaybeUninit::new(RenderBuffer::empty()));

        let song = Organya::empty();
        let frames_per_tick = (44100 / 1000) * song.time.wait as usize;

        OrgPlaybackEngine {
            song,
            lengths: [0; 8],
            swaps: [0; 8],
            keys: [255; 8],
            track_buffers: unsafe { std::mem::transmute(buffers) },
            play_pos: 0,
            output_format: WavFormat { channels: 2, sample_rate: 44100, bit_depth: 16 },
            frames_this_tick: 0,
            frames_per_tick,
            loops: 1,
            interpolation: InterpolationMode::Linear,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: usize) {
        self.frames_this_tick =
            (self.frames_this_tick as f32 * (self.output_format.sample_rate as f32 / sample_rate as f32)) as usize;
        self.output_format.sample_rate = sample_rate as u32;
        self.frames_per_tick = (sample_rate / 1000) * self.song.time.wait as usize;

        if self.frames_this_tick >= self.frames_per_tick {
            self.frames_this_tick = 0;
        }
    }

    pub fn get_state(&self) -> SavedOrganyaPlaybackState {
        SavedOrganyaPlaybackState { song: self.song.clone(), play_pos: self.play_pos }
    }

    pub fn set_state(&mut self, state: SavedOrganyaPlaybackState, samples: &SoundBank) {
        self.start_song(state.song, samples);
        self.play_pos = state.play_pos;
    }

    pub fn start_song(&mut self, song: Organya, samples: &SoundBank) {
        for i in 0..8 {
            let sound_index = song.tracks[i].inst.inst as usize;
            let sound = samples.get_wave(sound_index).iter().map(|&x| x ^ 128).collect();

            let format = WavFormat { channels: 1, sample_rate: 22050, bit_depth: 8 };

            let rbuf = RenderBuffer::new_organya(format, sound);

            for j in 0..8 {
                for &k in &[0, 64] {
                    self.track_buffers[i + (j * 8) + k] = rbuf.clone();
                }
            }
        }

        // Initialize drums
        for (idx, (track, buf)) in song.tracks[8..].iter().zip(self.track_buffers[128..].iter_mut()).enumerate() {
            if song.version == Version::Extended {
                // Check for OOB track count, instruments outside of the sample range will be set to the last valid sample
                let index = if track.inst.inst as usize >= samples.samples.len() {
                    samples.samples.len() - 1
                } else {
                    track.inst.inst as usize
                };
                *buf = RenderBuffer::new(samples.samples[index].clone());
            } else {
                let index = if idx >= samples.samples.len() { samples.samples.len() - 1 } else { idx };
                *buf = RenderBuffer::new(samples.samples[index].clone());
            }
        }

        self.song = song;
        self.play_pos = 0;
        self.frames_per_tick = (self.output_format.sample_rate as usize / 1000) * self.song.time.wait as usize;
        self.frames_this_tick = 0;
        self.lengths.fill(0);
        self.swaps.fill(0);
        self.keys.fill(255);
    }

    pub fn set_position(&mut self, position: i32) {
        self.play_pos = position;
    }

    pub fn rewind(&mut self) {
        self.set_position(0);
    }

    #[allow(unused)]
    pub fn get_total_samples(&self) -> u32 {
        let ticks_intro = self.song.time.loop_range.start;
        let ticks_loop = self.song.time.loop_range.end - self.song.time.loop_range.start;
        let ticks_total = ticks_intro + ticks_loop + (ticks_loop * self.loops as i32);

        self.frames_per_tick as u32 * ticks_total as u32
    }

    fn update_play_state(&mut self) {
        for track in 0..8 {
            if let Some(note) = self.song.tracks[track].notes.iter().find(|x| x.pos == self.play_pos) {
                // New note
                if note.key != 255 {
                    if self.keys[track] == 255 {
                        // New
                        let octave = (note.key / 12) * 8;
                        let j = octave as usize + track + self.swaps[track];
                        for k in 0..16 {
                            let swap = if k >= 8 { 64 } else { 0 };
                            let key = note.key % 12;
                            let p_oct = k % 8;

                            let freq = org_key_to_freq(key + p_oct * 12, self.song.tracks[track].inst.freq as i16);

                            let l = p_oct as usize * 8 + track + swap;
                            self.track_buffers[l].set_frequency(freq as u32);
                            self.track_buffers[l]
                                .organya_select_octave(p_oct as usize, self.song.tracks[track].inst.pipi != 0);
                        }
                        self.track_buffers[j].looping = true;
                        self.track_buffers[j].playing = true;
                        // last playing key
                        self.keys[track] = note.key;
                    } else if self.keys[track] == note.key {
                        // Same
                        //assert!(self.lengths[track] == 0);
                        let octave = (self.keys[track] / 12) * 8;
                        let j = octave as usize + track + self.swaps[track];
                        if self.song.tracks[track].inst.pipi == 0 {
                            self.track_buffers[j].looping = false;
                        }
                        self.swaps[track] += 64;
                        self.swaps[track] %= 128;
                        let j = octave as usize + track + self.swaps[track];
                        self.track_buffers[j]
                            .organya_select_octave(note.key as usize / 12, self.song.tracks[track].inst.pipi != 0);
                        self.track_buffers[j].looping = true;
                        self.track_buffers[j].playing = true;
                    } else {
                        // change
                        let octave = (self.keys[track] / 12) * 8;
                        let j = octave as usize + track + self.swaps[track];
                        if self.song.tracks[track].inst.pipi == 0 {
                            self.track_buffers[j].looping = false;
                        }
                        self.swaps[track] += 64;
                        self.swaps[track] %= 128;
                        let octave = (note.key / 12) * 8;
                        let j = octave as usize + track + self.swaps[track];
                        for k in 0..16 {
                            let swap = if k >= 8 { 64 } else { 0 };
                            let key = note.key % 12;
                            let p_oct = k % 8;

                            let freq = org_key_to_freq(key + p_oct * 12, self.song.tracks[track].inst.freq as i16);
                            let l = p_oct as usize * 8 + track + swap;
                            self.track_buffers[l].set_frequency(freq as u32);
                            self.track_buffers[l]
                                .organya_select_octave(p_oct as usize, self.song.tracks[track].inst.pipi != 0);
                        }
                        self.track_buffers[j].looping = true;
                        self.track_buffers[j].playing = true;
                        self.keys[track] = note.key;
                    }

                    self.lengths[track] = note.len;
                }

                if self.keys[track] != 255 {
                    let octave = (self.keys[track] / 12) * 8;
                    let j = octave as usize + track + self.swaps[track];

                    if note.vol != 255 {
                        let vol = org_vol_to_vol(note.vol);
                        self.track_buffers[j].set_volume(vol);
                    }

                    if note.pan != 255 {
                        let pan = org_pan_to_pan(note.pan);
                        self.track_buffers[j].set_pan(pan);
                    }
                }
            }

            if self.lengths[track] == 0 && self.keys[track] != 255 {
                let octave = (self.keys[track] / 12) * 8;
                let j = octave as usize + track + self.swaps[track];
                if self.song.tracks[track].inst.pipi == 0 {
                    self.track_buffers[j].looping = false;
                }
                self.keys[track] = 255;
            }

            self.lengths[track] = self.lengths[track].saturating_sub(1);
        }

        for i in 8..16 {
            let j = i + 120;

            let notes = &self.song.tracks[i].notes;

            // start a new note
            // note (hah) that drums are unaffected by length and pi values. This is the only case we have to handle.
            if let Some(note) = notes.iter().find(|x| x.pos == self.play_pos) {
                // FIXME: Add constants for dummy values
                if note.key != 255 {
                    let freq = org_key_to_drum_freq(note.key);
                    self.track_buffers[j].set_frequency(freq as u32);
                    self.track_buffers[j].set_position(0);
                    self.track_buffers[j].playing = true;
                }

                if note.vol != 255 {
                    let vol = org_vol_to_vol(note.vol);
                    self.track_buffers[j].set_volume(vol);
                }

                if note.pan != 255 {
                    let pan = org_pan_to_pan(note.pan);
                    self.track_buffers[j].set_pan(pan);
                }
            }
        }
    }

    pub fn render_to(&mut self, buf: &mut [u16]) -> usize {
        let mut i = 0;
        let mut iter = buf.iter_mut();

        // optimized for debug mode
        // bound / arithmetic checks give a HUGE performance hit in this code
        let fl = FIR.len() as f32;

        // raw pointer access is much faster than get_unchecked
        let fir_ptr = FIR.as_ptr();
        let freq = self.output_format.sample_rate as f64;

        if self.interpolation == InterpolationMode::Polyphase {
            for buf in &mut self.track_buffers {
                buf.fir.ensure_initialized();
            }
        }

        while let (Some(frame_l), Some(frame_r)) = (iter.next(), iter.next()) {
            if self.frames_this_tick == 0 {
                self.update_play_state()
            }

            for buf in &mut self.track_buffers {
                if buf.playing {
                    let is_16bit = buf.sample.format.bit_depth == 16;
                    let is_stereo = buf.sample.format.channels == 2;

                    let get_sample = match (is_16bit, is_stereo) {
                        (true, true) => |buf: &RenderBuffer, pos: usize| -> (f32, f32) {
                            let sl = i16::from_le_bytes([buf.sample.data[pos << 2], buf.sample.data[pos << 2 + 1]])
                                as f32
                                / 32768.0;
                            let sr = i16::from_le_bytes([buf.sample.data[pos << 2 + 2], buf.sample.data[pos << 2 + 3]])
                                as f32
                                / 32768.0;
                            (sl, sr)
                        },
                        (false, true) => |buf: &RenderBuffer, pos: usize| -> (f32, f32) {
                            let sl = (buf.sample.data[pos << 1] as f32 - 128.0) / 128.0;
                            let sr = (buf.sample.data[(pos << 1) + 1] as f32 - 128.0) / 128.0;
                            (sl, sr)
                        },
                        (true, false) => |buf: &RenderBuffer, pos: usize| -> (f32, f32) {
                            let s = i16::from_le_bytes([buf.sample.data[pos << 1], buf.sample.data[pos << 1 + 1]])
                                as f32
                                / 32768.0;
                            (s, s)
                        },
                        (false, false) => |buf: &RenderBuffer, pos: usize| -> (f32, f32) {
                            let s = (buf.sample.data[pos] as f32 - 128.0) / 128.0;
                            (s, s)
                        },
                    };

                    // index into sound samples
                    let advance = buf.frequency as f64 / freq;

                    let vol = buf.vol_cent;
                    let (pan_l, pan_r) = buf.pan_cent;

                    if self.interpolation == InterpolationMode::Polyphase {
                        let fir_step = (FIR_STEP * advance as f32).floor();
                        let fir_step = if fir_step == 0.0 { FIR_STEP } else { fir_step };
                        let fir_gain = fir_step / FIR_STEP;
                        let cache_ptr = buf.fir.cache.as_mut_ptr();
                        let sample_data_ptr = buf.sample.data.as_ptr();

                        let pos = buf.position as usize + buf.base_pos;
                        let cl = buf.fir.cache.len() / 2;
                        let i = buf.fir.pos % cl;

                        let (sl1, sr1, sl2, sr2) = match (is_16bit, is_stereo) {
                            (true, true) => unsafe {
                                let ps = pos << 2;
                                let sl1 = (*sample_data_ptr.add(ps) as u16 | (*sample_data_ptr.add(ps + 1) as u16) << 8)
                                    as f32
                                    / 32768.0;
                                let sr1 = (*sample_data_ptr.add(ps + 2) as u16
                                    | (*sample_data_ptr.add(ps + 3) as u16) << 8)
                                    as f32
                                    / 32768.0;
                                let ps = min(pos + 1, buf.base_pos + buf.len - 1) << 2;
                                let sl2 = (*sample_data_ptr.add(ps) as u16 | (*sample_data_ptr.add(ps + 1) as u16) << 8)
                                    as f32
                                    / 32768.0;
                                let sr2 = (*sample_data_ptr.add(ps + 2) as u16
                                    | (*sample_data_ptr.add(ps + 3) as u16) << 8)
                                    as f32
                                    / 32768.0;
                                (sl1, sr1, sl2, sr2)
                            },
                            (false, true) => unsafe {
                                let ps = pos << 1;
                                let sl1 = (*sample_data_ptr.add(ps) as f32 - 128.0) / 128.0;
                                let sr1 = (*sample_data_ptr.add(ps + 1) as f32 - 128.0) / 128.0;
                                let ps = min(pos + 1, buf.base_pos + buf.len - 1) << 1;
                                let sl2 = (*sample_data_ptr.add(ps) as f32 - 128.0) / 128.0;
                                let sr2 = (*sample_data_ptr.add(ps + 1) as f32 - 128.0) / 128.0;
                                (sl1, sr1, sl2, sr2)
                            },
                            (true, false) => unsafe {
                                let ps = pos << 1;
                                let s1 = (*sample_data_ptr.add(ps) as u16 | (*sample_data_ptr.add(ps + 1) as u16) << 8)
                                    as f32
                                    / 32768.0;
                                let ps = min(pos + 1, buf.base_pos + buf.len - 1) << 1;
                                let s2 = (*sample_data_ptr.add(ps) as u16 | (*sample_data_ptr.add(ps + 1) as u16) << 8)
                                    as f32
                                    / 32768.0;
                                (s1, s1, s2, s2)
                            },
                            (false, false) => unsafe {
                                let s1 = (*sample_data_ptr.add(pos) as f32 - 128.0) / 128.0;
                                let pos = min(pos + 1, buf.base_pos + buf.len - 1);
                                let s2 = (*sample_data_ptr.add(pos) as f32 - 128.0) / 128.0;
                                (s1, s1, s2, s2)
                            },
                        };

                        let r1 = buf.position.fract() as f32;

                        buf.position += advance;
                        if buf.position as usize >= buf.len {
                            if buf.looping && buf.nloops != 1 {
                                buf.position %= buf.len as f64;
                                if buf.nloops != -1 {
                                    buf.nloops -= 1;
                                }
                            } else {
                                buf.position = 0.0;
                                buf.playing = false;
                            }
                        }

                        let cl = cl as isize;
                        let mut insamp_idx = (buf.fir.pos as isize).wrapping_rem(cl);

                        if is_stereo {
                            let sl = sl1 + (sl2 - sl1) * r1;
                            let sr = sr1 + (sr2 - sr1) * r1;

                            buf.fir.cache[i * 2] = sl;
                            buf.fir.cache[i * 2 + 1] = sr;

                            let mut acc_l = 0.0;
                            let mut acc_r = 0.0;
                            let mut step = 0.0;

                            while step < fl {
                                unsafe {
                                    let idx = (insamp_idx as usize) << 1;
                                    acc_l += (*fir_ptr.add(step as usize)) * (*cache_ptr.add(idx));
                                    acc_r += (*fir_ptr.add(step as usize)) * (*cache_ptr.add(idx + 1));
                                    insamp_idx =
                                        if insamp_idx == 0 { cl.wrapping_sub(1) } else { insamp_idx.wrapping_sub(1) };
                                    step += fir_step;
                                }
                            }

                            acc_l *= fir_gain;
                            acc_r *= fir_gain;

                            let sl = acc_l * pan_l * vol * 32768.0;
                            let sr = acc_r * pan_r * vol * 32768.0;

                            let xl = (*frame_l ^ 0x8000) as i16;
                            let xr = (*frame_r ^ 0x8000) as i16;

                            *frame_l = xl.saturating_add(sl as i16) as u16 ^ 0x8000;
                            *frame_r = xr.saturating_add(sr as i16) as u16 ^ 0x8000;
                        } else {
                            let sl = sl1 + (sl2 - sl1) * r1;
                            buf.fir.cache[i * 2] = sl;

                            let mut acc = 0.0;
                            let mut step = 0.0;

                            while step < fl {
                                unsafe {
                                    let idx = (insamp_idx as usize) << 1;
                                    acc += (*fir_ptr.add(step as usize)) * (*cache_ptr.add(idx));
                                    insamp_idx =
                                        if insamp_idx == 0 { cl.wrapping_sub(1) } else { insamp_idx.wrapping_sub(1) };
                                    step += fir_step;
                                }
                            }

                            acc *= fir_gain;

                            let sl = acc * pan_l * vol * 32768.0;
                            let sr = acc * pan_r * vol * 32768.0;

                            let xl = (*frame_l ^ 0x8000) as i16;
                            let xr = (*frame_r ^ 0x8000) as i16;

                            *frame_l = xl.saturating_add(sl as i16) as u16 ^ 0x8000;
                            *frame_r = xr.saturating_add(sr as i16) as u16 ^ 0x8000;
                        }
                        buf.fir.pos += 1;
                    } else {
                        let pos = buf.position as usize + buf.base_pos;

                        let (sl, sr) = match self.interpolation {
                            InterpolationMode::Nearest => get_sample(buf, pos),
                            InterpolationMode::Linear => {
                                let (sl1, sr1) = get_sample(buf, pos);
                                let (sl2, sr2) = get_sample(buf, min(pos + 1, buf.base_pos + buf.len - 1));
                                let r1 = buf.position.fract() as f32;

                                let sl = sl1 + (sl2 - sl1) * r1;
                                let sr = sr1 + (sr2 - sr1) * r1;

                                (sl, sr)
                            }
                            InterpolationMode::Cosine => {
                                use std::f32::consts::PI;

                                let (sl1, sr1) = get_sample(buf, pos);
                                let (sl2, sr2) = get_sample(buf, min(pos + 1, buf.base_pos + buf.len - 1));

                                let r1 = buf.position.fract() as f32;
                                let r2 = (1.0 - f32::cos(r1 * PI)) / 2.0;

                                let sl = sl1 * (1.0 - r2) + sl2 * r2;
                                let sr = sr1 * (1.0 - r2) + sr2 * r2;

                                (sl, sr)
                            }
                            InterpolationMode::Cubic => {
                                let (sl1, sr1) = get_sample(buf, pos);
                                let (sl2, sr2) = get_sample(buf, min(pos + 1, buf.base_pos + buf.len - 1));
                                let (sl3, sr3) = get_sample(buf, min(pos + 2, buf.base_pos + buf.len - 1));
                                let (sl4, sr4) = get_sample(buf, pos.saturating_sub(1));

                                let r1 = buf.position.fract() as f32;

                                let sl = cubic_interp(sl1, sl2, sl4, sl3, r1);
                                let sr = cubic_interp(sr1, sr2, sr4, sr3, r1);

                                (sl, sr)
                            }
                            InterpolationMode::Polyphase => unsafe { unreachable_unchecked() },
                        };

                        let sl = sl * pan_l * vol * 32768.0;
                        let sr = sr * pan_r * vol * 32768.0;

                        buf.position += advance;

                        if buf.position as usize >= buf.len {
                            if buf.looping && buf.nloops != 1 {
                                buf.position %= buf.len as f64;
                                if buf.nloops != -1 {
                                    buf.nloops -= 1;
                                }
                            } else {
                                buf.position = 0.0;
                                buf.playing = false;
                                break;
                            }
                        }

                        let xl = (*frame_l ^ 0x8000) as i16;
                        let xr = (*frame_r ^ 0x8000) as i16;

                        *frame_l = xl.saturating_add(sl as i16) as u16 ^ 0x8000;
                        *frame_r = xr.saturating_add(sr as i16) as u16 ^ 0x8000;
                    }
                }
            }

            self.frames_this_tick += 1;

            if self.frames_this_tick == self.frames_per_tick {
                self.play_pos += 1;

                if self.play_pos == self.song.time.loop_range.end {
                    self.play_pos = self.song.time.loop_range.start;

                    if self.loops == 0 {
                        return i + 2;
                    }

                    self.loops -= 1;
                }

                self.frames_this_tick = 0;
            }

            i += 2;
        }

        buf.len()
    }
}

#[inline(always)]
pub fn centibel_to_scale(a: i32) -> f32 {
    f32::powf(10.0, a as f32 / 2000.0)
}

#[derive(Clone)]
pub struct RenderBuffer {
    pub position: f64,
    pub frequency: u32,
    pub volume: i32,
    pub pan: i32,
    pub sample: WavSample,
    pub playing: bool,
    pub looping: bool,
    pub base_pos: usize,
    pub len: usize,
    // -1 = infinite
    pub nloops: i32,
    pub fir: FIRData,
    vol_cent: f32,
    pan_cent: (f32, f32),
}

impl RenderBuffer {
    pub fn new(sample: WavSample) -> RenderBuffer {
        let bytes_per_sample = sample.format.channels as usize * if sample.format.bit_depth == 16 { 2 } else { 1 };
        RenderBuffer {
            position: 0.0,
            frequency: sample.format.sample_rate,
            volume: 0,
            pan: 0,
            len: sample.data.len() / bytes_per_sample,
            sample,
            playing: false,
            looping: false,
            base_pos: 0,
            nloops: -1,
            fir: FIRData::new(),
            vol_cent: 0.0,
            pan_cent: (0.0, 0.0),
        }
    }

    pub fn empty() -> RenderBuffer {
        RenderBuffer {
            position: 0.0,
            frequency: 22050,
            volume: 0,
            pan: 0,
            len: 0,
            sample: WavSample {
                format: WavFormat { channels: 2, sample_rate: 22050, bit_depth: 16 },
                data: Arc::new([]),
            },
            playing: false,
            looping: false,
            base_pos: 0,
            nloops: -1,
            fir: FIRData::new(),
            vol_cent: 0.0,
            pan_cent: (0.0, 0.0),
        }
    }

    pub fn new_organya(format: WavFormat, wave: Vec<u8>) -> RenderBuffer {
        let mut sample_data = Vec::with_capacity(wave.len());

        for size in &[256_usize, 256, 128, 128, 64, 32, 16, 8] {
            let step = 256 / size;
            let mut acc = 0;

            for _ in 0..*size {
                sample_data.push(wave[acc]);
                acc += step;

                if acc >= 256 {
                    acc = 0;
                }
            }
        }

        RenderBuffer::new(WavSample { format, data: sample_data.into() })
    }

    #[inline]
    pub fn organya_select_octave(&mut self, octave: usize, pipi: bool) {
        const OFFS: &[usize] = &[0x000, 0x100, 0x200, 0x280, 0x300, 0x340, 0x360, 0x370];
        const LENS: &[usize] = &[256_usize, 256, 128, 128, 64, 32, 16, 8];
        self.base_pos = OFFS[octave];
        self.len = LENS[octave];
        self.position %= self.len as f64;
        if pipi && !self.playing {
            self.nloops = ((octave + 1) * 4) as i32;
        }
    }

    #[inline]
    pub fn set_frequency(&mut self, frequency: u32) {
        //assert!(frequency >= 100 && frequency <= 100000);
        //dbg!(frequency);
        let rate_mod = self.sample.format.sample_rate as f32 / 22050.0;
        self.frequency = (frequency as f32 * rate_mod) as u32;
    }

    #[inline]
    pub fn set_volume(&mut self, volume: i32) {
        // assert!(volume >= -10000 && volume <= 0);

        self.volume = volume;
        self.vol_cent = centibel_to_scale(volume);
    }

    #[inline]
    pub fn set_pan(&mut self, pan: i32) {
        // assert!(pan >= -10000 && pan <= 10000);

        self.pan = pan;
        self.pan_cent = match self.pan.signum() {
            0 => (1.0, 1.0),
            1 => (centibel_to_scale(-self.pan), 1.0),
            -1 => (1.0, centibel_to_scale(self.pan)),
            _ => unsafe { std::hint::unreachable_unchecked() },
        };
    }

    #[inline]
    #[allow(unused)]
    pub fn set_position(&mut self, position: u32) {
        // assert!(position < self.sample.data.len() as u32 / self.sample.format.bit_depth as u32);

        self.position = position as f64;
    }
}
