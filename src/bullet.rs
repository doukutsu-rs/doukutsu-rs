use num_traits::clamp;

use crate::caret::CaretType;
use crate::common::{Condition, Direction, Flag, Rect};
use crate::engine_constants::{BulletData, EngineConstants};
use crate::physics::{OFF_X, OFF_Y, PhysicalEntity};
use crate::SharedGameState;
use crate::stage::Stage;
use crate::npc::NPCMap;

pub struct BulletManager {
    pub bullets: Vec<Bullet>,
}

impl BulletManager {
    #[allow(clippy::new_without_default)]
    pub fn new() -> BulletManager {
        BulletManager {
            bullets: Vec::with_capacity(32),
        }
    }

    pub fn create_bullet(&mut self, x: isize, y: isize, btype: u16, direction: Direction, constants: &EngineConstants) {
        self.bullets.push(Bullet::new(x, y, btype, direction, constants));
    }

    pub fn tick_bullets(&mut self, state: &mut SharedGameState, player: &dyn PhysicalEntity, stage: &mut Stage) {
        for bullet in self.bullets.iter_mut() {
            if bullet.life < 1 {
                bullet.cond.set_alive(false);
                continue;
            }

            bullet.tick(state, player);
            bullet.hit_flags.0 = 0;
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
    pub hit_flags: Flag,
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
            hit_flags: Flag(0),
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

    fn tick_fireball(&mut self, state: &mut SharedGameState, player: &dyn PhysicalEntity) {
        self.action_counter += 1;
        if self.action_counter > self.lifetime {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);
            return;
        }

        if (self.flags.hit_left_wall() && self.flags.hit_right_wall())
            || (self.flags.hit_top_wall() && self.flags.hit_bottom_wall()) {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
            // todo play sound 28
            return;
        }

        // bounce off walls
        match self.direction {
            Direction::Left if self.flags.hit_left_wall() => {
                self.direction = Direction::Right;
            }
            Direction::Right if self.flags.hit_right_wall() => {
                self.direction = Direction::Left;
            }
            _ => {}
        }

        if self.action_num == 0 {
            self.action_num = 1;

            match self.direction {
                Direction::Left => {
                    self.vel_x = -0x400;
                }
                Direction::Right => {
                    self.vel_x = 0x400;
                }
                Direction::Up => {
                    self.vel_x = player.vel_x();

                    self.direction = if self.vel_x < 0 {
                        Direction::Left
                    } else {
                        Direction::Right
                    };

                    self.vel_x += if player.direction() == Direction::Left {
                        -0x80
                    } else {
                        0x80
                    };

                    self.vel_y = -0x5ff;
                }
                Direction::Bottom => {
                    self.vel_x = player.vel_x();

                    self.direction = if self.vel_x < 0 {
                        Direction::Left
                    } else {
                        Direction::Right
                    };

                    self.vel_y = 0x5ff;
                }
            }
        } else {
            if self.flags.hit_bottom_wall() {
                self.vel_y = -0x400;
            } else if self.flags.hit_left_wall() {
                self.vel_x = 0x400;
            } else if self.flags.hit_right_wall() {
                self.vel_x = -0x400;
            }

            self.vel_y += 0x55;
            if self.vel_y > 0x3ff {
                self.vel_y = 0x3ff;
            }

            self.x += self.vel_x;
            self.y += self.vel_y;

            if self.flags.hit_left_wall() || self.flags.hit_right_wall() || self.flags.hit_bottom_wall() {
                // todo play sound 34
            }
        }

        self.anim_num += 1;

        if self.btype == 7 { // level 1
        }
    }

    pub fn tick(&mut self, state: &mut SharedGameState, player: &PhysicalEntity) {
        if self.lifetime == 0 {
            self.cond.set_alive(false);
            return;
        }

        match self.btype {
            4 | 5 | 6 => {
                self.tick_polar_star(state);
            }
            7 | 8 | 9 => {
                self.tick_fireball(state, player);
            }
            _ => { self.cond.set_alive(false); }
        }
    }

    pub fn vanish(&mut self, state: &mut SharedGameState) {
        if self.btype != 37 && self.btype != 38 && self.btype != 39 {
            // todo play sound 28
        } else {
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Up);
        }

        self.cond.set_alive(false);
        state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Right);
    }

    fn judge_hit_block_destroy(&mut self, x: isize, y: isize, hit_attribs: &[u8; 4], state: &mut SharedGameState) {
        let mut hits = [false; 4];
        let mut block_x = (x * 16 + 8) * 0x200;
        let mut block_y = (y * 16 + 8) * 0x200;

        for (i, &attr) in hit_attribs.iter().enumerate() {
            if self.flags.snack_destroy() {
                hits[i] = attr == 0x41 || attr == 0x61;
            } else {
                hits[i] = attr == 0x41 || attr == 0x43 || attr == 0x61;
            }
        }

        // left wall
        if hits[0] && hits[2] {
            if (self.x - self.hit_bounds.left as isize) < block_x {
                self.hit_flags.set_hit_left_wall(true);
            }
        } else if hits[0] && !hits[2] {
            if (self.x - self.hit_bounds.left as isize) < block_x
                && (self.y - self.hit_bounds.top as isize) < block_y - (3 * 0x200) {
                self.hit_flags.set_hit_left_wall(true);
            }
        } else if !hits[0] && hits[2]
            && (self.x - self.hit_bounds.left as isize) < block_x
            && (self.y + self.hit_bounds.top as isize) > block_y + (3 * 0x200) {
            self.hit_flags.set_hit_left_wall(true);
        }

        // right wall
        if hits[1] && hits[3] {
            if (self.x + self.hit_bounds.right as isize) > block_x {
                self.hit_flags.set_hit_right_wall(true);
            }
        } else if hits[1] && !hits[3] {
            if (self.x + self.hit_bounds.right as isize) > block_x
                && (self.y - self.hit_bounds.top as isize) < block_y - (3 * 0x200) {
                self.hit_flags.set_hit_right_wall(true);
            }
        } else if !hits[1] && hits[3]
            && (self.x + self.hit_bounds.right as isize) > block_x
            && (self.y + self.hit_bounds.top as isize) > block_y + (3 * 0x200) {
            self.hit_flags.set_hit_right_wall(true);
        }

        // ceiling
        if hits[0] && hits[1] {
            if (self.y - self.hit_bounds.top as isize) < block_y {
                self.hit_flags.set_hit_top_wall(true);
            }
        } else if hits[0] && !hits[1] {
            if (self.x - self.hit_bounds.left as isize) < block_x - (3 * 0x200)
                && (self.y - self.hit_bounds.top as isize) < block_y {
                self.hit_flags.set_hit_top_wall(true);
            }
        } else if !hits[0] && hits[1]
            && (self.x + self.hit_bounds.right as isize) > block_x + (3 * 0x200)
            && (self.y - self.hit_bounds.top as isize) < block_y {
            self.hit_flags.set_hit_top_wall(true);
        }

        // ground
        if hits[2] && hits[3] {
            if (self.y + self.hit_bounds.bottom as isize) > block_y {
                self.hit_flags.set_hit_bottom_wall(true);
            }
        } else if hits[2] && !hits[3] {
            if (self.x - self.hit_bounds.left as isize) < block_x - (3 * 0x200)
                && (self.y + self.hit_bounds.bottom as isize) > block_y {
                self.hit_flags.set_hit_bottom_wall(true);
            }
        } else if !hits[2] && hits[3]
            && (self.x + self.hit_bounds.right as isize) > block_x + (3 * 0x200)
            && (self.y + self.hit_bounds.bottom as isize) > block_y {
            self.hit_flags.set_hit_bottom_wall(true);
        }

        if self.flags.hit_bottom_wall() {
            if self.hit_flags.hit_left_wall() {
                self.x = block_x + self.hit_bounds.right as isize;
            } else if self.hit_flags.hit_right_wall() {
                self.x = block_x + self.hit_bounds.left as isize;
            } else if self.hit_flags.hit_left_wall() {
                self.x = block_x + self.hit_bounds.right as isize;
            } else if self.hit_flags.hit_right_wall() {
                self.x = block_x + self.hit_bounds.left as isize;
            }
        } else if self.hit_flags.hit_left_wall() || self.hit_flags.hit_top_wall()
            || self.hit_flags.hit_right_wall() || self.hit_flags.hit_bottom_wall() {
            self.vanish(state);
        }
    }
}

impl PhysicalEntity for Bullet {
    #[inline(always)]
    fn x(&self) -> isize {
        self.x
    }

    #[inline(always)]
    fn y(&self) -> isize {
        self.y
    }

    #[inline(always)]
    fn vel_x(&self) -> isize {
        self.vel_x
    }

    #[inline(always)]
    fn vel_y(&self) -> isize {
        self.vel_y
    }

    #[inline(always)]
    fn size(&self) -> u8 {
        1
    }

    #[inline(always)]
    fn hit_bounds(&self) -> &Rect<usize> {
        &self.hit_bounds
    }

    #[inline(always)]
    fn set_x(&mut self, x: isize) {
        self.x = x;
    }

    #[inline(always)]
    fn set_y(&mut self, y: isize) {
        self.y = y;
    }

    #[inline(always)]
    fn set_vel_x(&mut self, vel_x: isize) {
        self.vel_x = vel_x;
    }

    #[inline(always)]
    fn set_vel_y(&mut self, vel_y: isize) {
        self.vel_y = vel_y;
    }

    #[inline(always)]
    fn cond(&mut self) -> &mut Condition {
        &mut self.cond
    }

    #[inline(always)]
    fn flags(&mut self) -> &mut Flag {
        &mut self.hit_flags
    }

    #[inline(always)]
    fn direction(&self) -> Direction {
        self.direction
    }

    #[inline(always)]
    fn is_player(&self) -> bool {
        false
    }

    fn judge_hit_block(&mut self, state: &SharedGameState, x: isize, y: isize) {
        if (self.x - self.hit_bounds.left as isize) < (x * 16 + 8) * 0x200
            && (self.x + self.hit_bounds.right as isize) > (x * 16 - 8) * 0x200
            && (self.y - self.hit_bounds.top as isize) < (y * 16 + 8) * 0x200
            && (self.y + self.hit_bounds.bottom as isize) > (y * 16 - 8) * 0x200
        {
            self.hit_flags.set_weapon_hit_block(true);
        }
    }

    fn tick_map_collisions(&mut self, state: &mut SharedGameState, stage: &mut Stage) {
        let x = clamp(self.x() / 16 / 0x200, 0, stage.map.width as isize);
        let y = clamp(self.y() / 16 / 0x200, 0, stage.map.height as isize);
        let mut hit_attribs = [0u8; 4];

        if self.flags.hit_right_wall() { // ???
            return;
        }

        for (idx, (&ox, &oy)) in OFF_X.iter().zip(OFF_Y.iter()).enumerate() {
            if idx == 4 {
                break;
            }

            let attrib = stage.map.get_attribute((x + ox) as usize, (y + oy) as usize);
            hit_attribs[idx] = attrib;

            match attrib {
                // Blocks
                0x41 | 0x44 | 0x61 | 0x64 => {
                    self.judge_hit_block(state, x + ox, y + oy);
                }
                0x43 => {
                    self.judge_hit_block(state, x + ox, y + oy);

                    if self.hit_flags.0 != 0 && (self.flags.hit_left_slope() || self.flags.snack_destroy()) {
                        if !self.flags.snack_destroy() {
                            self.cond.set_alive(false);
                        }

                        state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
                        // todo play sound 12

                        for _ in 0..4 {
                            let mut npc = NPCMap::create_npc(4, &state.npc_table);

                            npc.cond.set_alive(true);
                            npc.direction = Direction::Left;
                            npc.x = x * 16 * 0x200;
                            npc.y = y * 16 * 0x200;
                            npc.vel_x = state.game_rng.range(-0x200..0x200) as isize;
                            npc.vel_y = state.game_rng.range(-0x200..0x200) as isize;

                            state.new_npcs.push(npc);
                        }

                        if let Some(tile) = stage.map.tiles.get_mut(stage.map.width * (y + oy) as usize + (x + ox) as usize) {
                            *tile = tile.wrapping_sub(1);
                        }
                    }
                }
                // Slopes
                0x50 | 0x70 => {
                    self.judge_hit_triangle_a(x + ox, y + oy);
                }
                0x51 | 0x71 => {
                    self.judge_hit_triangle_b(x + ox, y + oy);
                }
                0x52 | 0x72 => {
                    self.judge_hit_triangle_c(x + ox, y + oy);
                }
                0x53 | 0x73 => {
                    self.judge_hit_triangle_d(x + ox, y + oy);
                }
                0x54 | 0x74 => {
                    self.judge_hit_triangle_e(x + ox, y + oy);
                }
                0x55 | 0x75 => {
                    self.judge_hit_triangle_f(x + ox, y + oy);
                }
                0x56 | 0x76 => {
                    self.judge_hit_triangle_g(x + ox, y + oy);
                }
                0x57 | 0x77 => {
                    self.judge_hit_triangle_h(x + ox, y + oy);
                }
                _ => {}
            }
        }

        self.judge_hit_block_destroy(x, y, &hit_attribs, state);
    }
}
