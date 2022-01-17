use std::collections::HashMap;

use lazy_static::lazy_static;
use vec_mut_scan::VecMutScan;

use crate::sound::pixtone_sfx::DEFAULT_PIXTONE_TABLE;
use crate::sound::stuff::cubic_interp;

lazy_static! {
    static ref WAVEFORMS: [[i8; 0x100]; 6] = {
        let mut sine = [0i8; 0x100];
        let mut triangle = [0i8; 0x100];
        let mut saw_up = [0i8; 0x100];
        let mut saw_down = [0i8; 0x100];
        let mut square = [0i8; 0x100];
        let mut random = [0i8; 0x100];

        let ref_data = include_bytes!("pixtone_ref.dat");

        unsafe {
            sine.copy_from_slice(std::mem::transmute(&ref_data[0..0x100]));
            triangle.copy_from_slice(std::mem::transmute(&ref_data[0x100..0x200]));
            saw_up.copy_from_slice(std::mem::transmute(&ref_data[0x200..0x300]));
            saw_down.copy_from_slice(std::mem::transmute(&ref_data[0x300..0x400]));
            square.copy_from_slice(std::mem::transmute(&ref_data[0x400..0x500]));
            random.copy_from_slice(std::mem::transmute(&ref_data[0x500..0x600]));
        }

        [sine, triangle, saw_up, saw_down, square, random]
    };
}

#[derive(Copy, Clone)]
pub struct Waveform {
    pub waveform_type: u8,
    pub pitch: f32,
    pub level: i32,
    pub offset: i32,
}

impl Waveform {
    pub fn get_waveform(&self) -> &[i8; 0x100] {
        &WAVEFORMS[self.waveform_type as usize % WAVEFORMS.len()]
    }
}

#[derive(Copy, Clone)]
pub struct Envelope {
    pub initial: i32,
    pub time_a: i32,
    pub value_a: i32,
    pub time_b: i32,
    pub value_b: i32,
    pub time_c: i32,
    pub value_c: i32,
}

impl Envelope {
    pub fn evaluate(&self, i: i32) -> i32 {
        let (mut next_time, mut next_val) = (256, 0);
        let (mut prev_time, mut prev_val) = (0, self.initial);

        if i < self.time_c {
            next_time = self.time_c;
            next_val = self.value_c;
        }
        if i < self.time_b {
            next_time = self.time_b;
            next_val = self.value_b;
        }
        if i < self.time_a {
            next_time = self.time_a;
            next_val = self.value_a;
        }

        if i >= self.time_a {
            prev_time = self.time_a;
            prev_val = self.value_a;
        }
        if i >= self.time_b {
            prev_time = self.time_b;
            prev_val = self.value_b;
        }
        if i >= self.time_c {
            prev_time = self.time_c;
            prev_val = self.value_c;
        }

        if next_time <= prev_time {
            return prev_val;
        }

        (i - prev_time) * (next_val - prev_val) / (next_time - prev_time) + prev_val
    }
}

#[derive(Copy, Clone)]
pub struct Channel {
    pub enabled: bool,
    pub length: u32,
    pub carrier: Waveform,
    pub frequency: Waveform,
    pub amplitude: Waveform,
    pub envelope: Envelope,
}

impl Channel {
    pub const fn disabled() -> Channel {
        Channel {
            enabled: false,
            length: 0,
            carrier: Waveform { waveform_type: 0, pitch: 0.0, level: 0, offset: 0 },
            frequency: Waveform { waveform_type: 0, pitch: 0.0, level: 0, offset: 0 },
            amplitude: Waveform { waveform_type: 0, pitch: 0.0, level: 0, offset: 0 },
            envelope: Envelope { initial: 0, time_a: 0, value_a: 0, time_b: 0, value_b: 0, time_c: 0, value_c: 0 },
        }
    }
}

#[derive(Copy, Clone)]
pub struct PixToneParameters {
    pub channels: [Channel; 4],
}

impl PixToneParameters {
    pub const fn empty() -> PixToneParameters {
        PixToneParameters {
            channels: [Channel::disabled(), Channel::disabled(), Channel::disabled(), Channel::disabled()],
        }
    }

    pub fn synth(&self) -> Vec<i16> {
        let length = self.channels.iter().map(|c| c.length as usize).max().unwrap_or(0);
        if length == 0 {
            return Vec::new();
        }

        let mut samples = vec![0i16; length];

        for channel in &self.channels {
            if !channel.enabled {
                continue;
            }

            let mut phase = channel.carrier.offset as f32;
            let delta = 256.0 * channel.carrier.pitch as f32 / channel.length as f32;
            let carrier_wave = channel.carrier.get_waveform();
            let frequency_wave = channel.frequency.get_waveform();
            let amplitude_wave = channel.amplitude.get_waveform();

            for (i, result) in samples.iter_mut().enumerate() {
                if i == channel.length as usize {
                    break;
                }

                let s = |p: f32| -> f32 { 256.0 * p * i as f32 / channel.length as f32 };

                let carrier = carrier_wave[0xff & phase as usize] as i32 * channel.carrier.level;
                let freq = frequency_wave
                    [0xff & (channel.frequency.offset as f32 + s(channel.frequency.pitch)) as usize]
                    as i32
                    * channel.frequency.level;
                let amp = amplitude_wave[0xff & (channel.amplitude.offset as f32 + s(channel.amplitude.pitch)) as usize]
                    as i32
                    * channel.amplitude.level;

                *result = ((*result as i32)
                    + (carrier * (amp + 4096) / 4096 * channel.envelope.evaluate(s(1.0) as i32) / 4096) * 256)
                    .clamp(-32767, 32767) as i16;

                phase += delta * (1.0 + (freq as f32 / (if freq < 0 { 8192.0 } else { 2048.0 })));
            }
        }

        samples
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct PlaybackState {
    id: u8,
    pos: f32,
    tag: u32,
    looping: bool,
}

pub struct PixTonePlayback {
    pub samples: HashMap<u8, Vec<i16>>,
    pub playback_state: Vec<PlaybackState>,
    pub table: [PixToneParameters; 256],
}

#[allow(unused)]
impl PixTonePlayback {
    pub fn new() -> PixTonePlayback {
        let mut table = [PixToneParameters::empty(); 256];

        for (i, params) in DEFAULT_PIXTONE_TABLE.iter().enumerate() {
            table[i] = *params;
        }

        PixTonePlayback { samples: HashMap::new(), playback_state: vec![], table }
    }

    pub fn create_samples(&mut self) {
        for (i, params) in self.table.iter().enumerate() {
            self.samples.insert(i as u8, params.synth());
        }
    }

    pub fn set_sample_parameters(&mut self, id: u8, params: PixToneParameters) {
        self.table[id as usize] = params;
        self.samples.insert(id, params.synth());
    }

    pub fn set_sample_data(&mut self, id: u8, data: Vec<i16>) {
        self.samples.insert(id, data);
    }

    pub fn play_sfx(&mut self, id: u8) {
        for state in &mut self.playback_state {
            if state.id == id && state.tag == 0 {
                state.pos = 0.0;
                state.looping = false;
                return;
            }
        }

        self.playback_state.push(PlaybackState { id, pos: 0.0, tag: 0, looping: false });
    }

    pub fn loop_sfx(&mut self, id: u8) {
        for state in &mut self.playback_state {
            if state.id == id && state.tag == 0 {
                state.looping = true;
                return;
            }
        }

        self.playback_state.push(PlaybackState { id, pos: 0.0, tag: 0, looping: true });
    }

    pub fn stop_sfx(&mut self, id: u8) {
        if let Some(pos) = self.playback_state.iter().position(|s| s.id == id && s.tag == 0) {
            self.playback_state.remove(pos);
        }
    }

    pub fn play_concurrent(&mut self, id: u8, tag: u32) {
        self.playback_state.push(PlaybackState { id, pos: 0.0, tag, looping: false });
    }

    pub fn mix(&mut self, dst: &mut [u16], sample_rate: f32) {
        let mut scan = VecMutScan::new(&mut self.playback_state);
        let delta = 22050.0 / sample_rate;

        while let Some(item) = scan.next() {
            let mut state = *item;
            let mut remove = false;

            if let Some(sample) = self.samples.get(&state.id) {
                if sample.is_empty() {
                    item.remove();
                    continue;
                };

                for result in dst.iter_mut() {
                    if state.pos >= sample.len() as f32 {
                        if state.looping {
                            state.pos = 0.0;
                        } else {
                            remove = true;
                            break;
                        }
                    }

                    let pos = state.pos as usize;
                    let s1 = (sample[pos] as f32) / 32768.0;
                    let s2 = (sample[(pos + 1).clamp(0, sample.len() - 1)] as f32) / 32768.0;
                    let s3 = (sample[(pos + 2).clamp(0, sample.len() - 1)] as f32) / 32768.0;
                    let s4 = (sample[pos.saturating_sub(1)] as f32) / 32768.0;

                    let s = cubic_interp(s1, s2, s4, s3, state.pos.fract()) * 32768.0;
                    // let s = sample[pos] as f32;
                    let sam = (*result ^ 0x8000) as i16;
                    *result = sam.saturating_add(s as i16) as u16 ^ 0x8000;

                    state.pos += delta;
                }

                if remove {
                    item.remove();
                } else {
                    item.replace(state);
                }
            }
        }
    }
}
