use std::cmp::Ordering;

use num_traits::{AsPrimitive, Num};
use serde::{Deserialize, Serialize};

use crate::bitfield;

/// Multiply cave story degrees (0-255, which corresponds to 0°-360°) with this to get
/// respective value in radians.
pub const CDEG_RAD: f64 = std::f64::consts::PI / 128.0;

bitfield! {
  #[derive(Clone, Copy)]
  pub struct Flag(u32);
  impl Debug;

  pub hit_left_wall, set_hit_left_wall: 0; // 0x01
  pub hit_top_wall, set_hit_top_wall: 1; // 0x02
  pub hit_right_wall, set_hit_right_wall: 2; // 0x04
  pub hit_bottom_wall, set_hit_bottom_wall: 3; // 0x08
  pub hit_right_slope, set_hit_right_slope: 4; // 0x10
  pub hit_left_slope, set_hit_left_slope: 5; // 0x20
  pub snack_destroy, set_snack_destroy: 6; // 0x40
  pub flag_x80, set_flag_x80: 7; // 0x80
  pub in_water, set_in_water: 8; // 0x100
  pub weapon_hit_block, set_weapon_hit_block: 9; // 0x200
  pub hit_by_spike, set_hit_by_spike: 10; // 0x400
  pub water_splash_facing_right, set_water_splash_facing_right: 11; // 0x800
  pub force_left, set_force_left: 12; // 0x1000
  pub force_up, set_force_up: 13; // 0x2000
  pub force_right, set_force_right: 14; // 0x4000
  pub force_down, set_force_down: 15; // 0x8000
  pub hit_left_bigger_half, set_hit_left_bigger_half: 16; // 0x10000
  pub hit_left_smaller_half, set_hit_left_smaller_half: 17; // 0x20000
  pub hit_right_smaller_half, set_hit_right_smaller_half: 18; // 0x40000
  pub hit_right_bigger_half, set_hit_right_bigger_half: 19; // 0x80000
}

bitfield! {
  #[derive(Clone, Copy)]
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
  pub drs_dont_remove, set_drs_dont_remove: 13;
  pub drs_boss, set_drs_boss: 14;
  pub drs_destroyed, set_drs_destroyed: 15;
}

bitfield! {
  #[derive(Clone, Copy, Serialize, Deserialize)]
  pub struct ControlFlags(u16);
  impl Debug;

  pub tick_world, set_tick_world: 0; // 0x01
  pub control_enabled, set_control_enabled: 1; // 0x02
  pub interactions_disabled, set_interactions_disabled: 2; // 0x04
  pub credits_running, set_credits_running: 3; // 0x08

  // engine specific flags
  pub friendly_fire, set_friendly_fire: 14;
  pub wind, set_wind: 15;
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
    pub fn from_int(val: usize) -> Option<FadeDirection> {
        match val {
            0 => { Some(FadeDirection::Left) }
            1 => { Some(FadeDirection::Up) }
            2 => { Some(FadeDirection::Right) }
            3 => { Some(FadeDirection::Down) }
            4 => { Some(FadeDirection::Center) }
            _ => { None }
        }
    }

    pub fn opposite(&self) -> FadeDirection {
        match self {
            FadeDirection::Left => { FadeDirection::Right }
            FadeDirection::Up => { FadeDirection::Down }
            FadeDirection::Right => { FadeDirection::Left }
            FadeDirection::Down => { FadeDirection::Up }
            FadeDirection::Center => { FadeDirection::Center }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn from_int(val: usize) -> Option<Direction> {
        match val {
            0 => { Some(Direction::Left) }
            1 => { Some(Direction::Up) }
            2 => { Some(Direction::Right) }
            3 => { Some(Direction::Bottom) }
            _ => { None }
        }
    }

    pub fn from_int_facing(val: usize) -> Option<Direction> {
        match val {
            0 => { Some(Direction::Left) }
            1 => { Some(Direction::Up) }
            2 => { Some(Direction::Right) }
            3 => { Some(Direction::Bottom) }
            4 => { Some(Direction::FacingPlayer) }
            _ => { None }
        }
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Left => { Direction::Right }
            Direction::Up => { Direction::Bottom }
            Direction::Right => { Direction::Left }
            Direction::Bottom => { Direction::Up }
            Direction::FacingPlayer => unreachable!(),
        }
    }

    pub fn vector_x(&self) -> isize {
        match self {
            Direction::Left => { -1 }
            Direction::Up => { 0 }
            Direction::Right => { 1 }
            Direction::Bottom => { 0 }
            Direction::FacingPlayer => unreachable!(),
        }
    }

    pub fn vector_y(&self) -> isize {
        match self {
            Direction::Left => { 0 }
            Direction::Up => { -1 }
            Direction::Right => { 0 }
            Direction::Bottom => { 1 }
            Direction::FacingPlayer => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect<T: Num + PartialOrd + Copy = isize> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl<T: Num + PartialOrd + Copy> Rect<T> {
    pub fn new(left: T, top: T, right: T, bottom: T) -> Rect<T> {
        Rect {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn new_size(x: T, y: T, width: T, height: T) -> Rect<T> {
        Rect {
            left: x,
            top: y,
            right: x.add(width),
            bottom: y.add(height),
        }
    }

    pub fn from(rect: ggez::graphics::Rect) -> Rect<f32> {
        Rect {
            left: rect.x,
            top: rect.y,
            right: (rect.x + rect.w),
            bottom: (rect.y + rect.h),
        }
    }

    pub fn width(&self) -> T {
        if let Some(Ordering::Greater) = self.left.partial_cmp(&self.right) {
            self.left.sub(self.right)
        } else {
            self.right.sub(self.left)
        }
    }

    pub fn height(&self) -> T {
        if let Some(Ordering::Greater) = self.top.partial_cmp(&self.bottom) {
            self.top.sub(self.bottom)
        } else {
            self.bottom.sub(self.top)
        }
    }
}

impl<T: Num + PartialOrd + Copy + AsPrimitive<f32>> Into<ggez::graphics::Rect> for Rect<T> {
    fn into(self) -> ggez::graphics::Rect {
        ggez::graphics::Rect::new(self.left.as_(),
                                  self.top.as_(),
                                  self.width().as_(),
                                  self.height().as_())
    }
}

#[inline(always)]
pub fn fix9_scale(val: isize, scale: f32) -> f32 {
    (val as f64 * scale as f64 / 512.0).floor() as f32 / scale
}

#[inline(always)]
fn lerp_f64(v1: f64, v2: f64, t: f64) -> f64 {
    v1 * (1.0 - t.fract()) + v2 * t.fract()
}

pub fn interpolate_fix9_scale(old_val: isize, val: isize, frame_delta: f64) -> f32 {
    if (frame_delta - 1.0).abs() < 0.001 {
        return (val / 0x200) as f32;
    }

    (lerp_f64(old_val as f64, val as f64, frame_delta) / 512.0) as f32
    //((lerp_f64(old_val as f64, val as f64, frame_delta) * scale as f64 / 512.0).floor() / (scale as f64)) as f32
}
