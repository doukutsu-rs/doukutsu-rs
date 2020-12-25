pub const FRQ_TBL: [i16; 12] = [
    262, 277, 294, 311, 330, 349, 370, 392, 415, 440, 466, 494
];

pub const PAN_TBL: [i16; 13] = [
    0, 43, 86, 129, 172, 215, 256, 297, 340, 383, 426, 469, 512
];

pub const OCT_TBL: [i16; 8] = [
    32, 64, 64, 128, 128, 128, 128, 128
];

pub fn org_key_to_freq(key: u8, a: i16) -> i32 {
    let (oct, pitch) = org_key_to_oct_pitch(key);

    let freq = FRQ_TBL[pitch as usize] as f32;
    let oct = OCT_TBL[oct as usize] as f32;

    (freq * oct) as i32 + (a as i32 - 1000)
}

pub fn org_key_to_drum_freq(key: u8) -> i32 {
    key as i32 * 800 + 100
}

pub fn org_pan_to_pan(pan: u8) -> i32 {
    (PAN_TBL[pan as usize] as i32 - 256) * 10
}

pub fn org_vol_to_vol(vol: u8) -> i32 {
    (vol as i32 - 255) * 8
}

pub fn org_key_to_oct_pitch(key: u8) -> (u8, u8) {
    (key / 12, key % 12)
}

// s1: sample 1
// s2: sample 2
// sp: previous sample (before s1)
// sn: next sample (after s2)
// mu: position to interpolate for
pub fn cubic_interp(s1: f32, s2: f32, sp: f32, sn: f32, mu: f32) -> f32 {
    let mu2 = mu * mu;
    let a0 = sn - s2 - sp + s1;
    let a1 = sp - s1 - a0;
    let a2 = s2 - sp;
    let a3 = s1;

    a0 * mu * mu2 + a1 * mu2 + a2 * mu + a3
}
