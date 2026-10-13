// SPDX-License-Identifier: MIT
// Copyright (c) 2020 LunarLambda
// Copyright (c) 2020 doukutsu-rs contributors (see AUTHORS.md)
use std::mem::MaybeUninit;
use std::sync::Arc;

use crate::kernels::{self, Run};
use crate::organya::{Note, Song as Organya, Version};
use crate::polyphase::{PhaseBanks, GUARD};
use crate::stuff::*;
use crate::wav::*;
use crate::wave_bank::SoundBank;
use crate::InterpolationMode;

/// Frames processed per gain computation step, bounds the stack buffer used by `render_buffer`.
const BLOCK_LEN: usize = 64;

const ORG_OCTAVE_LENS: [usize; 8] = [256, 256, 128, 128, 64, 32, 16, 8];

/// Start of each octave in Organya sample data, every octave is surrounded by GUARD wrapped samples.
const ORG_OCTAVE_OFFSETS: [usize; 8] = {
    let mut offsets = [0; 8];
    let mut acc = 0;
    let mut i = 0;
    while i < 8 {
        offsets[i] = acc + GUARD;
        acc += ORG_OCTAVE_LENS[i] + GUARD * 2;
        i += 1;
    }
    offsets
};

pub struct OrgPlaybackEngine {
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
    phase_banks: PhaseBanks,
    mix_l: Vec<f32>,
    mix_r: Vec<f32>,
    /// Per track index of the first note at or after the last looked up position.
    note_cursors: [usize; 16],
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
            phase_banks: PhaseBanks::new(),
            mix_l: Vec::new(),
            mix_r: Vec::new(),
            note_cursors: [0; 16],
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

    pub fn start_song(&mut self, mut song: Organya, samples: &SoundBank) {
        for track in &mut song.tracks {
            track.notes.sort_by_key(|note| note.pos);
        }

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
        self.note_cursors.fill(0);
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

    /// Returns the note of `track` at the current play position, notes must be sorted by position.
    fn note_at_play_pos(&mut self, track: usize) -> Option<Note> {
        let notes = &self.song.tracks[track].notes;
        let cursor = &mut self.note_cursors[track];

        // Seeking backwards (loops, set_position) restarts the search.
        if *cursor > notes.len() || (*cursor > 0 && notes[*cursor - 1].pos >= self.play_pos) {
            *cursor = notes.partition_point(|note| note.pos < self.play_pos);
        }

        while *cursor < notes.len() && notes[*cursor].pos < self.play_pos {
            *cursor += 1;
        }

        notes.get(*cursor).filter(|note| note.pos == self.play_pos).copied()
    }

    fn update_play_state(&mut self) {
        for track in 0..8 {
            if let Some(note) = self.note_at_play_pos(track) {
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
                        self.track_buffers[j].playing = true;
                        self.track_buffers[j].looping = true;
                        self.track_buffers[j].nloops = -1;
                        self.track_buffers[j].release_len = 0;
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
                        self.track_buffers[j].playing = true;
                        self.track_buffers[j].looping = true;
                        self.track_buffers[j].nloops = -1;
                        self.track_buffers[j].release_len = 0;
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
                        self.track_buffers[j].playing = true;
                        self.track_buffers[j].looping = true;
                        self.track_buffers[j].nloops = -1;
                        self.track_buffers[j].release_len = 0;
                        self.keys[track] = note.key;
                    }

                    self.lengths[track] = note.len;
                }

                if self.keys[track] != 255 {
                    let octave = (self.keys[track] / 12) * 8;
                    let j = octave as usize + track + self.swaps[track];

                    if note.vol != 255 {
                        let vol = org_vol_to_vol(note.vol);
                        self.track_buffers[j].set_volume_glide(vol);
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
                    const NOTE_RELEASE: u16 = 255;
                    self.track_buffers[j].release_len = NOTE_RELEASE;
                    self.track_buffers[j].release_timer = NOTE_RELEASE;
                }
                self.keys[track] = 255;
            }

            self.lengths[track] = self.lengths[track].saturating_sub(1);
        }

        for i in 8..16 {
            let j = i + 120;

            // start a new note
            // note (hah) that drums are unaffected by length and pi values. This is the only case we have to handle.
            if let Some(note) = self.note_at_play_pos(i) {
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
        let frames = buf.len() / 2;
        let sample_rate = self.output_format.sample_rate as f64;

        self.mix_l.clear();
        self.mix_l.resize(frames, 0.0);
        self.mix_r.clear();
        self.mix_r.resize(frames, 0.0);

        let mut done = 0;
        let mut finished = false;

        while done < frames && !finished {
            if self.frames_this_tick == 0 {
                self.update_play_state()
            }

            let n = self.frames_per_tick.saturating_sub(self.frames_this_tick).max(1).min(frames - done);
            let out_l = &mut self.mix_l[done..done + n];
            let out_r = &mut self.mix_r[done..done + n];

            for buf in &mut self.track_buffers {
                if buf.playing {
                    render_buffer(buf, self.interpolation, sample_rate, &mut self.phase_banks, out_l, out_r);
                }
            }

            done += n;
            self.frames_this_tick += n;

            if self.frames_this_tick == self.frames_per_tick {
                self.play_pos += 1;

                if self.play_pos == self.song.time.loop_range.end {
                    self.play_pos = self.song.time.loop_range.start;

                    if self.loops == 0 {
                        finished = true;
                    } else {
                        self.loops -= 1;
                    }
                }

                self.frames_this_tick = 0;
            }
        }

        for ((frame, &l), &r) in buf.chunks_exact_mut(2).zip(&self.mix_l[..done]).zip(&self.mix_r[..done]) {
            frame[0] = mix_into_u16(frame[0], l);
            frame[1] = mix_into_u16(frame[1], r);
        }

        if finished {
            done * 2
        } else {
            buf.len()
        }
    }
}

#[inline(always)]
fn mix_into_u16(dst: u16, sample: f32) -> u16 {
    let mixed = ((dst ^ 0x8000) as i16 as f32 + sample * 32768.0).clamp(-32768.0, 32767.0);
    mixed as i16 as u16 ^ 0x8000
}

/// Renders frames from `buf` into `out_l`/`out_r` until they're filled or the buffer stops playing.
fn render_buffer(
    buf: &mut RenderBuffer,
    mode: InterpolationMode,
    sample_rate: f64,
    phase_banks: &mut PhaseBanks,
    out_l: &mut [f32],
    out_r: &mut [f32],
) {
    if buf.len == 0 {
        buf.playing = false;
        return;
    }

    let advance = buf.frequency as f64 / sample_rate;
    let bank = if mode == InterpolationMode::Polyphase { Some(phase_banks.get(advance)) } else { None };
    let (left, right) = buf.data.split_at(buf.right_offset);
    let (left, right) = if buf.right_offset == 0 { (right, None) } else { (left, Some(right)) };

    let mut gains = [0.0f32; BLOCK_LEN];
    let mut done = 0;

    while done < out_l.len() && buf.playing {
        // Split runs where the position wraps or the release ends, so the kernel never has to check either.
        let to_end = ((buf.len as f64 - buf.position) / advance).ceil();
        let mut n = if to_end >= 1.0 { to_end.min(BLOCK_LEN as f64) as usize } else { 1 };
        if buf.release_len > 0 {
            n = n.min(buf.release_timer as usize + 1);
        }
        n = n.min(out_l.len() - done);

        for gain in &mut gains[..n] {
            buf.vol_cent = buf.vol_cent + (buf.vol_cent_target - buf.vol_cent) * 0.1;

            // add a bit of release to the note for declicking
            let release = if buf.release_len > 0 { buf.release_timer as f32 / buf.release_len as f32 } else { 1.0 };
            *gain = buf.vol_cent * release;

            if buf.release_len > 0 && buf.release_timer == 0 {
                buf.release_len = 0;
                buf.looping = false;
                buf.playing = false;
            }
            if buf.release_timer > 0 {
                buf.release_timer -= 1;
            }
        }

        let run = Run { left, right, base: buf.base_pos, position: buf.position, advance };
        kernels::mix(
            mode,
            bank,
            &run,
            &gains[..n],
            buf.pan_cent,
            &mut out_l[done..done + n],
            &mut out_r[done..done + n],
        );

        done += n;
        buf.position += advance * n as f64;

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
    pub sample_rate: u32,
    pub data: Arc<[f32]>,
    pub right_offset: usize,
    pub playing: bool,
    pub looping: bool,
    pub release_len: u16,
    pub release_timer: u16,
    pub base_pos: usize,
    pub len: usize,
    // -1 = infinite
    pub nloops: i32,
    vol_cent_target: f32,
    vol_cent: f32,
    pan_cent: (f32, f32),
}

impl RenderBuffer {
    pub fn new(sample: WavSample) -> RenderBuffer {
        let channels = sample.format.channels.max(1) as usize;
        let bytes_per_sample = if sample.format.bit_depth == 16 { 2 } else { 1 };
        let bytes_per_frame = channels * bytes_per_sample;
        let len = sample.data.len() / bytes_per_frame;
        let stride = len + GUARD * 2;
        let planes = if channels == 2 { 2 } else { 1 };

        let mut data = vec![0.0; stride * planes];
        for (plane, out) in data.chunks_exact_mut(stride).enumerate() {
            let frames = sample.data.chunks_exact(bytes_per_frame);

            for (frame, out) in frames.zip(&mut out[GUARD..GUARD + len]) {
                let offset = plane * bytes_per_sample;
                *out = if bytes_per_sample == 2 {
                    i16::from_le_bytes([frame[offset], frame[offset + 1]]) as f32 / 32768.0
                } else {
                    (frame[offset] as f32 - 128.0) / 128.0
                };
            }
        }

        RenderBuffer {
            position: 0.0,
            frequency: sample.format.sample_rate,
            volume: 0,
            pan: 0,
            sample_rate: sample.format.sample_rate,
            data: data.into(),
            right_offset: if planes == 2 { stride } else { 0 },
            len,
            playing: false,
            looping: false,
            release_len: 0,
            release_timer: 0,
            base_pos: GUARD,
            nloops: -1,
            vol_cent_target: 0.0,
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
            sample_rate: 22050,
            data: Arc::new([]),
            right_offset: 0,
            len: 0,
            playing: false,
            looping: false,
            release_len: 0,
            release_timer: 0,
            base_pos: 0,
            nloops: -1,
            vol_cent_target: 0.0,
            vol_cent: 0.0,
            pan_cent: (0.0, 0.0),
        }
    }

    /// `wave` is a 256 sample unsigned 8-bit waveform.
    pub fn new_organya(format: WavFormat, wave: Vec<u8>) -> RenderBuffer {
        let total = ORG_OCTAVE_OFFSETS[7] + ORG_OCTAVE_LENS[7] + GUARD;
        let mut data = Vec::with_capacity(total);

        for size in ORG_OCTAVE_LENS {
            let step = 256 / size;
            let octave: Vec<f32> = (0..size).map(|i| (wave[i * step] as f32 - 128.0) / 128.0).collect();

            // Octaves are looped, so the padding continues the waveform in both directions.
            let lead = GUARD.next_multiple_of(size) - GUARD;
            data.extend((0..GUARD).map(|i| octave[(lead + i) % size]));
            data.extend_from_slice(&octave);
            data.extend((0..GUARD).map(|i| octave[i % size]));
        }

        RenderBuffer {
            frequency: format.sample_rate,
            sample_rate: format.sample_rate,
            data: data.into(),
            len: ORG_OCTAVE_LENS[0],
            base_pos: ORG_OCTAVE_OFFSETS[0],
            ..RenderBuffer::empty()
        }
    }

    #[inline]
    pub fn organya_select_octave(&mut self, octave: usize, pipi: bool) {
        self.base_pos = ORG_OCTAVE_OFFSETS[octave];
        self.len = ORG_OCTAVE_LENS[octave];
        self.position %= self.len as f64;
        if pipi && !self.playing {
            self.nloops = ((octave + 1) * 4) as i32;
        }
    }

    #[inline]
    pub fn set_frequency(&mut self, frequency: u32) {
        //assert!(frequency >= 100 && frequency <= 100000);
        //dbg!(frequency);
        let rate_mod = self.sample_rate as f32 / 22050.0;
        self.frequency = (frequency as f32 * rate_mod) as u32;
    }

    #[inline]
    pub fn set_volume(&mut self, volume: i32) {
        // assert!(volume >= -10000 && volume <= 0);

        self.volume = volume;
        self.vol_cent = centibel_to_scale(volume);
        self.vol_cent_target = self.vol_cent;
    }

    #[inline]
    pub fn set_volume_glide(&mut self, volume: i32) {
        self.volume = volume;
        self.vol_cent_target = centibel_to_scale(volume);
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
