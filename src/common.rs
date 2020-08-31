use num_traits::Num;

use crate::bitfield;

bitfield! {
  #[derive(Clone)]
  pub struct Flags(u32);
  impl Debug;

  pub hit_left_wall, set_hit_left_wall: 0; // 0x01
  pub hit_top_wall, set_hit_top_wall: 1; // 0x02
  pub hit_right_wall, set_hit_right_wall: 2; // 0x04
  pub hit_bottom_wall, set_hit_bottom_wall: 3; // 0x08
  pub hit_right_slope, set_hit_right_slope: 4; // 0x10
  pub hit_left_slope, set_hit_left_slope: 5; // 0x20
  pub flag_x40, set_flag_x40: 6; // 0x40
  pub flag_x80, set_flag_x80: 7; // 0x80
  pub in_water, set_in_water: 8; // 0x100
  pub flag_x200, set_flag_x200: 9; // 0x200
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

  // engine specific flags
  pub head_bounced, set_head_bounced: 31;
}

bitfield! {
  #[derive(Clone)]
  pub struct Equip(u16);
  impl Debug;

  pub has_booster_0_8, set_booster_0_8: 0;
  pub has_map, set_map: 1;
  pub has_arms_barrier, set_arms_barrier: 2;
  pub has_turbocharge, set_turbocharge: 3;
  pub has_air_tank, set_air_tank: 4;
  pub has_booster_2_0, set_booster_2_0: 5;
  pub has_mimiga_mask, set_mimiga_mask: 6;
  pub has_whimsical_star, set_whimsical_star: 7;
  pub has_nikumaru, set_nikumaru: 8;
  // 7 bits wasted, thx pixel
}

bitfield! {
  #[derive(Clone)]
  pub struct Cond(u16);
  impl Debug;

  pub interacted, set_interacted: 0; // 0x01
  pub hidden, set_hidden: 1; // 0x02
  pub fallen, set_fallen: 2; // 0x04
  pub explode_die, set_explode_die: 3; // 0x08
  pub damage_boss, set_damage_boss: 4; // 0x10
  pub cond_x20, set_cond_x20: 5; // 0x20
  pub cond_x40, set_cond_x40: 6; // 0x40
  pub alive, set_alive: 7; // 0x80
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

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Left => { Direction::Right }
            Direction::Up => { Direction::Bottom }
            Direction::Right => { Direction::Left }
            Direction::Bottom => { Direction::Up }
        }
    }

    pub fn vector_x(&self) -> isize {
        match self {
            Direction::Left => { -1 }
            Direction::Up => { 0 }
            Direction::Right => { 1 }
            Direction::Bottom => { 0 }
        }
    }

    pub fn vector_y(&self) -> isize {
        match self {
            Direction::Left => { 0 }
            Direction::Up => { -1 }
            Direction::Right => { 0 }
            Direction::Bottom => { 1 }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rect<T: Num + Copy = isize> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl<S: Num + Copy> Rect<S> {
    pub fn new<T: Num + Copy>(left: T, top: T, right: T, bottom: T) -> Rect<T> {
        Rect {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn new_size<T: Num + Copy>(x: T, y: T, width: T, height: T) -> Rect<T> {
        Rect {
            left: x,
            top: y,
            right: x.add(width),
            bottom: y.add(height),
        }
    }

    pub fn from(rect: crate::ggez::graphics::Rect) -> Rect<f32> {
        Rect {
            left: rect.x,
            top: rect.y,
            right: (rect.x + rect.w),
            bottom: (rect.y + rect.h),
        }
    }
}
