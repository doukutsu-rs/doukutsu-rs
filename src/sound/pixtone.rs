use lazy_static::lazy_static;

lazy_static! {
    static ref WAVEFORMS: [[i8; 0x100]; 6] = {
        let mut seed = 0;
        let mut sine = [0i8; 0x100];
        let mut triangle = [0i8; 0x100];
        let mut saw_up = [0i8; 0x100];
        let mut saw_down = [0i8; 0x100];
        let mut square = [0i8; 0x100];
        let mut random = [0i8; 0x100];

        for i in 0..255 {
            seed = (seed * 214013) + 2531011;
            sine[i] = (64.0 * (i as f32 * std::f32::consts::PI).sin()) as i8;
            triangle[i] = (if (0x40 + i as isize) & 0x80 != 0 { 0x80 - i as isize } else { i as isize }) as i8;
            saw_up[i] = (-0x40 + i as isize / 2) as i8;
            saw_down[i] = (0x40 - i as isize / 2) as i8;
            square[i] = (0x40 - (i as isize & 0x80)) as i8;
            random[i] = ((seed >> 16) / 2) as i8;
        }

        [sine, triangle, saw_up, saw_down, square, random]
    };
}

pub struct PixToneData {

}

pub struct PixTone {}

impl PixTone {

}

fn dupa() -> [[i8; 0x100]; 6] {
    let mut seed = 0;
    let mut sine = [0i8; 0x100];
    let mut triangle = [0i8; 0x100];
    let mut saw_up = [0i8; 0x100];
    let mut saw_down = [0i8; 0x100];
    let mut square = [0i8; 0x100];
    let mut random = [0i8; 0x100];

    for i in 0..255 {
        seed = (seed * 214013) + 2531011;
        sine[i] = (64.0 * (i as f32 * std::f32::consts::PI).sin()) as i8;
        triangle[i] = (if (0x40 + i as isize) & 0x80 != 0 { 0x80 - i as isize } else { i as isize }) as i8;
        saw_up[i] = (-0x40 + i as isize / 2) as i8;
        saw_down[i] = (0x40 - i as isize / 2) as i8;
        square[i] = (0x40 - (i as isize & 0x80)) as i8;
        random[i] = ((seed >> 16) / 2) as i8;
    }

    [sine, triangle, saw_up, saw_down, square, random]
}
