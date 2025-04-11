use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use lazy_static::lazy_static;
use num_traits::{abs, Num};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeTupleStruct;

use crate::bitfield;
use crate::graphics::texture_set::G_MAG;

/// Multiply cave story degrees (0-255, which corresponds to 0°-360°) with this constant to get
/// respective value in radians.
pub const CDEG_RAD: f64 = std::f64::consts::PI / 128.0;

lazy_static! {
    pub static ref VERSION_BANNER: String = {
        let version = option_env!("DRS_BUILD_VERSION_OVERRIDE").unwrap_or(env!("CARGO_PKG_VERSION"));
        format!("doukutsu-rs {}", version)
    };
}

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct Flag(u32);
    impl Debug;

    /// Set if left wall was hit. (corresponds to flag & 0x01)
    pub hit_left_wall, set_hit_left_wall: 0;
    /// Set if top wall was hit. (corresponds to flag & 0x02)
    pub hit_top_wall, set_hit_top_wall: 1;
    /// Set if right wall was hit. (corresponds to flag & 0x04)
    pub hit_right_wall, set_hit_right_wall: 2;
    /// Set if bottom wall was hit. (corresponds to flag & 0x08)
    pub hit_bottom_wall, set_hit_bottom_wall: 3;
    /// Set if entity stays on right slope. (corresponds to flag & 0x10)
    pub hit_right_slope, set_hit_right_slope: 4;
    /// Set if entity stays on left slope. (corresponds to flag & 0x20)
    pub hit_left_slope, set_hit_left_slope: 5;
    /// Used only in bullet code, set if a bullet hits upper right slope (corresponds to flag & 0x40)
    pub hit_upper_right_slope, set_hit_upper_right_slope: 6;
    /// Used only in bullet code, set if a bullet hits upper left slope (corresponds to flag & 0x80)
    pub hit_upper_left_slope, set_hit_upper_left_slope: 7;
    /// Set if entity is in water. (corresponds to flag & 0x100)
    pub in_water, set_in_water: 8;
    pub weapon_hit_block, set_weapon_hit_block: 9; // 0x200
    pub hit_by_spike, set_hit_by_spike: 10; // 0x400
    pub bloody_droplets, set_bloody_droplets: 11; // 0x800
    pub force_left, set_force_left: 12; // 0x1000
    pub force_up, set_force_up: 13; // 0x2000
    pub force_right, set_force_right: 14; // 0x4000
    pub force_down, set_force_down: 15; // 0x8000
    pub hit_left_higher_half, set_hit_left_higher_half: 16; // 0x10000
    pub hit_left_lower_half, set_hit_left_lower_half: 17; // 0x20000
    pub hit_right_lower_half, set_hit_right_lower_half: 18; // 0x40000
    pub hit_right_higher_half, set_hit_right_higher_half: 19; // 0x80000
}

impl Flag {
    pub fn any_flag(&self) -> bool {
        self.0 != 0
    }

    pub fn hit_anything(&self) -> bool {
        (self.0 & 0x2ff) != 0
    }
}

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct Equipment(u16);
    impl Debug;

    pub has_booster_0_8, set_booster_0_8: 0; // 0x01 / 0001
    pub has_map, set_map: 1; // 0x02 / 0002
    pub has_arms_barrier, set_arms_barrier: 2; // 0x04 / 0004
    pub has_turbocharge, set_turbocharge: 3; // 0x08 / 0008
    pub has_air_tank, set_air_tank: 4; // 0x10 / 0016
    pub has_booster_2_0, set_booster_2_0: 5; // 0x20 / 0032
    pub has_mimiga_mask, set_mimiga_mask: 6; // 0x40 / 0064
    pub has_whimsical_star, set_whimsical_star: 7; // 0x080 / 0128
    pub has_nikumaru, set_nikumaru: 8; // 0x100 / 0256
    // for custom equips
    pub unused_1, set_unused_1: 9; // 0x200 / 0512
    pub unused_2, set_unused_2: 10; // 0x400 / 1024
    pub unused_3, set_unused_3: 11; // 0x800 / 2048
    pub unused_4, set_unused_4: 12; // 0x1000 / 4096
    pub unused_5, set_unused_5: 13; // 0x2000 / 8192
    // bit 14 and 15 aren't accessible via TSC without abusing overflows (won't work in strict mode)
    pub unused_6, set_unused_6: 14; // 0x4000 / @384
    pub unused_7, set_unused_7: 15; // 0x8000 / P768
}

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct Condition(u16);
    impl Debug;

    pub interacted, set_interacted: 0; // 0x01
    pub hidden, set_hidden: 1; // 0x02
    pub fallen, set_fallen: 2; // 0x04
    pub explode_die, set_explode_die: 3; // 0x08
    pub damage_boss, set_damage_boss: 4; // 0x10
    pub increase_acceleration, set_increase_acceleration: 5; // 0x20
    pub cond_x40, set_cond_x40: 6; // 0x40
    pub alive, set_alive: 7; // 0x80

    // engine specific flags
    pub drs_novanish, set_drs_novanish: 14;
    pub drs_boss, set_drs_boss: 15;
}

bitfield! {
    #[derive(Clone, Copy, Serialize, Deserialize)]
    #[repr(C)]
    pub struct ControlFlags(u16);
    impl Debug;

    pub tick_world, set_tick_world: 0; // 0x01
    pub control_enabled, set_control_enabled: 1; // 0x02
    pub interactions_disabled, set_interactions_disabled: 2; // 0x04
    pub credits_running, set_credits_running: 3; // 0x08
    // cs+ switch specific, according to peri:
    // Flag 0x10 prevents the OK button from restarting the item description event (resets when the cursor is moved)
    // (it does not prevent the cancel button from exiting the inventory, however)
    pub ok_button_disabled, set_ok_button_disabled: 4; // 0x10
    // engine specific flags
    pub friendly_fire, set_friendly_fire: 14;
}

bitfield! {
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct BulletFlag(u8);
    impl Debug;
    pub flag_x01, set_flag_x01: 0; // 0x01, nowhere in code?
    pub flag_x02, set_flag_x02: 1; // 0x02, nowhere in code?
    /// Corresponds to flag & 0x04. If set, bullet will pass through blocks.
    pub no_collision_checks, set_no_collision_checks: 2;
    /// Corresponds to flag & 0x08. IF set, bullet will bounce off walls.
    pub bounce_from_walls, set_bounce_from_walls: 3;
    /// Corresponds to flag & 0x10. IF set, bullet will not produce projectile dissipation effect when it hits a NPC or boss.
    pub no_proj_dissipation, set_no_proj_dissipation: 4;
    /// Corresponds to flag & 0x20. If set, performs checks in block collision check procedure. Kills the bullet if flag 0x40 isn't set.
    pub check_block_hit, set_check_block_hit: 5;
    /// Corresponds to flag & 0x40. If set, bullet will destroy snack blocks on hit.
    pub can_destroy_snack, set_can_destroy_snack: 6;
    pub flag_x80, set_flag_x80: 7; // 0x80, nowhere in code?
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FadeDirection {
    Left = 0,
    Up,
    Right,
    Down,
    Center,
}

impl FadeDirection {
    pub const fn from_int(val: usize) -> Option<FadeDirection> {
        match val {
            0 => Some(FadeDirection::Left),
            1 => Some(FadeDirection::Up),
            2 => Some(FadeDirection::Right),
            3 => Some(FadeDirection::Down),
            4 => Some(FadeDirection::Center),
            _ => None,
        }
    }

    pub const fn opposite(&self) -> FadeDirection {
        match self {
            FadeDirection::Left => FadeDirection::Right,
            FadeDirection::Up => FadeDirection::Down,
            FadeDirection::Right => FadeDirection::Left,
            FadeDirection::Down => FadeDirection::Up,
            FadeDirection::Center => FadeDirection::Center,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum FadeState {
    Visible,
    FadeIn(i8, FadeDirection),
    Hidden,
    FadeOut(i8, FadeDirection),
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    Left = 0,
    Up,
    Right,
    Bottom,
    FacingPlayer,
}

pub const FILE_TYPES: [&str; 3] = [".png", ".bmp", ".pbm"];

impl Direction {
    pub const fn from_int(val: usize) -> Option<Direction> {
        match val {
            0 => Some(Direction::Left),
            1 => Some(Direction::Up),
            2 => Some(Direction::Right),
            3 => Some(Direction::Bottom),
            _ => None,
        }
    }

    pub const fn from_int_facing(val: usize) -> Option<Direction> {
        match val {
            0 => Some(Direction::Left),
            1 => Some(Direction::Up),
            2 => Some(Direction::Right),
            3 => Some(Direction::Bottom),
            4 => Some(Direction::FacingPlayer),
            _ => None,
        }
    }

    pub const fn opposite(&self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Up => Direction::Bottom,
            Direction::Right => Direction::Left,
            Direction::Bottom => Direction::Up,
            Direction::FacingPlayer => Direction::FacingPlayer,
        }
    }

    pub const fn vector_x(&self) -> i32 {
        match self {
            Direction::Left => -1,
            Direction::Up => 0,
            Direction::Right => 1,
            Direction::Bottom => 0,
            Direction::FacingPlayer => 0,
        }
    }

    pub const fn vector_y(&self) -> i32 {
        match self {
            Direction::Left => 0,
            Direction::Up => -1,
            Direction::Right => 0,
            Direction::Bottom => 1,
            Direction::FacingPlayer => 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Rect<T: Num + PartialOrd + Copy = isize> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl<T: Num + PartialOrd + Copy> Rect<T> {
    #[inline(always)]
    pub fn new(left: T, top: T, right: T, bottom: T) -> Rect<T> {
        Rect { left, top, right, bottom }
    }

    #[inline(always)]
    pub fn new_size(x: T, y: T, width: T, height: T) -> Rect<T> {
        Rect { left: x, top: y, right: x.add(width), bottom: y.add(height) }
    }

    pub fn has_point(&self, x: T, y: T) -> bool {
        self.left.ge(&x) && self.right.le(&x) && self.top.ge(&y) && self.bottom.le(&y)
    }

    pub fn width(&self) -> T {
        if self.left.gt(&self.right) {
            self.left.sub(self.right)
        } else {
            self.right.sub(self.left)
        }
    }

    pub fn height(&self) -> T {
        if self.top.gt(&self.bottom) {
            self.top.sub(self.bottom)
        } else {
            self.bottom.sub(self.top)
        }
    }
}

impl<T: Num + PartialOrd + Copy + Serialize> Serialize for Rect<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut state = serializer.serialize_tuple_struct("Rect", 4)?;
        state.serialize_field(&self.left)?;
        state.serialize_field(&self.top)?;
        state.serialize_field(&self.right)?;
        state.serialize_field(&self.bottom)?;
        state.end()
    }
}

impl<T: Num + PartialOrd + Copy + Serialize> Default for Rect<T> {
    fn default() -> Self {
        Rect {
            left: num_traits::zero(),
            top: num_traits::zero(),
            right: num_traits::zero(),
            bottom: num_traits::zero(),
        }
    }
}

macro_rules! rect_deserialize {
    ($num_type: ident) => {
        impl<'de> Deserialize<'de> for Rect<$num_type> {
            fn deserialize<D>(deserializer: D) -> Result<Rect<$num_type>, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct RectVisitor;

                impl<'de> Visitor<'de> for RectVisitor {
                    type Value = Rect<$num_type>;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("Expected Rect structure.")
                    }

                    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
                    where
                        V: SeqAccess<'de>,
                    {
                        let invalid_length = || de::Error::invalid_length(0, &self);

                        let left = seq.next_element()?.ok_or_else(invalid_length)?;
                        let top = seq.next_element()?.ok_or_else(invalid_length)?;
                        let right = seq.next_element()?.ok_or_else(invalid_length)?;
                        let bottom = seq.next_element()?.ok_or_else(invalid_length)?;

                        Ok(Rect { left, top, right, bottom })
                    }
                }

                deserializer.deserialize_tuple_struct("Rect", 4, RectVisitor)
            }
        }
    };
}

rect_deserialize!(u8);
rect_deserialize!(u16);
rect_deserialize!(i32);
rect_deserialize!(isize);
rect_deserialize!(usize);

#[inline(always)]
pub fn fix9_scale(val: i32) -> f32 {
    unsafe {
        let mag = G_MAG as f32;
        (val as f32 * mag / 512.0).floor() / mag
    }
}

#[inline(always)]
fn lerp_f64(v1: f64, v2: f64, t: f64) -> f64 {
    v1 * (1.0 - t) + v2 * t
}

pub fn interpolate_fix9_scale(old_val: i32, val: i32, frame_delta: f64) -> f32 {
    if abs(old_val - val) > 0x1800 {
        return val as f32 / 512.0;
    }

    unsafe {
        let interpolated = lerp_f64(old_val as f64, val as f64, frame_delta) as f32;
        let mag = G_MAG as f32;
        (interpolated * mag / 512.0).floor() / mag
    }
}

pub fn get_timestamp() -> u64 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH).unwrap().as_secs() as u64
}

/// A RGBA color in the `sRGB` color space represented as `f32`'s in the range `[0.0-1.0]`
///
/// For convenience, [`WHITE`](constant.WHITE.html) and [`BLACK`](constant.BLACK.html) are provided.
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Color {
    /// Red component
    pub r: f32,
    /// Green component
    pub g: f32,
    /// Blue component
    pub b: f32,
    /// Alpha component
    pub a: f32,
}

impl Color {
    /// Create a new `Color` from four `f32`'s in the range `[0.0-1.0]`
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    /// Create a new `Color` from four `u8`'s in the range `[0-255]`
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color::from((r, g, b, a))
    }

    /// Create a new `Color` from three u8's in the range `[0-255]`,
    /// with the alpha component fixed to 255 (opaque)
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color::from((r, g, b))
    }

    /// Return a tuple of four `u8`'s in the range `[0-255]` with the `Color`'s
    /// components.
    pub fn to_rgba(self) -> (u8, u8, u8, u8) {
        self.into()
    }

    /// Return a tuple of three `u8`'s in the range `[0-255]` with the `Color`'s
    /// components.
    pub fn to_rgb(self) -> (u8, u8, u8) {
        self.into()
    }

    /// Convert a packed `u32` containing `0xRRGGBBAA` into a `Color`
    pub fn from_rgba_u32(c: u32) -> Color {
        let c = c.to_be_bytes();

        Color::from((c[0], c[1], c[2], c[3]))
    }

    /// Convert a packed `u32` containing `0x00RRGGBB` into a `Color`.
    /// This lets you do things like `Color::from_rgb_u32(0xCD09AA)` easily if you want.
    pub fn from_rgb_u32(c: u32) -> Color {
        let c = c.to_be_bytes();

        Color::from((c[1], c[2], c[3]))
    }

    /// Convert a `Color` into a packed `u32`, containing `0xRRGGBBAA` as bytes.
    pub fn to_rgba_u32(self) -> u32 {
        let (r, g, b, a): (u8, u8, u8, u8) = self.into();

        u32::from_be_bytes([r, g, b, a])
    }

    /// Convert a `Color` into a packed `u32`, containing `0x00RRGGBB` as bytes.
    pub fn to_rgb_u32(self) -> u32 {
        let (r, g, b, _a): (u8, u8, u8, u8) = self.into();

        u32::from_be_bytes([0, r, g, b])
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    /// Convert a `(R, G, B, A)` tuple of `u8`'s in the range `[0-255]` into a `Color`
    fn from(val: (u8, u8, u8, u8)) -> Self {
        let (r, g, b, a) = val;
        let rf = (f32::from(r)) / 255.0;
        let gf = (f32::from(g)) / 255.0;
        let bf = (f32::from(b)) / 255.0;
        let af = (f32::from(a)) / 255.0;
        Color::new(rf, gf, bf, af)
    }
}

impl From<(u8, u8, u8)> for Color {
    /// Convert a `(R, G, B)` tuple of `u8`'s in the range `[0-255]` into a `Color`,
    /// with a value of 255 for the alpha element (i.e., no transparency.)
    fn from(val: (u8, u8, u8)) -> Self {
        let (r, g, b) = val;
        Color::from((r, g, b, 255))
    }
}

impl From<[f32; 4]> for Color {
    /// Turns an `[R, G, B, A] array of `f32`'s into a `Color` with no format changes.
    /// All inputs should be in the range `[0.0-1.0]`.
    fn from(val: [f32; 4]) -> Self {
        Color::new(val[0], val[1], val[2], val[3])
    }
}

impl From<(f32, f32, f32)> for Color {
    /// Convert a `(R, G, B)` tuple of `f32`'s in the range `[0.0-1.0]` into a `Color`,
    /// with a value of 1.0 to for the alpha element (ie, no transparency.)
    fn from(val: (f32, f32, f32)) -> Self {
        let (r, g, b) = val;
        Color::new(r, g, b, 1.0)
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    /// Convert a `(R, G, B, A)` tuple of `f32`'s in the range `[0.0-1.0]` into a `Color`
    fn from(val: (f32, f32, f32, f32)) -> Self {
        let (r, g, b, a) = val;
        Color::new(r, g, b, a)
    }
}

impl From<Color> for (u8, u8, u8, u8) {
    /// Convert a `Color` into a `(R, G, B, A)` tuple of `u8`'s in the range of `[0-255]`.
    fn from(color: Color) -> Self {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;
        (r, g, b, a)
    }
}

impl From<Color> for (u8, u8, u8) {
    /// Convert a `Color` into a `(R, G, B)` tuple of `u8`'s in the range of `[0-255]`,
    /// ignoring the alpha term.
    fn from(color: Color) -> Self {
        let (r, g, b, _) = color.into();
        (r, g, b)
    }
}

impl From<Color> for [f32; 4] {
    /// Convert a `Color` into an `[R, G, B, A]` array of `f32`'s in the range of `[0.0-1.0]`.
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

pub trait SliceExt {
    type Item;

    fn get_two_mut(&mut self, a: usize, b: usize) -> Option<(&mut Self::Item, &mut Self::Item)>;
}

impl<T> SliceExt for [T] {
    type Item = T;

    fn get_two_mut(&mut self, a: usize, b: usize) -> Option<(&mut Self::Item, &mut Self::Item)> {
        if a == b || a >= self.len() || b >= self.len() {
            return None;
        }

        unsafe {
            let ar = &mut *(self.get_unchecked_mut(a) as *mut _);
            let br = &mut *(self.get_unchecked_mut(b) as *mut _);
            Some((ar, br))
        }
    }
}
