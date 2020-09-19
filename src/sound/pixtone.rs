use std::collections::HashMap;

use num_traits::clamp;
use vec_mut_scan::VecMutScan;

use lazy_static::lazy_static;

use crate::sound::pixtone_sfx::PIXTONE_TABLE;
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
            sine.copy_from_slice(&*(&ref_data[0..0x100] as *const [u8] as *const [i8]));
            triangle.copy_from_slice(&*(&ref_data[0x100..0x200] as *const [u8] as *const [i8]));
            saw_up.copy_from_slice(&*(&ref_data[0x200..0x300] as *const [u8] as *const [i8]));
            saw_down.copy_from_slice(&*(&ref_data[0x300..0x400] as *const [u8] as *const [i8]));
            square.copy_from_slice(&*(&ref_data[0x400..0x500] as *const [u8] as *const [i8]));
            random.copy_from_slice(&*(&ref_data[0x500..0x600] as *const [u8] as *const [i8]));
        }

        // todo i can't get this shit right
/*
        let mut seed = 0i32;

        for i in 0..255 {
            seed = seed.wrapping_mul(214013).wrapping_add(2531011);
            sine[i] = (64.0 * (i as f64 * std::f64::consts::PI).sin()) as i8;
            triangle[i] = (if (0x40i32.wrapping_add(i as i32)) & 0x80 != 0 { 0x80i32.wrapping_sub(i as i32) } else { i as i32 }) as i8;
            saw_up[i] = (-0x40i32).wrapping_add(i as i32 / 2) as i8;
            saw_down[i] = (0x40i32.wrapping_sub(i as i32 / 2)) as i8;
            square[i] = (0x40i32.wrapping_sub(i as i32 & 0x80)) as i8;
            random[i] = (seed >> 16) as i8 / 2;
        }*/

        [sine, triangle, saw_up, saw_down, square, random]
    };
}

/*#[test]
fn test_waveforms() {
    let reference = include_bytes!("pixtone_ref.dat");

    for n in 1..(WAVEFORMS.len()) {
        for (i, &val) in WAVEFORMS[n].iter().enumerate() {
            assert_eq!((val as u8, i, n), (reference[n as usize * 256 + i], i, n));
        }
    }
}*/

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
        let (next_time, next_val) = {
            if i < self.time_c {
                (self.time_c, self.value_c)
            } else if i < self.time_b {
                (self.time_b, self.value_b)
            } else if i < self.time_a {
                (self.time_a, self.value_a)
            } else {
                (256, 0)
            }
        };

        let (prev_time, prev_val) = {
            if i >= self.time_a {
                (self.time_a, self.value_a)
            } else if i >= self.time_b {
                (self.time_b, self.value_b)
            } else if i >= self.time_c {
                (self.time_c, self.value_c)
            } else {
                (0, self.initial)
            }
        };

        if next_time <= prev_time {
            return prev_val;
        }

        (i - prev_time) * (next_val - prev_val) / (next_time - prev_time) + prev_val
    }
}

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
            carrier: Waveform {
                waveform_type: 0,
                pitch: 0.0,
                level: 0,
                offset: 0,
            },
            frequency: Waveform {
                waveform_type: 0,
                pitch: 0.0,
                level: 0,
                offset: 0,
            },
            amplitude: Waveform {
                waveform_type: 0,
                pitch: 0.0,
                level: 0,
                offset: 0,
            },
            envelope: Envelope {
                initial: 0,
                time_a: 0,
                value_a: 0,
                time_b: 0,
                value_b: 0,
                time_c: 0,
                value_c: 0,
            },
        }
    }
}

pub struct PixToneParameters {
    pub channels: [Channel; 4],
}

impl PixToneParameters {
    pub const fn empty() -> PixToneParameters {
        PixToneParameters {
            channels: [Channel::disabled(), Channel::disabled(), Channel::disabled(), Channel::disabled()]
        }
    }

    pub fn synth(&self) -> Vec<i16> {
        let length = self.channels.iter().map(|c| c.length as usize).max().unwrap_or(0);
        if length == 0 {
            return Vec::new();
        }

        let mut samples = vec![0i16; length + 100];

        for channel in self.channels.iter() {
            if !channel.enabled { continue; }

            fn s(p: f32, i: usize, length: u32) -> f32 {
                256.0 * p * i as f32 / length as f32
            }

            let mut phase = channel.carrier.offset as f32;
            let delta = 256.0 * channel.carrier.pitch as f32 / channel.length as f32;
            let carrier_wave = channel.carrier.get_waveform();
            let frequency_wave = channel.frequency.get_waveform();
            let amplitude_wave = channel.amplitude.get_waveform();
            let mut last_wave = 0;

            for (i, result) in samples.iter_mut().enumerate() {
                if i >= channel.length as usize {
                    if i == channel.length as usize {
                        last_wave = *result;
                    } else if i == (channel.length as usize + 100) {
                        break;
                    }

                    let fac = (i - channel.length as usize) as i16 / 2 + 1;

                    last_wave /= fac;
                    *result = last_wave;
                    continue;
                } else {
                    let carrier = carrier_wave[0xff & phase as usize] as i32 * channel.carrier.level;
                    let freq = frequency_wave[0xff & (channel.frequency.offset as f32 + s(channel.frequency.pitch, i, channel.length)) as usize] as i32 * channel.frequency.level;
                    let amp = amplitude_wave[0xff & (channel.amplitude.offset as f32 + s(channel.amplitude.pitch, i, channel.length)) as usize] as i32 * channel.amplitude.level;

                    *result = clamp((*result as i32) + (carrier * (amp + 4096) / 4096 * channel.envelope.evaluate(s(1.0, i, channel.length) as i32) / 4096) * 192, -32767, 32767) as i16;

                    phase += delta * (1.0 + (freq as f32 / (if freq < 0 { 8192.0 } else { 2048.0 })));
                }
            }
        }

        samples
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct PlaybackState(u8, f32, u32);

pub struct PixTonePlayback {
    pub samples: HashMap<u8, Vec<i16>>,
    pub playback_state: Vec<PlaybackState>,
}

impl PixTonePlayback {
    pub fn new() -> PixTonePlayback {
        PixTonePlayback {
            samples: HashMap::new(),
            playback_state: vec![],
        }
    }

    pub fn create_samples(&mut self) {
        for (i, params) in PIXTONE_TABLE.iter().enumerate() {
            self.samples.insert(i as u8, params.synth());
        }
    }

    pub fn play_sfx(&mut self, id: u8) {
        for state in self.playback_state.iter_mut() {
            if state.0 == id && state.2 == 0 {
                state.1 = 0.0;
                break;
            }
        }

        self.playback_state.push(PlaybackState(id, 0.0, 0));
    }

    pub fn play_concurrent(&mut self, id: u8, tag: u32) {
        self.playback_state.push(PlaybackState(id, 0.0, tag));
    }

    pub fn mix(&mut self, dst: &mut [u16], sample_rate: f32) {
        let mut scan = VecMutScan::new(&mut self.playback_state);
        let delta = 22050.0 / sample_rate;

        while let Some(item) = scan.next() {
            let mut state = *item;
            let mut remove = false;

            if let Some(sample) = self.samples.get(&state.0) {
                if sample.is_empty() {
                    item.remove();
                    continue;
                };

                for result in dst.iter_mut() {
                    if state.1 >= sample.len() as f32 {
                        remove = true;
                        break;
                    } else {
                        let pos = state.1 as usize;
                        let s1 = (sample[pos] as f32) / 32768.0;
                        let s2 = (sample[clamp(pos + 1, 0, sample.len() - 1)] as f32) / 32768.0;
                        let s3 = (sample[clamp(pos + 2, 0, sample.len() - 1)] as f32) / 32768.0;
                        let s4 = (sample[pos.saturating_sub(1)] as f32) / 32768.0;

                        let s = cubic_interp(s1, s2, s4, s3, state.1.fract()) * 32768.0;
                        let sam = (*result ^ 0x8000) as i16;
                        *result = sam.saturating_add(s as i16) as u16 ^ 0x8000;

                        state.1 += delta;
                    }
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
