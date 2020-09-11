use crate::common::{Condition, Direction, Rect, Flag};

pub struct Bullet {
    pub btype: u16,
    pub x: isize,
    pub y: isize,
    pub vel_x: isize,
    pub vel_y: isize,
    pub target_x: isize,
    pub target_y: isize,
    pub life: u16,
    pub lifetime: u16,
    pub damage: u16,
    pub cond: Condition,
    pub flags: Flag,
    pub direction: Direction,
    pub anim_rect: Rect<usize>,
    pub enemy_hit_width: u32,
    pub enemy_hit_height: u32,
    pub block_hit_width: u32,
    pub block_hit_height: u32,
    pub anim_num: u16,
    pub anim_counter: u16,
    pub action_num: u16,
    pub action_counter: u16,
    pub display_bounds: Rect<usize>,
}

impl Bullet {

}
