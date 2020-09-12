use crate::caret::CaretType;
use crate::common::{Condition, Direction, Flag, Rect};
use crate::engine_constants::{BulletData, EngineConstants};
use crate::physics::PhysicalEntity;
use crate::SharedGameState;
use crate::stage::Stage;

pub struct BulletManager {
    pub bullets: Vec<Bullet>,
}

impl BulletManager {
    pub fn new() -> BulletManager {
        BulletManager {
            bullets: Vec::with_capacity(32),
        }
    }

    pub fn create_bullet(&mut self, x: isize, y: isize, btype: u16, direction: Direction, constants: &EngineConstants) {
        self.bullets.push(Bullet::new(x, y, btype, direction, constants));
    }

    pub fn tick_bullets(&mut self, state: &mut SharedGameState, stage: &Stage) {
        for bullet in self.bullets.iter_mut() {
            bullet.tick(state);
            bullet.flags.0 = 0;
            bullet.tick_map_collisions(state, stage);
        }

        self.bullets.retain(|b| !b.is_dead());
    }

    pub fn count_bullets(&self, btype: u16) -> usize {
        self.bullets.iter().filter(|b| b.btype == btype).count()
    }

    pub fn count_bullets_multi(&self, btypes: [u16; 3]) -> usize {
        self.bullets.iter().filter(|b| btypes.contains(&b.btype)).count()
    }
}

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
    pub anim_num: u16,
    pub anim_counter: u16,
    pub action_num: u16,
    pub action_counter: u16,
    pub hit_bounds: Rect<usize>,
    pub display_bounds: Rect<usize>,
}

impl Bullet {
    pub fn new(x: isize, y: isize, btype: u16, direction: Direction, constants: &EngineConstants) -> Bullet {
        let bullet = constants.weapon.bullet_table
            .get(btype as usize)
            .unwrap_or_else(|| &BulletData {
                damage: 0,
                life: 0,
                lifetime: 0,
                flags: Flag(0),
                enemy_hit_width: 0,
                enemy_hit_height: 0,
                block_hit_width: 0,
                block_hit_height: 0,
                display_bounds: Rect { left: 0, top: 0, right: 0, bottom: 0 },
            });

        Bullet {
            btype,
            x,
            y,
            vel_x: 0,
            vel_y: 0,
            target_x: 0,
            target_y: 0,
            life: bullet.life as u16,
            lifetime: bullet.lifetime,
            damage: bullet.damage as u16,
            cond: Condition(0x80),
            flags: bullet.flags,
            direction,
            anim_rect: Rect::new(0, 0, 0, 0),
            enemy_hit_width: bullet.enemy_hit_width as u32 * 0x200,
            enemy_hit_height: bullet.enemy_hit_height as u32 * 0x200,
            anim_num: 0,
            anim_counter: 0,
            action_num: 0,
            action_counter: 0,
            display_bounds: Rect::new(
                bullet.display_bounds.left as usize * 0x200,
                bullet.display_bounds.top as usize * 0x200,
                bullet.display_bounds.right as usize * 0x200,
                bullet.display_bounds.bottom as usize * 0x200,
            ),
            hit_bounds: Rect::new(
                bullet.block_hit_width as usize * 0x200,
                bullet.block_hit_height as usize * 0x200,
                bullet.block_hit_width as usize * 0x200,
                bullet.block_hit_height as usize * 0x200,
            ),
        }
    }

    #[inline]
    pub fn is_dead(&self) -> bool {
        !self.cond.alive()
    }

    fn tick_polar_star(&mut self, state: &mut SharedGameState) {
        self.action_counter += 1;
        if self.action_counter > self.lifetime {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);
            return;
        }

        if self.action_num == 0 {
            self.action_num = 1;

            match self.direction {
                Direction::Left => { self.vel_x = -0x1000 }
                Direction::Up => { self.vel_y = -0x1000 }
                Direction::Right => { self.vel_x = 0x1000 }
                Direction::Bottom => { self.vel_y = 0x1000 }
            }

            match self.btype {
                4 => {
                    match self.direction {
                        Direction::Left | Direction::Right => {
                            self.enemy_hit_height = 0x400;
                        }
                        Direction::Up | Direction::Bottom => {
                            self.enemy_hit_width = 0x400;
                        }
                    }
                }
                5 => {
                    match self.direction {
                        Direction::Left | Direction::Right => {
                            self.enemy_hit_height = 0x800;
                        }
                        Direction::Up | Direction::Bottom => {
                            self.enemy_hit_width = 0x800;
                        }
                    }
                }
                6 => {
                    // level 3 uses default values
                }
                _ => { unreachable!() }
            }
        } else {
            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        match self.btype {
            4 => {
                if self.direction == Direction::Up || self.direction == Direction::Bottom {
                    self.anim_num = 1;
                    self.anim_rect = state.constants.weapon.bullet_rects.b004_polar_star_l1[1];
                } else {
                    self.anim_num = 0;
                    self.anim_rect = state.constants.weapon.bullet_rects.b004_polar_star_l1[0];
                }
            }
            5 => {
                if self.direction == Direction::Up || self.direction == Direction::Bottom {
                    self.anim_num = 1;
                    self.anim_rect = state.constants.weapon.bullet_rects.b005_polar_star_l2[1];
                } else {
                    self.anim_num = 0;
                    self.anim_rect = state.constants.weapon.bullet_rects.b005_polar_star_l2[0];
                }
            }
            6 => {
                if self.direction == Direction::Up || self.direction == Direction::Bottom {
                    self.anim_num = 1;
                    self.anim_rect = state.constants.weapon.bullet_rects.b006_polar_star_l3[1];
                } else {
                    self.anim_num = 0;
                    self.anim_rect = state.constants.weapon.bullet_rects.b006_polar_star_l3[0];
                }
            }
            _ => { unreachable!() }
        }
    }

    pub fn tick(&mut self, state: &mut SharedGameState) {
        if self.lifetime == 0 {
            self.cond.set_alive(false);
            return;
        }

        match self.btype {
            4 | 5 | 6 => {
                self.tick_polar_star(state);
            }
            _ => { self.cond.set_alive(false); }
        }
    }
}

impl PhysicalEntity for Bullet {
    fn x(&self) -> isize {
        self.x
    }

    fn y(&self) -> isize {
        self.y
    }

    fn vel_x(&self) -> isize {
        self.vel_x
    }

    fn vel_y(&self) -> isize {
        self.vel_y
    }

    fn size(&self) -> u8 {
        1
    }

    fn hit_bounds(&self) -> &Rect<usize> {
        &self.hit_bounds
    }

    fn set_x(&mut self, x: isize) {
        self.x = x;
    }

    fn set_y(&mut self, y: isize) {
        self.y = y;
    }

    fn set_vel_x(&mut self, vel_x: isize) {
        self.vel_x = vel_x;
    }

    fn set_vel_y(&mut self, vel_y: isize) {
        self.vel_y = vel_y;
    }

    fn cond(&mut self) -> &mut Condition {
        &mut self.cond
    }

    fn flags(&mut self) -> &mut Flag {
        &mut self.flags
    }

    fn is_player(&self) -> bool {
        false
    }

    /*fn judge_hit_block(&mut self, state: &SharedGameState, x: isize, y: isize) {

    }*/
}
