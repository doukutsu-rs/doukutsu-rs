use std::ops::{Add, Sub};

use num_traits::clamp;

use crate::caret::CaretType;
use crate::common::{BulletFlag, Condition, Direction, Flag, Rect};
use crate::engine_constants::{BulletData, EngineConstants};
use crate::npc::list::NPCList;
use crate::npc::NPC;
use crate::physics::{PhysicalEntity, OFF_X, OFF_Y};
use crate::player::TargetPlayer;
use crate::rng::{XorShift, Xoroshiro32PlusPlus, RNG};
use crate::shared_game_state::SharedGameState;
use crate::stage::Stage;

pub struct BulletManager {
    pub bullets: Vec<Bullet>,
    pub new_bullets: Vec<Bullet>,
    pub seeder: XorShift,
}

impl BulletManager {
    #[allow(clippy::new_without_default)]
    pub fn new() -> BulletManager {
        BulletManager {
            bullets: Vec::with_capacity(64),
            new_bullets: Vec::with_capacity(8),
            seeder: XorShift::new(0x359c482f),
        }
    }

    pub fn create_bullet(
        &mut self,
        x: i32,
        y: i32,
        btype: u16,
        owner: TargetPlayer,
        direction: Direction,
        constants: &EngineConstants,
    ) {
        let mut bullet = Bullet::new(x, y, btype, owner, direction, constants);
        bullet.rng = Xoroshiro32PlusPlus::new(self.seeder.next_u32());

        self.bullets.push(bullet);
    }

    pub fn push_bullet(&mut self, mut bullet: Bullet) {
        bullet.rng = Xoroshiro32PlusPlus::new(self.seeder.next_u32());
        self.bullets.push(bullet);
    }

    pub fn tick_bullets(
        &mut self,
        state: &mut SharedGameState,
        players: [&dyn PhysicalEntity; 2],
        npc_list: &NPCList,
        stage: &mut Stage,
    ) {
        let mut i = 0;
        while i < self.bullets.len() {
            {
                let bullet = unsafe { self.bullets.get_unchecked_mut(i) };
                i += 1;

                if bullet.life < 1 {
                    bullet.cond.set_alive(false);
                    continue;
                }

                bullet.tick(state, players, npc_list, &mut self.new_bullets);
                bullet.tick_map_collisions(state, npc_list, stage);
            }

            for bullet in self.new_bullets.iter_mut() {
                bullet.rng = Xoroshiro32PlusPlus::new(self.seeder.next_u32());
            }

            self.bullets.append(&mut self.new_bullets);
        }

        self.bullets.retain(|b| !b.is_dead());
    }

    pub fn count_bullets(&self, btype: u16, player_id: TargetPlayer) -> usize {
        self.bullets.iter().filter(|b| b.owner == player_id && b.btype == btype).count()
    }

    pub fn count_bullets_type_idx_all(&self, type_idx: u16) -> usize {
        self.bullets.iter().filter(|b| (b.btype.saturating_sub(2) / 3) == type_idx).count()
    }

    pub fn count_bullets_multi(&self, btypes: &[u16], player_id: TargetPlayer) -> usize {
        self.bullets.iter().filter(|b| b.owner == player_id && btypes.contains(&b.btype)).count()
    }
}

pub struct Bullet {
    pub btype: u16,
    pub x: i32,
    pub y: i32,
    pub vel_x: i32,
    pub vel_y: i32,
    pub target_x: i32,
    pub target_y: i32,
    pub prev_x: i32,
    pub prev_y: i32,
    pub life: u16,
    pub lifetime: u16,
    pub damage: i16,
    pub counter1: u16,
    pub rng: Xoroshiro32PlusPlus,
    pub owner: TargetPlayer,
    pub cond: Condition,
    pub weapon_flags: BulletFlag,
    pub flags: Flag,
    pub direction: Direction,
    pub anim_rect: Rect<u16>,
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
    pub fn new(
        x: i32,
        y: i32,
        btype: u16,
        owner: TargetPlayer,
        direction: Direction,
        constants: &EngineConstants,
    ) -> Bullet {
        let bullet = constants.weapon.bullet_table.get(btype as usize).unwrap_or_else(|| &BulletData {
            damage: 0,
            life: 0,
            lifetime: 0,
            flags: BulletFlag(0),
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
            prev_x: x,
            prev_y: y,
            life: bullet.life as u16,
            lifetime: bullet.lifetime,
            damage: bullet.damage as i16,
            counter1: 0,
            rng: Xoroshiro32PlusPlus::new(1),
            owner,
            cond: Condition(0x80),
            weapon_flags: bullet.flags,
            flags: Flag(0),
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

    fn tick_snake_1(&mut self, state: &mut SharedGameState) {
        self.action_counter += 1;
        if self.action_counter > self.lifetime {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);
            return;
        }

        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = state.game_rng.range(0..2) as u16;

            match self.direction {
                Direction::Left => self.vel_x = -0x600,
                Direction::Up => self.vel_y = -0x600,
                Direction::Right => self.vel_x = 0x600,
                Direction::Bottom => self.vel_y = 0x600,
                Direction::FacingPlayer => unreachable!(),
            }
        } else {
            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        self.anim_num = (self.anim_num + 1) % 4;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.weapon.bullet_rects.b001_snake_l1[self.anim_num as usize + dir_offset];
    }

    fn tick_snake_2(&mut self, state: &mut SharedGameState, npc_list: &NPCList) {
        self.action_counter += 1;
        if self.action_counter > self.lifetime {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);
            return;
        }

        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = state.game_rng.range(0..2) as u16;

            match self.direction {
                Direction::Left => {
                    self.vel_x = -0x200;
                    self.vel_y = if self.target_x & 1 == 0 { -0x400 } else { 0x400 };
                }
                Direction::Up => {
                    self.vel_y = -0x200;
                    self.vel_x = if self.target_x & 1 == 0 { -0x400 } else { 0x400 };
                }
                Direction::Right => {
                    self.vel_x = 0x200;
                    self.vel_y = if self.target_x & 1 == 0 { -0x400 } else { 0x400 };
                }
                Direction::Bottom => {
                    self.vel_y = 0x200;
                    self.vel_x = if self.target_x & 1 == 0 { -0x400 } else { 0x400 };
                }
                Direction::FacingPlayer => unreachable!(),
            };
        } else {
            match self.direction {
                Direction::Left => self.vel_x += -0x80,
                Direction::Up => self.vel_y += -0x80,
                Direction::Right => self.vel_x += 0x80,
                Direction::Bottom => self.vel_y += 0x80,
                Direction::FacingPlayer => unreachable!(),
            }

            if self.action_counter % 5 == 2 {
                match self.direction {
                    Direction::Left | Direction::Right => {
                        self.vel_y = if self.vel_y < 0 { 0x400 } else { -0x400 };
                    }
                    Direction::Up | Direction::Bottom => {
                        self.vel_x = if self.vel_x < 0 { 0x400 } else { -0x400 };
                    }
                    Direction::FacingPlayer => unreachable!(),
                }
            }

            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        self.anim_num = (self.anim_num + 1) % 3;

        self.anim_rect = state.constants.weapon.bullet_rects.b002_003_snake_l2_3[self.anim_num as usize];

        let mut npc = NPC::create(129, &state.npc_table);
        npc.cond.set_alive(true);
        npc.x = self.x;
        npc.y = self.y;
        npc.vel_y = -0x200;
        npc.action_counter2 = if self.btype == 3 { self.anim_num + 3 } else { self.anim_num };

        let _ = npc_list.spawn(0x100, npc);
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
                Direction::Left => self.vel_x = -0x1000,
                Direction::Up => self.vel_y = -0x1000,
                Direction::Right => self.vel_x = 0x1000,
                Direction::Bottom => self.vel_y = 0x1000,
                Direction::FacingPlayer => unreachable!(),
            }

            match self.btype {
                4 => match self.direction {
                    Direction::Left | Direction::Right => self.enemy_hit_height = 0x400,
                    Direction::Up | Direction::Bottom => self.enemy_hit_width = 0x400,
                    Direction::FacingPlayer => unreachable!(),
                },
                5 => match self.direction {
                    Direction::Left | Direction::Right => {
                        self.enemy_hit_height = 0x800;
                    }
                    Direction::Up | Direction::Bottom => {
                        self.enemy_hit_width = 0x800;
                    }
                    Direction::FacingPlayer => unreachable!(),
                },
                6 => {
                    // level 3 uses default values
                }
                _ => {
                    unreachable!()
                }
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
            _ => {
                unreachable!()
            }
        }
    }

    fn tick_fireball(&mut self, state: &mut SharedGameState, players: [&dyn PhysicalEntity; 2], npc_list: &NPCList) {
        self.action_counter += 1;
        if self.action_counter > self.lifetime {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);
            return;
        }

        if (self.flags.hit_left_wall() && self.flags.hit_right_wall())
            || (self.flags.hit_top_wall() && self.flags.hit_bottom_wall())
        {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
            state.sound_manager.play_sfx(28);
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
                    self.vel_x = players[self.owner.index()].vel_x();

                    self.direction = if self.vel_x < 0 { Direction::Left } else { Direction::Right };

                    self.vel_x += if players[self.owner.index()].direction() == Direction::Left { -0x80 } else { 0x80 };

                    self.vel_y = -0x5ff;
                }
                Direction::Bottom => {
                    self.vel_x = players[self.owner.index()].vel_x();

                    self.direction = if self.vel_x < 0 { Direction::Left } else { Direction::Right };

                    self.vel_y = 0x5ff;
                }
                Direction::FacingPlayer => unreachable!(),
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
                state.sound_manager.play_sfx(34);
            }
        }

        self.anim_num += 1;

        if self.btype == 7 {
            // level 1
            if self.anim_num > 3 {
                self.anim_num = 0;
            }

            let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

            self.anim_rect = state.constants.weapon.bullet_rects.b007_fireball_l1[self.anim_num as usize + dir_offset];
        } else {
            if self.anim_num > 2 {
                self.anim_num = 0;
            }

            let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

            self.anim_rect =
                state.constants.weapon.bullet_rects.b008_009_fireball_l2_3[self.anim_num as usize + dir_offset];

            let mut npc = NPC::create(129, &state.npc_table);
            npc.cond.set_alive(true);
            npc.x = self.x;
            npc.y = self.y;
            npc.vel_y = -0x200;
            npc.action_counter2 = if self.btype == 9 { self.anim_num + 3 } else { self.anim_num };

            let _ = npc_list.spawn(0x100, npc);
        }
    }

    fn tick_machine_gun(&mut self, state: &mut SharedGameState, npc_list: &NPCList) {
        self.action_counter += 1;
        if self.action_counter > self.lifetime {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);
            return;
        }

        if self.action_num == 0 {
            self.action_num = 1;

            match self.direction {
                Direction::Left => {
                    self.vel_x = -0x1000;
                    self.vel_y = self.rng.range(-0xaa..0xaa);
                }
                Direction::Up => {
                    self.vel_y = -0x1000;
                    self.vel_x = self.rng.range(-0xaa..0xaa);
                }
                Direction::Right => {
                    self.vel_x = 0x1000;
                    self.vel_y = self.rng.range(-0xaa..0xaa);
                }
                Direction::Bottom => {
                    self.vel_y = 0x1000;
                    self.vel_x = self.rng.range(-0xaa..0xaa);
                }
                Direction::FacingPlayer => unreachable!(),
            }
        } else {
            self.x += self.vel_x;
            self.y += self.vel_y;

            match self.btype {
                11 => {
                    let mut npc = NPC::create(127, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;

                    match self.direction {
                        Direction::Left | Direction::Right => {
                            npc.direction = Direction::Left;
                        }
                        Direction::Bottom | Direction::Up => {
                            npc.direction = Direction::Up;
                        }
                        _ => {}
                    }

                    let _ = npc_list.spawn(256, npc);
                }
                12 => {
                    let mut npc = NPC::create(128, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    npc.direction = self.direction;

                    let _ = npc_list.spawn(256, npc);
                }
                _ => {}
            }
        }

        match self.btype {
            10 => {
                self.anim_rect = state.constants.weapon.bullet_rects.b010_machine_gun_l1[self.direction as usize];
            }
            11 => {
                self.anim_rect = state.constants.weapon.bullet_rects.b011_machine_gun_l2[self.direction as usize];
            }
            12 => {
                self.anim_rect = state.constants.weapon.bullet_rects.b012_machine_gun_l3[self.direction as usize];
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn tick_blade_slash(&mut self, state: &mut SharedGameState) {
        if self.action_num == 0 {
            self.action_num = 1;
            self.x += if self.direction == Direction::Left { 0x2000 } else { -0x2000 };
            self.y -= 0x1800;
        }

        self.anim_counter += 1;
        if self.anim_counter > 2 {
            self.anim_counter = 0;
            self.anim_num += 1;

            self.damage = if self.anim_num == 1 { 2 } else { 1 };
            if self.anim_num > 4 {
                self.cond.set_alive(false);
                return;
            }
        }

        self.x += if self.direction == Direction::Left { -0x400 } else { 0x400 };
        self.y += 0x400;

        let dir_offset = if self.direction == Direction::Left { 0 } else { 5 };

        self.anim_rect = state.constants.weapon.bullet_rects.b023_blade_slash[self.anim_num as usize + dir_offset];
    }

    fn tick_blade_1(&mut self, state: &mut SharedGameState) {
        self.action_counter += 1;
        if self.action_counter > self.lifetime {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);
            return;
        }

        if self.action_counter == 3 {
            self.weapon_flags.set_flag_x04(false);
        }

        if self.action_counter % 5 == 1 {
            state.sound_manager.play_sfx(34);
        }

        if self.action_num == 0 {
            self.action_num = 1;

            match self.direction {
                Direction::Left => self.vel_x = -0x800,
                Direction::Up => self.vel_y = -0x800,
                Direction::Right => self.vel_x = 0x800,
                Direction::Bottom => self.vel_y = 0x800,
                Direction::FacingPlayer => unreachable!(),
            }
        } else {
            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num += 1;
            if self.anim_num > 3 {
                self.anim_num = 0;
            }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.weapon.bullet_rects.b025_blade_l1[self.anim_num as usize + dir_offset];
    }

    fn tick_blade_2(&mut self, state: &mut SharedGameState) {
        self.action_counter += 1;
        if self.action_counter > self.lifetime {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);
            return;
        }

        if self.action_counter == 3 {
            self.weapon_flags.set_flag_x04(false);
        }

        if self.action_counter % 7 == 1 {
            state.sound_manager.play_sfx(106);
        }

        if self.action_num == 0 {
            self.action_num = 1;

            match self.direction {
                Direction::Left => self.vel_x = -0x800,
                Direction::Up => self.vel_y = -0x800,
                Direction::Right => self.vel_x = 0x800,
                Direction::Bottom => self.vel_y = 0x800,
                Direction::FacingPlayer => unreachable!(),
            }
        } else {
            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num += 1;
            if self.anim_num > 3 {
                self.anim_num = 0;
            }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 4 };

        self.anim_rect = state.constants.weapon.bullet_rects.b026_blade_l2[self.anim_num as usize + dir_offset];
    }

    fn tick_blade_3(&mut self, state: &mut SharedGameState, new_bullets: &mut Vec<Bullet>) {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.vel_x = 0;
                    self.vel_y = 0;
                }

                match self.direction {
                    Direction::Left => self.vel_x = -0x800,
                    Direction::Up => self.vel_y = -0x800,
                    Direction::Right => self.vel_x = 0x800,
                    Direction::Bottom => self.vel_y = 0x800,
                    Direction::FacingPlayer => unreachable!(),
                }

                if self.life != 100 {
                    self.action_num = 2;
                    self.anim_num = 1;
                    self.damage = -1;
                    self.action_counter = 0;
                }

                self.action_counter += 1;
                if self.action_counter % 4 == 1 {
                    state.sound_manager.play_sfx(106);

                    self.counter1 += 1;
                    let direction = if self.counter1 % 2 != 0 { Direction::Left } else { Direction::Right };

                    new_bullets.push(Bullet::new(self.x, self.y, 23, self.owner, direction, &state.constants));
                }

                self.counter1 += 1;
                if self.counter1 == 5 {
                    self.weapon_flags.set_flag_x04(false);
                }

                if self.counter1 > self.lifetime {
                    self.cond.set_alive(false);
                    state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);
                    return;
                }
            }
            2 => {
                self.vel_x = 0;
                self.vel_y = 0;

                self.action_counter += 1;
                if self.rng.range(-1..1) == 0 {
                    state.sound_manager.play_sfx(106);

                    let x = self.rng.range(-64..64) * 0x200 + self.x;
                    let y = self.rng.range(-64..64) * 0x200 + self.y;
                    let direction = if self.rng.range(0..1) != 0 { Direction::Left } else { Direction::Right };

                    new_bullets.push(Bullet::new(x, y, 23, self.owner, direction, &state.constants));
                }

                if self.action_counter > 50 {
                    self.cond.set_alive(false);
                }
            }
            _ => {}
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        let dir_offset = match self.direction {
            Direction::Left => 0,
            Direction::Up => 2,
            Direction::Right => 4,
            Direction::Bottom => 6,
            _ => unreachable!(),
        };

        self.anim_rect = state.constants.weapon.bullet_rects.b027_blade_l3[self.anim_num as usize + dir_offset];

        if self.action_counter % 2 != 0 {
            self.anim_rect = Rect { left: 0, top: 0, right: 0, bottom: 0 };
        }
    }

    fn tick_spur(&mut self, state: &mut SharedGameState, new_bullets: &mut Vec<Bullet>) {
        self.action_counter += 1;
        if self.action_counter > self.lifetime {
            self.cond.set_alive(false);
            state.create_caret(self.x, self.y, CaretType::Shoot, Direction::Left);
            return;
        }

        if self.action_num == 0 {
            self.action_num = 1;

            match self.direction {
                Direction::Left => self.vel_x = -0x1000,
                Direction::Up => self.vel_y = -0x1000,
                Direction::Right => self.vel_x = 0x1000,
                Direction::Bottom => self.vel_y = 0x1000,
                Direction::FacingPlayer => unreachable!(),
            }

            match self.btype {
                37 => match self.direction {
                    Direction::Left | Direction::Right => self.enemy_hit_height = 0x400,
                    Direction::Up | Direction::Bottom => self.enemy_hit_width = 0x400,
                    Direction::FacingPlayer => unreachable!(),
                },
                38 => match self.direction {
                    Direction::Left | Direction::Right => {
                        self.enemy_hit_height = 0x800;
                    }
                    Direction::Up | Direction::Bottom => {
                        self.enemy_hit_width = 0x800;
                    }
                    Direction::FacingPlayer => unreachable!(),
                },
                39 => {
                    // level 3 uses default values
                }
                _ => {
                    unreachable!()
                }
            }
        } else {
            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        match self.btype {
            37 => {
                if self.direction == Direction::Up || self.direction == Direction::Bottom {
                    self.anim_num = 1;
                    self.anim_rect = state.constants.weapon.bullet_rects.b037_spur_l1[1];
                } else {
                    self.anim_num = 0;
                    self.anim_rect = state.constants.weapon.bullet_rects.b037_spur_l1[0];
                }

                new_bullets.push(Bullet::new(self.x, self.y, 40, self.owner, self.direction, &state.constants));
            }
            38 => {
                if self.direction == Direction::Up || self.direction == Direction::Bottom {
                    self.anim_num = 1;
                    self.anim_rect = state.constants.weapon.bullet_rects.b038_spur_l2[1];
                } else {
                    self.anim_num = 0;
                    self.anim_rect = state.constants.weapon.bullet_rects.b038_spur_l2[0];
                }

                new_bullets.push(Bullet::new(self.x, self.y, 41, self.owner, self.direction, &state.constants));
            }
            39 => {
                if self.direction == Direction::Up || self.direction == Direction::Bottom {
                    self.anim_num = 1;
                    self.anim_rect = state.constants.weapon.bullet_rects.b039_spur_l3[1];
                } else {
                    self.anim_num = 0;
                    self.anim_rect = state.constants.weapon.bullet_rects.b039_spur_l3[0];
                }

                new_bullets.push(Bullet::new(self.x, self.y, 42, self.owner, self.direction, &state.constants));
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn tick_spur_trail(&mut self, state: &mut SharedGameState) {
        self.action_counter += 1;
        if self.action_counter > 20 {
            self.anim_num = self.action_counter.wrapping_sub(20);

            if self.anim_num > 2 {
                self.cond.set_alive(false);
                return;
            }
        }

        if self.damage != 0 && self.life != 100 {
            self.damage = 0;
        }

        match self.btype {
            40 => {
                if self.direction == Direction::Up || self.direction == Direction::Bottom {
                    self.anim_rect = state.constants.weapon.bullet_rects.b040_spur_trail_l1[self.anim_num as usize + 3];
                } else {
                    self.anim_rect = state.constants.weapon.bullet_rects.b040_spur_trail_l1[self.anim_num as usize];
                }
            }
            41 => {
                if self.direction == Direction::Up || self.direction == Direction::Bottom {
                    self.anim_rect = state.constants.weapon.bullet_rects.b041_spur_trail_l2[self.anim_num as usize + 3];
                } else {
                    self.anim_rect = state.constants.weapon.bullet_rects.b041_spur_trail_l2[self.anim_num as usize];
                }
            }
            42 => {
                if self.direction == Direction::Up || self.direction == Direction::Bottom {
                    self.anim_rect = state.constants.weapon.bullet_rects.b042_spur_trail_l3[self.anim_num as usize + 3];
                } else {
                    self.anim_rect = state.constants.weapon.bullet_rects.b042_spur_trail_l3[self.anim_num as usize];
                }
            }
            _ => {
                unreachable!()
            }
        }
    }

    pub fn tick(
        &mut self,
        state: &mut SharedGameState,
        players: [&dyn PhysicalEntity; 2],
        npc_list: &NPCList,
        new_bullets: &mut Vec<Bullet>,
    ) {
        if self.life == 0 {
            self.cond.set_alive(false);
            return;
        }

        match self.btype {
            1 => self.tick_snake_1(state),
            2 | 3 => self.tick_snake_2(state, npc_list),
            4 | 5 | 6 => self.tick_polar_star(state),
            7 | 8 | 9 => self.tick_fireball(state, players, npc_list),
            10 | 11 | 12 => self.tick_machine_gun(state, npc_list),
            23 => self.tick_blade_slash(state),
            25 => self.tick_blade_1(state),
            26 => self.tick_blade_2(state),
            27 => self.tick_blade_3(state, new_bullets),
            37 | 38 | 39 => self.tick_spur(state, new_bullets),
            40 | 41 | 42 => self.tick_spur_trail(state),
            _ => self.cond.set_alive(false),
        }
    }

    pub fn vanish(&mut self, state: &mut SharedGameState) {
        match self.btype {
            37 | 38 | 39 => state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Up),
            _ => state.sound_manager.play_sfx(28),
        }

        self.cond.set_alive(false);
        state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Right);
    }

    fn judge_hit_block_destroy(&mut self, x: i32, y: i32, hit_attribs: &[u8; 4], state: &mut SharedGameState) {
        let mut hits = [false; 4];
        let block_x = (x * 16 + 8) * 0x200;
        let block_y = (y * 16 + 8) * 0x200;

        for (i, &attr) in hit_attribs.iter().enumerate() {
            if self.weapon_flags.flag_x40() {
                hits[i] = attr == 0x41 || attr == 0x61;
            } else {
                hits[i] = attr == 0x41 || attr == 0x43 || attr == 0x61;
            }
        }

        // left wall
        if hits[0] && hits[2] {
            if (self.x - self.hit_bounds.left as i32) < block_x {
                self.flags.set_hit_left_wall(true);
            }
        } else if hits[0] && !hits[2] {
            if (self.x - self.hit_bounds.left as i32) < block_x
                && (self.y - self.hit_bounds.top as i32) < block_y - (3 * 0x200)
            {
                self.flags.set_hit_left_wall(true);
            }
        } else if !hits[0]
            && hits[2]
            && (self.x - self.hit_bounds.left as i32) < block_x
            && (self.y + self.hit_bounds.top as i32) > block_y + (3 * 0x200)
        {
            self.flags.set_hit_left_wall(true);
        }

        // right wall
        if hits[1] && hits[3] {
            if (self.x + self.hit_bounds.right as i32) > block_x {
                self.flags.set_hit_right_wall(true);
            }
        } else if hits[1] && !hits[3] {
            if (self.x + self.hit_bounds.right as i32) > block_x
                && (self.y - self.hit_bounds.top as i32) < block_y - (3 * 0x200)
            {
                self.flags.set_hit_right_wall(true);
            }
        } else if !hits[1]
            && hits[3]
            && (self.x + self.hit_bounds.right as i32) > block_x
            && (self.y + self.hit_bounds.top as i32) > block_y + (3 * 0x200)
        {
            self.flags.set_hit_right_wall(true);
        }

        // ceiling
        if hits[0] && hits[1] {
            if (self.y - self.hit_bounds.top as i32) < block_y {
                self.flags.set_hit_top_wall(true);
            }
        } else if hits[0] && !hits[1] {
            if (self.x - self.hit_bounds.left as i32) < block_x - (3 * 0x200)
                && (self.y - self.hit_bounds.top as i32) < block_y
            {
                self.flags.set_hit_top_wall(true);
            }
        } else if !hits[0]
            && hits[1]
            && (self.x + self.hit_bounds.right as i32) > block_x + (3 * 0x200)
            && (self.y - self.hit_bounds.top as i32) < block_y
        {
            self.flags.set_hit_top_wall(true);
        }

        // ground
        if hits[2] && hits[3] {
            if (self.y + self.hit_bounds.bottom as i32) > block_y {
                self.flags.set_hit_bottom_wall(true);
            }
        } else if hits[2] && !hits[3] {
            if (self.x - self.hit_bounds.left as i32) < block_x - (3 * 0x200)
                && (self.y + self.hit_bounds.bottom as i32) > block_y
            {
                self.flags.set_hit_bottom_wall(true);
            }
        } else if !hits[2]
            && hits[3]
            && (self.x + self.hit_bounds.right as i32) > block_x + (3 * 0x200)
            && (self.y + self.hit_bounds.bottom as i32) > block_y
        {
            self.flags.set_hit_bottom_wall(true);
        }

        if self.weapon_flags.flag_x08() {
            if self.flags.hit_left_wall() {
                self.x = block_x + self.hit_bounds.right as i32;
            } else if self.flags.hit_right_wall() {
                self.x = block_x - self.hit_bounds.left as i32;
            } else if self.flags.hit_top_wall() {
                self.y = block_y + self.hit_bounds.bottom as i32;
            } else if self.flags.hit_bottom_wall() {
                self.y = block_y - self.hit_bounds.top as i32;
            }
        } else if self.flags.hit_left_wall()
            || self.flags.hit_top_wall()
            || self.flags.hit_right_wall()
            || self.flags.hit_bottom_wall()
        {
            self.vanish(state);
        }
    }
}

impl PhysicalEntity for Bullet {
    #[inline(always)]
    fn x(&self) -> i32 {
        self.x
    }

    #[inline(always)]
    fn y(&self) -> i32 {
        self.y
    }

    #[inline(always)]
    fn vel_x(&self) -> i32 {
        self.vel_x
    }

    #[inline(always)]
    fn vel_y(&self) -> i32 {
        self.vel_y
    }

    fn hit_rect_size(&self) -> usize {
        2
    }

    #[inline(always)]
    fn hit_bounds(&self) -> &Rect<usize> {
        &self.hit_bounds
    }

    #[inline(always)]
    fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    #[inline(always)]
    fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    #[inline(always)]
    fn set_vel_x(&mut self, vel_x: i32) {
        self.vel_x = vel_x;
    }

    #[inline(always)]
    fn set_vel_y(&mut self, vel_y: i32) {
        self.vel_y = vel_y;
    }

    #[inline(always)]
    fn cond(&mut self) -> &mut Condition {
        &mut self.cond
    }

    #[inline(always)]
    fn flags(&mut self) -> &mut Flag {
        &mut self.flags
    }

    #[inline(always)]
    fn direction(&self) -> Direction {
        self.direction
    }

    #[inline(always)]
    fn is_player(&self) -> bool {
        false
    }

    fn judge_hit_block(&mut self, _state: &mut SharedGameState, x: i32, y: i32) {
        if (self.x - self.hit_bounds.left as i32) < (x * 16 + 8) * 0x200
            && (self.x + self.hit_bounds.right as i32) > (x * 16 - 8) * 0x200
            && (self.y - self.hit_bounds.top as i32) < (y * 16 + 8) * 0x200
            && (self.y + self.hit_bounds.bottom as i32) > (y * 16 - 8) * 0x200
        {
            self.flags.set_weapon_hit_block(true);
        }
    }

    fn tick_map_collisions(&mut self, state: &mut SharedGameState, npc_list: &NPCList, stage: &mut Stage) {
        self.flags().0 = 0;
        if self.weapon_flags.flag_x04() {
            // ???
            return;
        }

        let x = clamp(self.x() / 16 / 0x200, 0, stage.map.width as i32);
        let y = clamp(self.y() / 16 / 0x200, 0, stage.map.height as i32);
        let mut hit_attribs = [0u8; 4];

        for (idx, (&ox, &oy)) in OFF_X.iter().zip(OFF_Y.iter()).enumerate() {
            if idx == 4 || !self.cond.alive() {
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
                    let old_hit = self.flags;
                    self.flags.0 = 0;
                    self.judge_hit_block(state, x + ox, y + oy);

                    if self.flags.weapon_hit_block() && (self.weapon_flags.flag_x20() || self.weapon_flags.flag_x40()) {
                        if !self.weapon_flags.flag_x40() {
                            self.cond.set_alive(false);
                        }

                        state.create_caret(self.x, self.y, CaretType::ProjectileDissipation, Direction::Left);
                        state.sound_manager.play_sfx(12);

                        let mut npc = NPC::create(4, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.direction = Direction::Left;
                        npc.x = (x * 16 + 8) * 0x200;
                        npc.y = (y * 16 + 8) * 0x200;

                        for _ in 0..4 {
                            npc.vel_x = state.game_rng.range(-0x200..0x200) as i32;
                            npc.vel_y = state.game_rng.range(-0x200..0x200) as i32;

                            let _ = npc_list.spawn(0x100, npc.clone());
                        }

                        if let Some(tile) =
                            stage.map.tiles.get_mut(stage.map.width as usize * (y + oy) as usize + (x + ox) as usize)
                        {
                            *tile = tile.wrapping_sub(1);
                        }
                    }

                    self.flags.0 |= old_hit.0;
                }
                // Slopes
                0x50 | 0x70 => {
                    self.judge_hit_triangle_a(state, x + ox, y + oy);
                }
                0x51 | 0x71 => {
                    self.judge_hit_triangle_b(state, x + ox, y + oy);
                }
                0x52 | 0x72 => {
                    self.judge_hit_triangle_c(state, x + ox, y + oy);
                }
                0x53 | 0x73 => {
                    self.judge_hit_triangle_d(state, x + ox, y + oy);
                }
                0x54 | 0x74 => {
                    self.judge_hit_triangle_e(state, x + ox, y + oy);
                }
                0x55 | 0x75 => {
                    self.judge_hit_triangle_f(state, x + ox, y + oy);
                }
                0x56 | 0x76 => {
                    self.judge_hit_triangle_g(state, x + ox, y + oy);
                }
                0x57 | 0x77 => {
                    self.judge_hit_triangle_h(state, x + ox, y + oy);
                }
                _ => {}
            }
        }

        self.judge_hit_block_destroy(x, y, &hit_attribs, state);
    }
}
