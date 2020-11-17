use num_traits::{abs, clamp};
use num_traits::real::Real;

use crate::caret::CaretType;
use crate::common::Direction;
use ggez::GameResult;
use crate::npc::{NPC, NPCMap};
use crate::player::Player;
use crate::shared_game_state::SharedGameState;
use crate::stage::Stage;

impl NPC {
    pub(crate) fn tick_n000_null(&mut self) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            if self.direction == Direction::Right {
                self.y += 16 * 0x200;
            }
        }

        self.anim_rect.left = 0;
        self.anim_rect.top = 0;
        self.anim_rect.right = 16;
        self.anim_rect.bottom = 16;

        Ok(())
    }

    pub(crate) fn tick_n003_dead_enemy(&mut self) -> GameResult {
        if self.action_num != 0xffff {
            self.action_num = 0xffff;
            self.action_counter2 = 0;
            self.anim_rect.left = 0;
            self.anim_rect.top = 0;
            self.anim_rect.right = 0;
            self.anim_rect.bottom = 0;
        }

        self.action_counter2 += 1;
        if self.action_counter2 == 100 {
            self.cond.set_alive(false);
        }

        Ok(())
    }

    pub(crate) fn tick_n004_smoke(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = state.game_rng.range(0..4) as u16;
            self.anim_counter = state.game_rng.range(0..3) as u16;

            if self.direction == Direction::Left || self.direction == Direction::Up {
                let angle = state.game_rng.range(0..31415) as f32 / 5000.0;
                self.vel_x = (angle.cos() * state.game_rng.range(0x200..0x5ff) as f32) as isize;
                self.vel_y = (angle.sin() * state.game_rng.range(0x200..0x5ff) as f32) as isize;
            }
        } else {
            self.vel_x = (self.vel_x * 20) / 21;
            self.vel_y = (self.vel_y * 20) / 21;

            self.x += self.vel_x;
            self.y += self.vel_y;
        }

        self.anim_counter += 1;
        if self.anim_counter > 4 {
            self.anim_counter = 0;
            self.anim_num += 1;

            if self.anim_num > 7 {
                self.cond.set_alive(false);
            }
        }

        match self.direction {
            Direction::Left | Direction::Right => {
                self.anim_rect = state.constants.npc.n004_smoke[self.anim_num as usize];
            }
            Direction::Up => {
                self.anim_rect = state.constants.npc.n004_smoke[self.anim_num as usize + 8];
            }
            _ => {}
        }

        Ok(())
    }

    pub(crate) fn tick_n013_forcefield(&mut self, state: &mut SharedGameState) -> GameResult {
        self.anim_counter = (self.anim_counter + 1) % 2;
        if self.anim_counter == 1 {
            self.anim_num = (self.anim_num + 1) % 4;
            self.anim_rect = state.constants.npc.n013_forcefield[self.anim_num as usize];
        }

        Ok(())
    }

    pub(crate) fn tick_n014_key(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            if self.direction == Direction::Right {
                self.vel_y = -0x200;
            }
        }

        if self.flags.hit_bottom_wall() {
            self.npc_flags.set_interactable(true);
        }

        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num += 1;
            if self.anim_num > 2 {
                self.anim_num = 0
            }
        }

        self.vel_y += 0x40;

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n014_key[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n015_chest_closed(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.npc_flags.set_interactable(true);

                    if self.direction == Direction::Right {
                        self.vel_y = -0x200;

                        let mut npc = NPCMap::create_npc(4, &state.npc_table);

                        for _ in 0..4 {
                            npc.cond.set_alive(true);
                            npc.direction = Direction::Left;
                            npc.x = self.x + state.game_rng.range(-12..12) as isize * 0x200;
                            npc.y = self.y + state.game_rng.range(-12..12) as isize * 0x200;
                            npc.vel_x = state.game_rng.range(-0x155..0x155) as isize;
                            npc.vel_y = state.game_rng.range(-0x600..0) as isize;

                            state.new_npcs.push(npc);
                        }
                    }

                    self.anim_rect = state.constants.npc.n015_closed_chest[0];
                }

                self.anim_num = 0;
                if state.game_rng.range(0..30) == 0 {
                    self.action_num = 2;
                }
            }
            2 => {
                if self.anim_counter == 0 {
                    self.anim_rect = state.constants.npc.n015_closed_chest[self.anim_num as usize];
                }

                self.anim_counter += 1;
                if self.anim_counter > 1 {
                    self.anim_counter = 0;
                    self.anim_num += 1;

                    if self.anim_num > 2 {
                        self.anim_num = 0;
                        self.action_num = 1;
                    }
                }
            }
            _ => {}
        }

        self.vel_y += 0x40;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n016_save_point(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            if self.direction == Direction::Right {
                self.npc_flags.set_interactable(false);
                self.vel_y = -0x200;
            }
        }

        if self.flags.hit_bottom_wall() {
            self.npc_flags.set_interactable(true);
        }

        self.anim_counter = (self.anim_counter + 1) % 24;
        self.anim_num = self.anim_counter / 3;
        self.anim_rect = state.constants.npc.n016_save_point[self.anim_num as usize];

        self.vel_y += 0x40;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n017_health_refill(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
        }

        match self.action_num {
            1 => {
                let rand = state.game_rng.range(0..30);

                if rand < 10 {
                    self.action_num = 2;
                } else if rand < 25 {
                    self.action_num = 3;
                } else {
                    self.action_num = 4;
                }

                self.action_counter = state.game_rng.range(0x10..0x40) as u16;
                self.anim_counter = 0;
            }
            2 => {
                self.anim_rect = state.constants.npc.n017_health_refill[0];
                self.anim_num = 0;

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 1;
                }
            }
            3 => {
                self.anim_counter += 1;

                if self.anim_counter % 2 == 0 {
                    self.anim_num = 1;
                    self.anim_rect = state.constants.npc.n017_health_refill[1];
                } else {
                    self.anim_num = 0;
                    self.anim_rect = state.constants.npc.n017_health_refill[0];
                }

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 1;
                }
            }
            4 => {
                self.anim_num = 1;
                self.anim_rect = state.constants.npc.n017_health_refill[1];

                if self.action_counter > 0 {
                    self.action_counter -= 1;
                } else {
                    self.action_num = 1;
                }
            }
            _ => {}
        }

        self.vel_y += 0x40;
        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n018_door(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                match self.direction {
                    Direction::Left => { self.anim_rect = state.constants.npc.n018_door[0] }
                    Direction::Right => { self.anim_rect = state.constants.npc.n018_door[1] }
                    _ => {}
                }
            }
            1 => {
                for _ in 0..4 {
                    let mut npc = NPCMap::create_npc(4, &state.npc_table);

                    npc.cond.set_alive(true);
                    npc.direction = Direction::Left;
                    npc.x = self.x;
                    npc.y = self.y;
                    npc.vel_x = state.game_rng.range(-0x155..0x155) as isize;
                    npc.vel_y = state.game_rng.range(-0x600..0) as isize;

                    state.new_npcs.push(npc);
                }

                self.action_num = 0;
                self.anim_rect = state.constants.npc.n018_door[0]
            }
            _ => {}
        }

        Ok(())
    }

    pub(crate) fn tick_n020_computer(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.direction {
            Direction::Left if self.anim_num == 0 => {
                self.anim_num = 1;
                self.anim_rect = state.constants.npc.n020_computer[0];
            }
            Direction::Right => {
                self.anim_counter = (self.anim_counter + 1) % 12;
                self.anim_num = self.anim_counter / 4;
                self.anim_rect = state.constants.npc.n020_computer[1 + self.anim_num as usize];
            }
            _ => {}
        }

        Ok(())
    }

    pub(crate) fn tick_n021_chest_open(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            if self.direction == Direction::Right {
                self.y += 16 * 0x200;
            }

            self.anim_rect = state.constants.npc.n021_chest_open;
        }

        Ok(())
    }

    pub(crate) fn tick_n022_teleporter(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 if self.anim_counter == 0 => {
                self.anim_counter = 1;
                self.anim_rect = state.constants.npc.n022_teleporter[0];
            }
            1 => {
                self.anim_num = (self.anim_num + 1) & 1;
                self.anim_rect = state.constants.npc.n022_teleporter[self.anim_num as usize];
            }
            _ => {}
        }

        Ok(())
    }

    pub(crate) fn tick_n023_teleporter_lights(&mut self, state: &mut SharedGameState) -> GameResult {
        self.anim_counter += 1;
        if self.anim_counter > 1 {
            self.anim_counter = 0;
            self.anim_num += 1;
            if self.anim_num > 7 {
                self.anim_num = 0;
            }
        } else if self.anim_counter == 1 {
            self.anim_rect = state.constants.npc.n023_teleporter_lights[self.anim_num as usize];
        }

        Ok(())
    }

    pub(crate) fn tick_n027_death_trap(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_rect = state.constants.npc.n027_death_trap;
        }

        Ok(())
    }

    pub(crate) fn tick_n030_gunsmith(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.direction == Direction::Left {
            match self.action_num {
                0 => {
                    self.action_num = 1;
                    self.anim_counter = 0;
                    self.anim_rect = state.constants.npc.n030_hermit_gunsmith[0];
                }
                1 => {
                    self.action_num = 1;
                    self.anim_counter = 0;

                    if state.game_rng.range(0..120) == 10 {
                        self.action_num = 2;
                        self.action_counter = 8;
                        self.anim_rect = state.constants.npc.n030_hermit_gunsmith[1];
                    }
                }
                2 => {
                    if self.action_counter > 0 {
                        self.action_counter -= 1;
                        self.anim_rect = state.constants.npc.n030_hermit_gunsmith[0];
                    }
                }
                _ => {}
            }
        } else {
            if self.action_num == 0 {
                self.action_num = 1;
                self.anim_rect = state.constants.npc.n030_hermit_gunsmith[2];
                self.y += 16 * 0x200;
            }

            self.action_counter += 1;
            if self.action_counter > 100 {
                self.action_counter = 0;
                state.create_caret(self.x, self.y - 0x400, CaretType::Zzz, Direction::Left);
            }
        }

        Ok(())
    }


    pub(crate) fn tick_n032_life_capsule(&mut self, state: &mut SharedGameState) -> GameResult {
        self.anim_counter = (self.anim_counter + 1) % 4;
        self.anim_num = self.anim_counter / 2;
        self.anim_rect = state.constants.npc.n032_life_capsule[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n034_bed(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            match self.direction {
                Direction::Left => { self.anim_rect = state.constants.npc.n034_bed[0] }
                Direction::Right => { self.anim_rect = state.constants.npc.n034_bed[1] }
                _ => {}
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n037_sign(&mut self, state: &mut SharedGameState) -> GameResult {
        self.anim_counter = (self.anim_counter + 1) % 4;
        self.anim_num = self.anim_counter / 2;
        self.anim_rect = state.constants.npc.n037_sign[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n038_fireplace(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                self.anim_counter = (self.anim_counter + 1) % 16;
                self.anim_num = self.anim_counter / 4;
                self.anim_rect = state.constants.npc.n038_fireplace[self.anim_num as usize];
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                }

                self.anim_rect.left = 0;
                self.anim_rect.right = 0;
            }
            _ => {}
        }

        Ok(())
    }

    pub(crate) fn tick_n039_save_sign(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            match self.direction {
                Direction::Left => { self.anim_rect = state.constants.npc.n039_save_sign[0] }
                Direction::Right => { self.anim_rect = state.constants.npc.n039_save_sign[1] }
                _ => {}
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n041_busted_door(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_rect = state.constants.npc.n041_busted_door;
            self.y -= 16 * 0x200;
        }

        Ok(())
    }

    pub(crate) fn tick_n043_chalkboard(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.y -= 16 * 0x200;

            match self.direction {
                Direction::Left => { self.anim_rect = state.constants.npc.n043_chalkboard[0] }
                Direction::Right => { self.anim_rect = state.constants.npc.n043_chalkboard[1] }
                _ => {}
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n046_hv_trigger(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        self.npc_flags.set_event_when_touched(true);

        if self.direction == Direction::Left {
            if self.x < player.x {
                self.x += 0x5ff;
            } else {
                self.x -= 0x5ff;
            }
        } else if self.y < player.y {
            self.y += 0x5ff;
        } else {
            self.y -= 0x5ff;
        }

        Ok(())
    }

    pub(crate) fn tick_n070_sparkle(&mut self, state: &mut SharedGameState) -> GameResult {
        self.anim_counter = (self.anim_counter + 1) % 16;
        self.anim_num = self.anim_counter / 4;
        self.anim_rect = state.constants.npc.n070_sparkle[self.anim_num as usize];

        Ok(())
    }


    pub(crate) fn tick_n072_sprinkler(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        if self.direction == Direction::Left {
            self.anim_counter = (self.anim_counter + 1) % 4;
            self.anim_num = self.anim_counter / 2;
            self.anim_rect = state.constants.npc.n072_sprinkler[self.anim_num as usize];

            if self.anim_num % 2 == 0 && (player.x - self.x).abs() < 480 * 0x200 {
                self.action_counter = self.action_counter.wrapping_add(1);

                let mut droplet = NPCMap::create_npc(73, &state.npc_table);
                droplet.cond.set_alive(true);
                droplet.direction = Direction::Left;
                droplet.x = self.x;
                droplet.y = self.y;
                droplet.vel_x = 2 * state.game_rng.range(-0x200..0x200) as isize;
                droplet.vel_y = 3 * state.game_rng.range(-0x200..0x80) as isize;
                state.new_npcs.push(droplet);

                if self.action_counter % 2 == 0 {
                    droplet.vel_x = 2 * state.game_rng.range(-0x200..0x200) as isize;
                    droplet.vel_y = 3 * state.game_rng.range(-0x200..0x80) as isize;
                    state.new_npcs.push(droplet);
                }
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n073_water_droplet(&mut self, state: &mut SharedGameState, stage: &Stage) -> GameResult {
        self.vel_y += 0x20;

        self.anim_rect = state.constants.npc.n073_water_droplet[state.game_rng.range(0..4) as usize];

        if self.vel_y > 0x5ff {
            self.vel_y = 0x5ff;
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.direction == Direction::Right {
            self.anim_rect.top += 2;
            self.anim_rect.bottom += 2;
        }

        self.action_counter += 1;
        if self.action_counter > 10 && (self.flags.hit_left_wall() || self.flags.hit_right_wall()
            || self.flags.hit_bottom_wall() || self.flags.in_water()) {
            // hit something
            self.cond.set_alive(false);
        }

        if self.y > stage.map.height as isize * 16 * 0x200 {
            // out of map
            self.cond.set_alive(false);
        }

        Ok(())
    }

    pub(crate) fn tick_n076_flowers(&mut self) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            self.anim_rect.left = self.event_num as u16 * 16;
            self.anim_rect.top = 0;
            self.anim_rect.right = self.anim_rect.left + 16;
            self.anim_rect.bottom = self.anim_rect.top + 16;
        }

        Ok(())
    }

    pub(crate) fn tick_n078_pot(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            match self.direction {
                Direction::Left => { self.anim_rect = state.constants.npc.n078_pot[0] }
                Direction::Right => { self.anim_rect = state.constants.npc.n078_pot[1] }
                _ => {}
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n085_terminal(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        match self.action_num {
            0 => {
                self.anim_num = 0;
                if abs(player.x - self.x) < 8 * 0x200 && player.y < self.y + 8 * 0x200 && player.y > self.y - 16 * 0x200 {
                    state.sound_manager.play_sfx(43);
                    self.action_num = 1;
                }
            }
            1 => {
                self.anim_num += 1;
                if self.anim_num > 2 {
                    self.anim_num = 1;
                }
            }
            _ =>{ }
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };
        self.anim_rect = state.constants.npc.n085_terminal[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n096_fan_left(&mut self, state: &mut SharedGameState, player: &mut Player) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 && self.direction == Direction::Right {
                    self.action_num = 2;
                }

                self.anim_num = 1;
            }
            2 => {
                self.anim_counter += 1;
                if self.anim_counter > 0 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 2 {
                        self.anim_num = 0;
                    }
                }

                if abs(player.x - self.x) < 480 * 0x200 && abs(player.y - self.y) < 240 * 0x200
                    && state.game_rng.range(0..5) == 1 {
                    let mut particle = NPCMap::create_npc(199, &state.npc_table);
                    particle.cond.set_alive(true);
                    particle.direction = Direction::Left;
                    particle.x = self.x;
                    particle.y = self.y + (state.game_rng.range(-8..8) * 0x200) as isize;
                    state.new_npcs.push(particle);
                }

                if abs(player.y - self.y) < 8 * 0x200 && player.x < self.x && player.x > self.x - 96 * 0x200 {
                    player.vel_x -= 0x88;
                    player.cond.set_increase_acceleration(true);
                }
            }
            _ => {}
        }

        if self.anim_counter == 0 {
            self.anim_rect = state.constants.npc.n098_fan_right[self.anim_num as usize];
        }

        Ok(())
    }

    pub(crate) fn tick_n097_fan_up(&mut self, state: &mut SharedGameState, player: &mut Player) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 && self.direction == Direction::Right {
                    self.action_num = 2;
                }

                self.anim_num = 1;
            }
            2 => {
                self.anim_counter += 1;
                if self.anim_counter > 0 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 2 {
                        self.anim_num = 0;
                    }
                }

                if abs(player.x - self.x) < 480 * 0x200 && abs(player.y - self.y) < 240 * 0x200
                    && state.game_rng.range(0..5) == 1 {
                    let mut particle = NPCMap::create_npc(199, &state.npc_table);
                    particle.cond.set_alive(true);
                    particle.direction = Direction::Up;
                    particle.x = self.x + (state.game_rng.range(-8..8) * 0x200) as isize;
                    particle.y = self.y;
                    state.new_npcs.push(particle);
                }

                if abs(player.x - self.x) < 8 * 0x200 && player.y < self.y && player.y > self.y - 96 * 0x200 {
                    player.vel_y -= 0x88;
                }
            }
            _ => {}
        }

        if self.anim_counter == 0 {
            self.anim_rect = state.constants.npc.n097_fan_up[self.anim_num as usize];
        }

        Ok(())
    }

    pub(crate) fn tick_n098_fan_right(&mut self, state: &mut SharedGameState, player: &mut Player) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 && self.direction == Direction::Right {
                    self.action_num = 2;
                }

                self.anim_num = 1;
            }
            2 => {
                self.anim_counter += 1;
                if self.anim_counter > 0 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 2 {
                        self.anim_num = 0;
                    }
                }

                if abs(player.x - self.x) < 480 * 0x200 && abs(player.y - self.y) < 240 * 0x200
                    && state.game_rng.range(0..5) == 1 {
                    let mut particle = NPCMap::create_npc(199, &state.npc_table);
                    particle.cond.set_alive(true);
                    particle.direction = Direction::Right;
                    particle.x = self.x;
                    particle.y = self.y + (state.game_rng.range(-8..8) * 0x200) as isize;
                    state.new_npcs.push(particle);
                }

                if abs(player.y - self.y) < 8 * 0x200 && player.x > self.x && player.x < self.x + 96 * 0x200 {
                    player.vel_x += 0x88;
                    player.cond.set_increase_acceleration(true);
                }
            }
            _ => {}
        }

        if self.anim_counter == 0 {
            self.anim_rect = state.constants.npc.n098_fan_right[self.anim_num as usize];
        }

        Ok(())
    }

    pub(crate) fn tick_n099_fan_down(&mut self, state: &mut SharedGameState, player: &mut Player) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 && self.direction == Direction::Right {
                    self.action_num = 2;
                }

                self.anim_num = 1;
            }
            2 => {
                self.anim_counter += 1;
                if self.anim_counter > 0 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                    if self.anim_num > 2 {
                        self.anim_num = 0;
                    }
                }

                if abs(player.x - self.x) < 480 * 0x200 && abs(player.y - self.y) < 240 * 0x200
                    && state.game_rng.range(0..5) == 1 {
                    let mut particle = NPCMap::create_npc(199, &state.npc_table);
                    particle.cond.set_alive(true);
                    particle.direction = Direction::Bottom;
                    particle.x = self.x + (state.game_rng.range(-8..8) * 0x200) as isize;
                    particle.y = self.y;
                    state.new_npcs.push(particle);
                }

                if abs(player.x - self.x) < 8 * 0x200 && player.y > self.y && player.y < self.y + 96 * 0x200 {
                    player.vel_y -= 0x88;
                }
            }
            _ => {}
        }

        if self.anim_counter == 0 {
            self.anim_rect = state.constants.npc.n097_fan_up[self.anim_num as usize];
        }

        Ok(())
    }

    pub(crate) fn tick_n149_horizontal_moving_block(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        match self.action_num {
            0 => {
                self.x += 8 * 0x200;
                self.y += 8 * 0x200;
                self.npc_flags.set_solid_hard(true);
                self.vel_x = 0;
                self.vel_y = 0;
                self.action_num = if self.direction == Direction::Right { 20 } else { 10 };
            }
            10 => {
                self.npc_flags.set_rear_and_top_not_hurt(false);
                self.damage = 0;
                if (player.x < self.x + 25 * 0x200) && (player.x > self.x - 25 * 16 * 0x200)
                    && (player.y < self.y + 25 * 0x200) && (player.y > self.y - 25 * 0x200) {
                    self.action_num = 11;
                    self.action_counter = 0;
                }
            }
            11 => {
                self.action_counter += 1;
                if self.action_counter % 10 == 6 {
                    state.sound_manager.play_sfx(107);
                }

                if self.flags.hit_left_wall() {
                    self.vel_x = 0;
                    self.direction = Direction::Right;
                    self.action_num = 20;

                    state.quake_counter = 10;
                    state.sound_manager.play_sfx(26);

                    let mut npc = NPCMap::create_npc(4, &state.npc_table);
                    for _ in 0..3 {
                        npc.cond.set_alive(true);
                        npc.direction = Direction::Left;
                        npc.x = self.x + state.game_rng.range(-12..12) as isize * 0x200;
                        npc.y = self.y + state.game_rng.range(-12..12) as isize * 0x200;
                        npc.vel_x = state.game_rng.range(-0x155..0x155) as isize;
                        npc.vel_y = state.game_rng.range(-0x600..0) as isize;

                        state.new_npcs.push(npc);
                    }
                } else {
                    if player.flags.hit_left_wall() {
                        self.npc_flags.set_rear_and_top_not_hurt(true);
                        self.damage = 100;
                    } else {
                        self.npc_flags.set_rear_and_top_not_hurt(false);
                        self.damage = 0;
                    }

                    self.vel_x -= 0x20;
                }
            }
            20 => {
                self.npc_flags.set_rear_and_top_not_hurt(false);
                self.damage = 0;

                if (player.x > self.x - 25 * 0x200) && (player.x < self.x + 25 * 16 * 0x200)
                    && (player.y < self.y + 25 * 0x200) && (player.y > self.y - 25 * 0x200) {
                    self.action_num = 21;
                    self.action_counter = 0;
                }
            }
            21 => {
                self.action_counter += 1;
                if self.action_counter % 10 == 6 {
                    state.sound_manager.play_sfx(107);
                }

                if self.flags.hit_right_wall() {
                    self.vel_x = 0;
                    self.direction = Direction::Left;
                    self.action_num = 10;

                    state.quake_counter = 10;
                    state.sound_manager.play_sfx(26);

                    let mut npc = NPCMap::create_npc(4, &state.npc_table);
                    for _ in 0..3 {
                        npc.cond.set_alive(true);
                        npc.direction = Direction::Left;
                        npc.x = self.x + state.game_rng.range(-12..12) as isize * 0x200;
                        npc.y = self.y + state.game_rng.range(-12..12) as isize * 0x200;
                        npc.vel_x = state.game_rng.range(-0x155..0x155) as isize;
                        npc.vel_y = state.game_rng.range(-0x600..0) as isize;

                        state.new_npcs.push(npc);
                    }
                } else {
                    if player.flags.hit_right_wall() {
                        self.npc_flags.set_rear_and_top_not_hurt(true);
                        self.damage = 100;
                    } else {
                        self.npc_flags.set_rear_and_top_not_hurt(false);
                        self.damage = 0;
                    }

                    self.vel_x += 0x20;
                }
            }
            _ => {}
        }

        self.vel_x = clamp(self.vel_x, -0x200, 0x200);
        self.x += self.vel_x;

        if self.anim_num != 149 {
            self.anim_num = 149;
            self.anim_rect = state.constants.npc.n149_horizontal_moving_block;
        }

        Ok(())
    }

    pub(crate) fn tick_n157_vertical_moving_block(&mut self, state: &mut SharedGameState, player: &Player) -> GameResult {
        match self.action_num {
            0 => {
                self.x += 8 * 0x200;
                self.y += 8 * 0x200;
                self.npc_flags.set_solid_hard(true);
                self.vel_x = 0;
                self.vel_y = 0;
                self.action_num = if self.direction == Direction::Right { 20 } else { 10 };
            }
            10 => {
                self.npc_flags.set_rear_and_top_not_hurt(false);
                self.damage = 0;
                if (player.y < self.y + 25 * 0x200) && (player.y > self.y - 25 * 16 * 0x200)
                    && (player.x < self.x + 25 * 0x200) && (player.x > self.x - 25 * 0x200) {
                    self.action_num = 11;
                    self.action_counter = 0;
                }
            }
            11 => {
                self.action_counter += 1;
                if self.action_counter % 10 == 6 {
                    state.sound_manager.play_sfx(107);
                }

                if self.flags.hit_top_wall() {
                    self.vel_y = 0;
                    self.direction = Direction::Right;
                    self.action_num = 20;

                    state.quake_counter = 10;
                    state.sound_manager.play_sfx(26);

                    let mut npc = NPCMap::create_npc(4, &state.npc_table);
                    for _ in 0..3 {
                        npc.cond.set_alive(true);
                        npc.direction = Direction::Left;
                        npc.x = self.x + state.game_rng.range(-12..12) as isize * 0x200;
                        npc.y = self.y + state.game_rng.range(-12..12) as isize * 0x200;
                        npc.vel_x = state.game_rng.range(-0x155..0x155) as isize;
                        npc.vel_y = state.game_rng.range(-0x600..0) as isize;

                        state.new_npcs.push(npc);
                    }
                } else {
                    if player.flags.hit_top_wall() {
                        self.npc_flags.set_rear_and_top_not_hurt(true);
                        self.damage = 100;
                    } else {
                        self.npc_flags.set_rear_and_top_not_hurt(false);
                        self.damage = 0;
                    }

                    self.vel_y -= 0x20;
                }
            }
            20 => {
                self.npc_flags.set_rear_and_top_not_hurt(false);
                self.damage = 0;

                if (player.y > self.y - 25 * 0x200) && (player.y < self.y + 25 * 16 * 0x200)
                    && (player.x < self.x + 25 * 0x200) && (player.x > self.x - 25 * 0x200) {
                    self.action_num = 21;
                    self.action_counter = 0;
                }
            }
            21 => {
                self.action_counter += 1;
                if self.action_counter % 10 == 6 {
                    state.sound_manager.play_sfx(107);
                }

                if self.flags.hit_bottom_wall() {
                    self.vel_x = 0;
                    self.direction = Direction::Left;
                    self.action_num = 10;

                    state.quake_counter = 10;
                    state.sound_manager.play_sfx(26);

                    let mut npc = NPCMap::create_npc(4, &state.npc_table);
                    for _ in 0..3 {
                        npc.cond.set_alive(true);
                        npc.direction = Direction::Left;
                        npc.x = self.x + state.game_rng.range(-12..12) as isize * 0x200;
                        npc.y = self.y + state.game_rng.range(-12..12) as isize * 0x200;
                        npc.vel_x = state.game_rng.range(-0x155..0x155) as isize;
                        npc.vel_y = state.game_rng.range(-0x600..0) as isize;

                        state.new_npcs.push(npc);
                    }
                } else {
                    if player.flags.hit_bottom_wall() {
                        self.npc_flags.set_rear_and_top_not_hurt(true);
                        self.damage = 100;
                    } else {
                        self.npc_flags.set_rear_and_top_not_hurt(false);
                        self.damage = 0;
                    }

                    self.vel_y += 0x20;
                }
            }
            _ => {}
        }

        self.vel_y = clamp(self.vel_y, -0x200, 0x200);
        self.y += self.vel_y;

        if self.anim_num != 149 {
            self.anim_num = 149;
            self.anim_rect = state.constants.npc.n149_horizontal_moving_block;
        }

        Ok(())
    }

    pub(crate) fn tick_n199_wind_particles(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = state.game_rng.range(0..2) as u16;
            self.vel_x = self.direction.vector_x() * (state.game_rng.range(4..8) * 0x200 / 2) as isize;
            self.vel_y = self.direction.vector_y() * (state.game_rng.range(4..8) * 0x200 / 2) as isize;
        }

        self.anim_counter += 1;
        if self.anim_counter > 6 {
            self.anim_counter = 0;
            self.anim_num += 1;
            if self.anim_num > 4 {
                self.cond.set_alive(false);
                return Ok(());
            }
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.anim_counter == 1 {
            self.anim_rect = state.constants.npc.n199_wind_particles[self.anim_num as usize];
        }

        Ok(())
    }

    pub(crate) fn tick_n211_small_spikes(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_rect = state.constants.npc.n211_small_spikes[self.event_num as usize % 4];
        }

        Ok(())
    }
}
