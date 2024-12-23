use std::hint::unreachable_unchecked;

use crate::common::{Direction, Rect};
use crate::components::flash::Flash;
use crate::framework::error::GameResult;
use crate::game::caret::CaretType;
use crate::game::npc::{NPC, NPCLayer};
use crate::game::npc::boss::BossNPC;
use crate::game::npc::list::NPCList;
use crate::game::player::Player;
use crate::game::shared_game_state::{GameDifficulty, SharedGameState};
use crate::game::stage::Stage;
use crate::util::rng::RNG;

impl NPC {
    pub(crate) fn tick_n000_null(&mut self) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            if self.direction == Direction::Right {
                self.y += 0x2000;
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
            self.anim_num = self.rng.range(0..4) as u16;
            self.anim_counter = self.rng.range(0..3) as u16;

            if self.direction == Direction::Left || self.direction == Direction::Up {
                let angle = self.rng.range(0..31415) as f32 / 5000.0;
                self.vel_x = (angle.cos() * self.rng.range(0x200..0x5ff) as f32) as i32;
                self.vel_y = (angle.sin() * self.rng.range(0x200..0x5ff) as f32) as i32;
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
                return Ok(());
            }
        }

        match self.direction {
            Direction::Left | Direction::Right => {
                self.anim_rect = state.constants.npc.n004_smoke[self.anim_num as usize];
            }
            Direction::Up => {
                self.anim_rect = state.constants.npc.n004_smoke[self.anim_num as usize + 8];
            }
            _ => (),
        }

        Ok(())
    }

    pub(crate) fn tick_n013_forcefield(&mut self, state: &mut SharedGameState) -> GameResult {
        self.anim_num = (self.anim_num + 1) % 4;
        self.anim_rect = state.constants.npc.n013_forcefield[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n014_key(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            if self.direction == Direction::Right {
                self.vel_y = -0x200;

                let mut npc = NPC::create(4, &state.npc_table);
                npc.cond.set_alive(true);
                for _ in 0..4 {
                    npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                    npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                    npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                    npc.vel_y = self.rng.range(-0x600..0) as i32;

                    let _ = npc_list.spawn(0x100, npc.clone());
                }
            }
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

        self.clamp_fall_speed();

        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n014_key[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n015_chest_closed(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if state.difficulty == GameDifficulty::Hard && state.constants.missile_flags.contains(&self.flag_num) {
            self.cond.set_alive(false);
            return Ok(());
        }

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.npc_flags.set_interactable(true);

                    if self.direction == Direction::Right {
                        self.vel_y = -0x200;

                        let mut npc = NPC::create(4, &state.npc_table);
                        npc.cond.set_alive(true);

                        for _ in 0..4 {
                            npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                            npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                            npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                            npc.vel_y = self.rng.range(-0x600..0) as i32;

                            let _ = npc_list.spawn(0x100, npc.clone());
                        }
                    }
                }

                self.anim_num = 0;
                if self.rng.range(0..30) == 0 {
                    self.action_num = 2;
                }
            }
            2 => {
                self.animate(1, 0, 3);
                if self.anim_num > 2 {
                    self.anim_num = 0;
                    self.action_num = 1;
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n015_closed_chest[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n016_save_point(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if self.action_num == 0 {
            self.npc_flags.set_interactable(true);
            self.action_num = 1;
            
            if self.direction == Direction::Right {
                self.npc_flags.set_interactable(false);
                self.vel_y = -0x200;
                
                //Creates smoke
                let mut npc = NPC::create(4, &state.npc_table);
                npc.cond.set_alive(true);

                for _ in 0..4 {
                    npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                    npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                    npc.vel_x = self.rng.range(-341..341) as i32;
                    npc.vel_y = self.rng.range(-0x600..0) as i32;

                    let _ = npc_list.spawn(0x100, npc.clone());
                }
            }
        }

        if self.action_num == 1 && self.flags.hit_bottom_wall() {
            self.npc_flags.set_interactable(true);
        }

        self.anim_counter = (self.anim_counter + 1) % 24;
        self.anim_num = self.anim_counter / 3;
        self.anim_rect = state.constants.npc.n016_save_point[self.anim_num as usize];

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n017_health_refill(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            
            //Creates smoke when spawned in a shelter
            if self.direction == Direction::Right {
                self.vel_y = -0x200;

                //Creates smoke
                let mut npc = NPC::create(4, &state.npc_table);
                npc.cond.set_alive(true);

                for _ in 0..4 {
                    npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                    npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                    npc.vel_x = self.rng.range(-341..341) as i32;
                    npc.vel_y = self.rng.range(-0x600..0) as i32;

                    let _ = npc_list.spawn(0x100, npc.clone());
                }
            }
        }

        match self.action_num {
            1 => {
                let rand = self.rng.range(0..30);

                if rand < 10 {
                    self.action_num = 2;
                } else if rand < 25 {
                    self.action_num = 3;
                } else {
                    self.action_num = 4;
                }

                self.action_counter = self.rng.range(0x10..0x40) as u16;
                self.anim_counter = 0;
            }
            2 => {
                self.anim_rect = state.constants.npc.n017_health_refill[0];
                self.anim_num = 0;

                self.action_counter = self.action_counter.saturating_sub(1);
                if self.action_counter == 0 {
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

                self.action_counter = self.action_counter.saturating_sub(1);
                if self.action_counter == 0 {
                    self.action_num = 1;
                }
            }
            4 => {
                self.anim_num = 1;
                self.anim_rect = state.constants.npc.n017_health_refill[1];

                self.action_counter = self.action_counter.saturating_sub(1);
                if self.action_counter == 0 {
                    self.action_num = 1;
                }
            }
            _ => (),
        }

        self.vel_y += 0x40;
        self.clamp_fall_speed();

        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n018_door(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 => match self.direction {
                Direction::Left => self.anim_rect = state.constants.npc.n018_door[0],
                Direction::Right => self.anim_rect = state.constants.npc.n018_door[1],
                _ => (),
            },
            1 => {
                let mut npc = NPC::create(4, &state.npc_table);

                npc.cond.set_alive(true);
                npc.x = self.x;
                npc.y = self.y;
                for _ in 0..4 {
                    npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                    npc.vel_y = self.rng.range(-0x600..0) as i32;

                    let _ = npc_list.spawn(0x100, npc.clone());
                }

                self.action_num = 0;
                self.anim_rect = state.constants.npc.n018_door[0]
            }
            _ => (),
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
            _ => (),
        }

        Ok(())
    }

    pub(crate) fn tick_n021_chest_open(&mut self, state: &mut SharedGameState) -> GameResult {
        if state.difficulty == GameDifficulty::Hard && state.constants.missile_flags.contains(&self.flag_num) {
            self.cond.set_alive(false);
            return Ok(());
        }

        if self.action_num == 0 {
            self.action_num = 1;

            if self.direction == Direction::Right {
                self.y += 0x2000;
            }

            self.anim_rect = state.constants.npc.n021_chest_open;
        }

        Ok(())
    }

    pub(crate) fn tick_n022_teleporter(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 => {
                self.anim_num = 0;
            }
            1 => {
                self.anim_num = (self.anim_num + 1) & 1;
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n022_teleporter[self.anim_num as usize];

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
                0 | 1 => {
                    if self.action_num == 0 {
                        self.action_num = 1;
                        self.anim_counter = 0;
                        self.anim_rect = state.constants.npc.n030_hermit_gunsmith[0];
                    }

                    if self.rng.range(0..120) == 10 {
                        self.action_num = 2;
                        self.action_counter = 0;
                        self.anim_rect = state.constants.npc.n030_hermit_gunsmith[1];
                    }
                }
                2 => {
                    self.action_counter += 1;
                    if self.action_counter > 8 {
                        self.action_num = 1;
                        self.anim_rect = state.constants.npc.n030_hermit_gunsmith[0];
                    }
                }
                _ => (),
            }
        } else {
            if self.action_num == 0 {
                self.action_num = 1;
                self.anim_rect = state.constants.npc.n030_hermit_gunsmith[2];
                self.y += 0x2000;
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
        if state.difficulty == GameDifficulty::Hard {
            self.cond.set_alive(false);
            return Ok(());
        }

        self.anim_counter = (self.anim_counter + 1) % 6;
        self.anim_num = self.anim_counter / 3;
        self.anim_rect = state.constants.npc.n032_life_capsule[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n034_bed(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            match self.direction {
                Direction::Left => self.anim_rect = state.constants.npc.n034_bed[0],
                Direction::Right => self.anim_rect = state.constants.npc.n034_bed[1],
                _ => (),
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

    pub(crate) fn tick_n038_fireplace(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 => {
                self.anim_counter = (self.anim_counter + 1) % 16;
                self.anim_num = self.anim_counter / 4;
                self.anim_rect = state.constants.npc.n038_fireplace[self.anim_num as usize];
            }
            10 | 11 => {
                if self.action_num == 10 {
                    self.action_num = 11;
                    npc_list.create_death_smoke(
                        self.x,
                        self.y,
                        self.display_bounds.right as usize,
                        8,
                        state,
                        &mut self.rng,
                    );
                }

                self.anim_rect.left = 0;
                self.anim_rect.right = 0;
            }
            _ => (),
        }

        Ok(())
    }

    pub(crate) fn tick_n039_save_sign(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            match self.direction {
                Direction::Left => self.anim_rect = state.constants.npc.n039_save_sign[0],
                Direction::Right => self.anim_rect = state.constants.npc.n039_save_sign[1],
                _ => (),
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n041_busted_door(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.layer = NPCLayer::Background;
            self.anim_rect = state.constants.npc.n041_busted_door;
            self.y -= 0x2000;
        }

        Ok(())
    }

    pub(crate) fn tick_n043_chalkboard(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.y -= 0x2000;

            match self.direction {
                Direction::Left => self.anim_rect = state.constants.npc.n043_chalkboard[0],
                Direction::Right => self.anim_rect = state.constants.npc.n043_chalkboard[1],
                _ => (),
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n046_hv_trigger(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        // Nicalis
        if state.constants.is_cs_plus && self.tsc_direction != 0 {
            self.direction = Direction::Right;
        }
        self.npc_flags.set_event_when_touched(true);

        let player = self.get_closest_player_mut(players);

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

    pub(crate) fn tick_n072_sprinkler(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        if self.direction == Direction::Left {
            self.animate(1, 0, 2);
            if self.anim_num > 1 {
                self.anim_num = 0;
                return Ok(());
            }

            let player = self.get_closest_player_mut(players);
            if (player.x - self.x).abs() < 0x28000 && (player.y - self.y).abs() < 0x1E000 {
                self.action_counter = self.action_counter.wrapping_add(1);

                let mut droplet = NPC::create(73, &state.npc_table);
                droplet.cond.set_alive(true);
                droplet.direction = Direction::Left;
                droplet.x = self.x;
                droplet.y = self.y;
                droplet.vel_x = 2 * self.rng.range(-0x200..0x200) as i32;
                droplet.vel_y = 3 * self.rng.range(-0x200..0x80) as i32;
                let _ = npc_list.spawn(0x100, droplet.clone());

                if self.action_counter % 2 == 1 {
                    droplet.vel_x = 2 * self.rng.range(-0x200..0x200) as i32;
                    droplet.vel_y = 3 * self.rng.range(-0x200..0x80) as i32;
                    let _ = npc_list.spawn(0x100, droplet);
                }
            }
        }

        self.anim_rect = state.constants.npc.n072_sprinkler[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n073_water_droplet(&mut self, state: &mut SharedGameState, stage: &Stage) -> GameResult {
        self.vel_y += 0x20;

        self.anim_rect = state.constants.npc.n073_water_droplet[self.rng.range(0..4) as usize];

        self.clamp_fall_speed();

        self.x += self.vel_x;
        self.y += self.vel_y;

        if self.direction == Direction::Right {
            self.anim_rect.top += 2;
            self.anim_rect.bottom += 2;
        }

        self.action_counter += 1;
        if self.action_counter > 10
            && (self.flags.hit_left_wall()
            || self.flags.hit_right_wall()
            || self.flags.hit_bottom_wall()
            || self.flags.in_water())
        {
            // hit something
            self.cond.set_alive(false);
        }

        if self.y > stage.map.height as i32 * state.tile_size.as_int() * 0x200 {
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
                Direction::Left => self.anim_rect = state.constants.npc.n078_pot[0],
                Direction::Right => self.anim_rect = state.constants.npc.n078_pot[1],
                _ => (),
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n085_terminal(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        match self.action_num {
            0 => {
                self.anim_num = 0;
                let player = self.get_closest_player_mut(players);

                if (player.x - self.x).abs() < 0x1000 && player.y < self.y + 0x1000 && player.y > self.y - 0x2000 {
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
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 3 };

        self.anim_rect = state.constants.npc.n085_terminal[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n090_background(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_rect = state.constants.npc.n090_background;
        }

        Ok(())
    }

    pub(crate) fn tick_n096_fan_left(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 && self.direction == Direction::Right {
                    self.action_num = 2;
                }

                self.anim_num = 0;
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

                {
                    let i = self.get_closest_player_idx_mut(&players);
                    if (players[i].x - self.x).abs() < 0x28000
                        && (players[i].y - self.y).abs() < 0x1e000
                        && self.rng.range(0..5) == 1
                    {
                        let mut particle = NPC::create(199, &state.npc_table);
                        particle.cond.set_alive(true);
                        particle.direction = Direction::Left;
                        particle.x = self.x;
                        particle.y = self.y + (self.rng.range(-8..8) * 0x200) as i32;
                        let _ = npc_list.spawn(0x100, particle);
                    }
                }

                for player in players {
                    if !player.cond.alive() || player.cond.hidden() {
                        continue;
                    }

                    if (player.y - self.y).abs() < 0x1000 && player.x < self.x && player.x > self.x - 0xc000 {
                        player.vel_x -= 0x88;
                        player.cond.set_increase_acceleration(true);
                    }
                }
            }
            _ => (),
        }

        if self.anim_counter == 0 {
            self.anim_rect = state.constants.npc.n096_fan_left[self.anim_num as usize];
        }

        Ok(())
    }

    pub(crate) fn tick_n097_fan_up(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 && self.direction == Direction::Right {
                    self.action_num = 2;
                }

                self.anim_num = 0;
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

                {
                    let i = self.get_closest_player_idx_mut(&players);
                    if (players[i].x - self.x).abs() < 0x28000
                        && (players[i].y - self.y).abs() < 0x1e000
                        && self.rng.range(0..5) == 1
                    {
                        let mut particle = NPC::create(199, &state.npc_table);
                        particle.cond.set_alive(true);
                        particle.direction = Direction::Up;
                        particle.x = self.x + (self.rng.range(-8..8) * 0x200) as i32;
                        particle.y = self.y;
                        let _ = npc_list.spawn(0x100, particle);
                    }
                }

                for player in players {
                    if !player.cond.alive() || player.cond.hidden() {
                        continue;
                    }

                    if (player.x - self.x).abs() < 0x1000 && player.y < self.y && player.y > self.y - 0xc000 {
                        player.vel_y -= 0x88;
                    }
                }
            }
            _ => (),
        }

        if self.anim_counter == 0 {
            self.anim_rect = state.constants.npc.n097_fan_up[self.anim_num as usize];
        }

        Ok(())
    }

    pub(crate) fn tick_n098_fan_right(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 && self.direction == Direction::Right {
                    self.action_num = 2;
                }

                self.anim_num = 0;
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

                {
                    let i = self.get_closest_player_idx_mut(&players);
                    if (players[i].x - self.x).abs() < 0x28000
                        && (players[i].y - self.y).abs() < 0x1e000
                        && self.rng.range(0..5) == 1
                    {
                        let mut particle = NPC::create(199, &state.npc_table);
                        particle.cond.set_alive(true);
                        particle.direction = Direction::Right;
                        particle.x = self.x;
                        particle.y = self.y + (self.rng.range(-8..8) * 0x200) as i32;
                        let _ = npc_list.spawn(0x100, particle);
                    }
                }

                for player in players {
                    if (player.y - self.y).abs() < 0x1000 && player.x > self.x && player.x < self.x + 0xc000 {
                        player.vel_x += 0x88;
                        player.cond.set_increase_acceleration(true);
                    }
                }
            }
            _ => (),
        }

        if self.anim_counter == 0 {
            self.anim_rect = state.constants.npc.n098_fan_right[self.anim_num as usize];
        }

        Ok(())
    }

    pub(crate) fn tick_n099_fan_down(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 && self.direction == Direction::Right {
                    self.action_num = 2;
                }

                self.anim_num = 0;
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

                {
                    let i = self.get_closest_player_idx_mut(&players);
                    if (players[i].x - self.x).abs() < 0x28000
                        && (players[i].y - self.y).abs() < 0x1e000
                        && self.rng.range(0..5) == 1
                    {
                        let mut particle = NPC::create(199, &state.npc_table);
                        particle.cond.set_alive(true);
                        particle.direction = Direction::Bottom;
                        particle.x = self.x + (self.rng.range(-8..8) * 0x200) as i32;
                        particle.y = self.y;
                        let _ = npc_list.spawn(0x100, particle);
                    }
                }

                for player in players {
                    if (player.x - self.x).abs() < 0x1000 && player.y > self.y && player.y < self.y + 0xc000 {
                        player.vel_y += 0x88;
                    }
                }
            }
            _ => (),
        }

        if self.anim_counter == 0 {
            self.anim_rect = state.constants.npc.n099_fan_down[self.anim_num as usize];
        }

        Ok(())
    }

    pub(crate) fn tick_n105_hey_bubble_low(&mut self, state: &mut SharedGameState) -> GameResult {
        self.action_counter += 1;

        if self.action_counter < 5 {
            self.y -= 0x200
        } else if self.action_counter > 30 {
            self.cond.set_alive(false);
        }

        self.anim_rect = state.constants.npc.n105_hey_bubble_low[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n106_hey_bubble_high(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            let mut npc = NPC::create(105, &state.npc_table);
            npc.cond.set_alive(true);
            npc.x = self.x;
            npc.y = self.y - 0x1000;

            let _ = npc_list.spawn(0x180, npc);
        }

        Ok(())
    }

    pub(crate) fn tick_n114_press(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.y -= 0x800;
                }

                if !self.flags.hit_bottom_wall() {
                    self.action_num = 10;
                    self.anim_num = 1;
                    self.anim_counter = 0;
                }
            }
            10 => {
                self.animate(2, 2, 2);

                for player in players {
                    if !player.cond.alive() || player.cond.hidden() {
                        continue;
                    }
                    if player.y > self.y {
                        self.npc_flags.set_solid_hard(false);
                        self.damage = 127;
                        break;
                    } else {
                        self.npc_flags.set_solid_hard(true);
                        self.damage = 0;
                    }
                }

                if self.flags.hit_bottom_wall() {
                    if self.anim_num > 1 {
                        let mut npc = NPC::create(4, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.x;
                        npc.y = self.y;

                        for _ in 0..4 {
                            npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                            npc.vel_y = self.rng.range(-0x600..0) as i32;

                            let _ = npc_list.spawn(0x100, npc.clone());
                        }

                        state.quake_counter = 10;
                        state.quake_rumble_counter = 10;
                        state.sound_manager.play_sfx(26);
                    }

                    self.action_num = 1;
                    self.anim_num = 0;
                    self.damage = 0;
                    self.npc_flags.set_solid_hard(true);
                }
            }
            _ => (),
        }

        self.vel_y += 0x20;

        self.clamp_fall_speed();

        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n114_press[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n116_red_petals(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_rect = state.constants.npc.n116_red_petals;
        }

        Ok(())
    }

    pub(crate) fn tick_n119_table_chair(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_rect = state.constants.npc.n119_table_chair;
        }

        Ok(())
    }

    pub(crate) fn tick_n125_hidden_item(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if self.life < 990 {
            npc_list.create_death_smoke(self.x, self.y, self.display_bounds.right as usize, 8, state, &mut self.rng);
            self.cond.set_alive(false);
            state.sound_manager.play_sfx(70);

            match self.direction {
                // hidden heart
                Direction::Left => {
                    let mut npc = NPC::create(87, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    npc.direction = Direction::Right;
                    let _ = npc_list.spawn(0, npc);
                }
                // hidden missile
                Direction::Right => {
                    let mut npc = NPC::create(86, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x;
                    npc.y = self.y;
                    npc.direction = Direction::Right;
                    let _ = npc_list.spawn(0, npc);
                }
                _ => (),
            }
        }

        match self.direction {
            Direction::Left => self.anim_rect = state.constants.npc.n125_hidden_item[0],
            Direction::Right => self.anim_rect = state.constants.npc.n125_hidden_item[1],
            _ => (),
        }

        Ok(())
    }

    pub(crate) fn tick_n137_large_door_frame(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_rect = state.constants.npc.n137_large_door_frame;
        }

        Ok(())
    }

    pub(crate) fn tick_n146_lightning(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
        flash: &mut Flash,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    if self.direction == Direction::Right {
                        flash.set_blink();
                    }
                }

                self.action_counter += 1;
                if self.action_counter > 10 {
                    self.action_num = 2;
                    state.sound_manager.play_sfx(101);
                }
            }
            2 => {
                self.anim_counter += 1;
                if self.anim_counter > 2 {
                    self.anim_counter = 0;
                    self.anim_num += 1;
                }
                if self.anim_num == 2 {
                    self.damage = 10;
                }
                if self.anim_num > 4 {
                    npc_list.create_death_smoke(self.x, self.y, 4096, 8, state, &mut self.rng);
                    self.cond.set_alive(false);
                    return Ok(());
                }
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n146_lightning[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n149_horizontal_moving_block(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 => {
                self.x += 0x1000;
                self.y += 0x1000;
                self.npc_flags.set_solid_hard(true);
                self.vel_x = 0;
                self.vel_y = 0;
                self.action_num = if self.direction == Direction::Right { 20 } else { 10 };
            }
            10 => {
                self.npc_flags.set_rear_and_top_not_hurt(false);
                self.damage = 0;
                let player = self.get_closest_player_mut(players);
                if (player.x < self.x + 0x3200)
                    && (player.x > self.x - 0x32000)
                    && (player.y < self.y + 0x3200)
                    && (player.y > self.y - 0x3200)
                {
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
                    state.quake_rumble_counter = 10;
                    state.sound_manager.play_sfx(26);

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    for _ in 0..4 {
                        npc.x = self.x - 0x2000;
                        npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                } else {
                    let player = self.get_closest_player_mut(players);
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

                let player = self.get_closest_player_mut(players);
                if (player.x > self.x - 0x3200)
                    && (player.x < self.x + 0x32000)
                    && (player.y < self.y + 0x3200)
                    && (player.y > self.y - 0x3200)
                {
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
                    state.quake_rumble_counter = 10;
                    state.sound_manager.play_sfx(26);

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    for _ in 0..4 {
                        npc.x = self.x + 0x2000;
                        npc.y = self.y + self.rng.range(-12..12) as i32 * 0x200;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                } else {
                    let player = self.get_closest_player_mut(players);
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
            _ => (),
        }

        self.vel_x = self.vel_x.clamp(-0x200, 0x200);
        self.x += self.vel_x;

        if self.anim_num != 149 {
            self.anim_num = 149;
            self.anim_rect = state.constants.npc.n149_horizontal_moving_block;
        }

        Ok(())
    }

    pub(crate) fn tick_n152_shutter_stuck(&mut self) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            if self.direction == Direction::Right {
                self.y += 0x2000;
            }

            self.anim_rect = Rect { left: 0, top: 0, right: 0, bottom: 0 };
        }

        Ok(())
    }

    pub(crate) fn tick_n157_vertical_moving_block(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 => {
                self.x += 0x1000;
                self.y += 0x1000;
                self.npc_flags.set_solid_hard(true);
                self.vel_x = 0;
                self.vel_y = 0;
                self.action_num = if self.direction == Direction::Right { 20 } else { 10 };
            }
            10 => {
                self.npc_flags.set_rear_and_top_not_hurt(false);
                self.damage = 0;
                let player = self.get_closest_player_mut(players);
                if (player.y < self.y + 0x3200)
                    && (player.y > self.y - 0x32000)
                    && (player.x < self.x + 0x3200)
                    && (player.x > self.x - 0x3200)
                {
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
                    state.quake_rumble_counter = 10;
                    state.sound_manager.play_sfx(26);

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    for _ in 0..4 {
                        npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc.y = self.y - 0x2000;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                } else {
                    let player = self.get_closest_player_mut(players);
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

                let player = self.get_closest_player_mut(players);
                if (player.y > self.y - 0x3200)
                    && (player.y < self.y + 0x32000)
                    && (player.x < self.x + 0x3200)
                    && (player.x > self.x - 0x3200)
                {
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
                    state.quake_rumble_counter = 10;
                    state.sound_manager.play_sfx(26);

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    for _ in 0..4 {
                        npc.x = self.x + self.rng.range(-12..12) as i32 * 0x200;
                        npc.y = self.y + 0x2000;
                        npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                        npc.vel_y = self.rng.range(-0x600..0) as i32;

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                } else {
                    let player = self.get_closest_player_mut(players);
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
            _ => (),
        }

        self.vel_y = self.vel_y.clamp(-0x200, 0x200);
        self.y += self.vel_y;

        if self.anim_num != 149 {
            self.anim_num = 149;
            self.anim_rect = state.constants.npc.n149_horizontal_moving_block;
        }

        Ok(())
    }

    pub(crate) fn tick_n189_homing_flame(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
    ) -> GameResult {
        let player = self.get_closest_player_ref(&players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.vel_x = -0x40;
                }

                self.y += self.vel_y;
                self.action_counter += 1;
                if self.action_counter > 256 {
                    self.action_num = 10;
                }
            }
            10 => {
                self.vel_x += if player.x >= self.x { 8 } else { -8 };
                self.vel_y += if player.y >= self.y { 8 } else { -8 };

                self.vel_x = self.vel_x.clamp(-0x400, 0x400);
                self.vel_y = self.vel_y.clamp(-0x400, 0x400);
                self.x += self.vel_x;
                self.y += self.vel_y;
            }
            _ => (),
        }

        if player.x >= self.x {
            self.direction = Direction::Right;
        } else {
            self.direction = Direction::Left;
        }

        self.animate(2, 0, 2);

        self.anim_rect = state.constants.npc.n189_homing_flame[self.anim_num as usize];
        Ok(())
    }

    pub(crate) fn tick_n190_broken_robot(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 => self.anim_num = 0,
            10 => {
                let mut npc = NPC::create(4, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = self.x;

                for _ in 0..8 {
                    npc.y = self.y + self.rng.range(-8..8) * 0x200;
                    npc.vel_x = self.rng.range(-8..-2) * 0x200;
                    npc.vel_y = self.rng.range(-3..3) * 0x200;

                    let _ = npc_list.spawn(0x100, npc.clone());
                }

                state.sound_manager.play_sfx(72);
                self.cond.set_alive(false);
            }
            20 => self.animate(10, 0, 1),
            _ => (),
        }

        self.anim_rect = state.constants.npc.n190_broken_robot[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n191_water_level(&mut self, state: &mut SharedGameState) -> GameResult {
        match self.action_num {
            0 | 10 => {
                if self.action_num == 0 {
                    self.action_num = 10;
                    self.target_y = self.y;
                    self.vel_y = 0x200;
                }

                self.vel_y += if self.y >= self.target_y { -4 } else { 4 };
                self.vel_y = self.vel_y.clamp(-0x100, 0x100);

                self.y += self.vel_y;
            }
            20 | 21 => {
                if self.action_num == 20 {
                    self.action_num = 21;
                    self.action_counter = 0;
                }

                self.vel_y += if self.y >= self.target_y { -4 } else { 4 };
                self.vel_y = self.vel_y.clamp(-0x200, 0x200);

                self.y += self.vel_y;

                self.action_counter += 1;
                if self.action_counter > 1000 {
                    self.action_num = 22;
                }
            }
            22 => {
                self.vel_y += if self.y >= 0 { -4 } else { 4 };
                self.vel_y = self.vel_y.clamp(-0x200, 0x200);

                self.y += self.vel_y;

                if self.y <= 0x7FFF || state.npc_super_pos.1 > 0 {
                    self.action_num = 21;
                    self.action_counter = 0;
                }
            }
            30 => {
                self.vel_y += if self.y >= 0 { -4 } else { 4 };
                self.vel_y = self.vel_y.clamp(-0x200, 0x100);

                self.y += self.vel_y;
            }
            _ => (),
        }

        state.water_level = self.y;
        self.anim_rect = Rect::new(0, 0, 0, 0);

        Ok(())
    }

    pub(crate) fn tick_n194_broken_blue_robot(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.y += 0x800;
        }

        self.anim_rect = state.constants.npc.n194_broken_blue_robot;

        Ok(())
    }

    pub(crate) fn tick_n195_background_grate(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.layer = NPCLayer::Background;
            self.action_num = 1;
            self.anim_rect = state.constants.npc.n195_background_grate;
        }

        Ok(())
    }

    pub(crate) fn tick_n199_wind_particles(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = self.rng.range(0..2) as u16;
            self.vel_x = self.direction.vector_x() * (self.rng.range(4..8) * 0x200 / 2) as i32;
            self.vel_y = self.direction.vector_y() * (self.rng.range(4..8) * 0x200 / 2) as i32;
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

    pub(crate) fn tick_n216_debug_cat(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_rect = state.constants.npc.n216_debug_cat;
        }

        Ok(())
    }

    pub(crate) fn tick_n219_smoke_generator(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if self.direction != Direction::Left {
            let mut npc = NPC::create(199, &state.npc_table);
            npc.x = self.x + self.rng.range(-160..160) * 0x200;
            npc.y = self.y + self.rng.range(-128..128) * 0x200;
            npc.direction = Direction::Right;

            let _ = npc_list.spawn(0x100, npc);
        } else if self.rng.range(0..40) == 1 {
            let mut npc = NPC::create(4, &state.npc_table);
            npc.x = self.x + self.rng.range(-20..20) * 0x200;
            npc.y = self.y;
            npc.cond.set_alive(true);
            npc.vel_y = -0x200;

            let _ = npc_list.spawn(0x100, npc);
        }

        self.anim_rect = Rect::new(0, 0, 0, 0);

        Ok(())
    }

    pub(crate) fn tick_n222_prison_bars(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.y -= 0x1000;

            self.action_num = 1;
            self.anim_rect = state.constants.npc.n222_prison_bars;
        }

        Ok(())
    }

    pub(crate) fn tick_n227_bucket(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_rect = state.constants.npc.n227_bucket;
        }

        Ok(())
    }

    pub(crate) fn tick_n229_red_flowers_sprouts(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.y -= 0x2000;

            let anim = if self.direction == Direction::Left { 0 } else { 1 };

            self.anim_rect = state.constants.npc.n229_red_flowers_sprouts[anim];
        }

        Ok(())
    }

    pub(crate) fn tick_n230_red_flowers_blooming(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.x -= 0x2000;
            self.y -= 0x2000;

            let anim = if self.direction == Direction::Left { 0 } else { 1 };

            self.anim_rect = state.constants.npc.n230_red_flowers_blooming[anim];
        }

        Ok(())
    }

    pub(crate) fn tick_n234_red_flowers_picked(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.y += 0x2000;

            match self.direction {
                Direction::Left => self.anim_rect = state.constants.npc.n234_red_flowers_picked[0],
                Direction::Right => self.anim_rect = state.constants.npc.n234_red_flowers_picked[1],
                _ => self.anim_rect = state.constants.npc.n234_red_flowers_picked[1],
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n238_press_sideways(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        let player = self.get_closest_player_ref(&players);

        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.target_x = self.x;
                    self.target_y = self.y;
                    self.display_bounds.left = 0x2000;
                    self.display_bounds.right = 0x1000;
                }

                if self.direction == Direction::Left
                    && player.x < self.x
                    && player.x > self.x - 0x18000
                    && player.y > self.y - 0x800
                    && player.y < self.y + 0x1000
                {
                    self.action_num = 10;
                    self.action_counter = 0;
                    self.anim_num = 2;
                }

                if self.direction == Direction::Right
                    && player.x > self.x
                    && player.x < self.x + 0x18000
                    && player.y > self.y - 0x800
                    && player.y < self.y + 0x1000
                {
                    self.action_num = 10;
                    self.action_counter = 0;
                    self.anim_num = 2;
                }
            }
            10 => {
                self.damage = 127;
                self.x += if self.direction != Direction::Left { 0xC00 } else { -0xC00 };

                self.action_counter += 1;
                if self.action_counter == 8 {
                    self.action_num = 20;
                    self.action_counter = 0;

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);

                    for _ in 0..4 {
                        npc.x = self.x + self.rng.range(-16..16) * 0x200;
                        npc.y = self.y + self.rng.range(-8..8) * 0x200;
                        let _ = npc_list.spawn(0x100, npc.clone());

                        state.sound_manager.play_sfx(12);
                    }
                }
            }
            20 => {
                self.damage = 0;
                self.action_counter += 1;
                if self.action_counter > 50 {
                    self.action_counter = 0;
                    self.action_num = 30;
                }
            }
            30 => {
                self.damage = 0;
                self.anim_num = 1;
                self.action_counter += 1;
                if self.action_counter == 12 {
                    self.action_num = 1;
                    self.action_counter = 0;
                    self.anim_num = 0;
                }

                self.x += if self.direction != Direction::Left { -0x800 } else { 0x800 };
            }
            _ => (),
        }

        if self.direction != Direction::Left || player.x >= self.x {
            if self.direction == Direction::Right && player.x > self.x {
                self.hit_bounds.right = 0x2000;
            } else {
                self.hit_bounds.right = 0x1000;
            }
        } else {
            self.hit_bounds.right = 0x2000;
        }

        self.anim_rect = state.constants.npc.n238_press_sideways[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n239_cage_bars(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            if self.direction == Direction::Left {
                self.x += 0x1000;
                self.y += 0x2000;
            } else {
                self.display_bounds.left = 0x3000;
                self.display_bounds.top = 0x1000;
                self.display_bounds.right = 0x3000;
            }
        }

        let anim = if self.direction == Direction::Left { 0 } else { 1 };

        self.anim_rect = state.constants.npc.n239_cage_bars[anim];

        Ok(())
    }

    pub(crate) fn tick_n246_press_proximity(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        match self.action_num {
            0 | 1 => {
                if self.action_num == 0 {
                    self.action_num = 1;
                    self.y -= 0x800;
                }

                for player in players {
                    if player.x < self.x + 0x1000
                        && player.x > self.x - 0x1000
                        && player.y > self.y + 0x1000
                        && player.y < self.y + 0x10000
                    {
                        self.action_num = 5;
                    }
                }
            }
            5 => {
                if !self.flags.hit_bottom_wall() {
                    self.action_num = 10;
                    self.anim_counter = 0;
                    self.anim_num = 1;
                }
            }
            10 => {
                self.animate(2, 2, 2);

                for player in players {
                    if !player.cond.alive() || player.cond.hidden() {
                        continue;
                    }
                    if player.y > self.y {
                        self.npc_flags.set_solid_hard(false);
                        self.damage = 127;
                        break;
                    } else {
                        self.npc_flags.set_solid_hard(true);
                        self.damage = 0;
                    }
                }

                if self.flags.hit_bottom_wall() {
                    if self.anim_num > 1 {
                        let mut npc = NPC::create(4, &state.npc_table);
                        npc.cond.set_alive(true);
                        npc.x = self.x;
                        npc.y = self.y;

                        for _ in 0..4 {
                            npc.vel_x = self.rng.range(-0x155..0x155) as i32;
                            npc.vel_y = self.rng.range(-0x600..0) as i32;

                            let _ = npc_list.spawn(0x100, npc.clone());
                        }

                        state.quake_counter = 10;
                        state.quake_rumble_counter = 10;
                        state.sound_manager.play_sfx(26);
                    }

                    self.action_num = 20;
                    self.anim_num = 0;
                    self.anim_counter = 0;
                    self.damage = 0;
                    self.npc_flags.set_solid_hard(true);
                }
            }
            _ => (),
        }

        if self.action_num >= 5 {
            self.vel_y += 0x80;

            self.clamp_fall_speed();

            self.y += self.vel_y;
        }

        self.anim_rect = state.constants.npc.n246_press_proximity[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n253_experience_capsule(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
        }

        self.animate(4, 0, 1);

        if self.life <= 100 {
            self.cond.set_alive(false);

            self.create_xp_drop_custom(self.x, self.y, self.flag_num, state, npc_list);
            npc_list.create_death_smoke(self.x, self.y, self.display_bounds.right as usize, 8, state, &mut self.rng);

            state.sound_manager.play_sfx(25);
        }

        self.anim_rect = state.constants.npc.n253_experience_capsule[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n258_mimiga_sleeping(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_rect = state.constants.npc.n258_mimiga_sleeping;
        }

        Ok(())
    }

    pub(crate) fn tick_n271_ironhead_block(&mut self, state: &mut SharedGameState, stage: &mut Stage) -> GameResult {
        if self.vel_x < 0 && self.x < -0x2000
            || self.vel_x > 0 && self.x > stage.map.width as i32 * state.tile_size.as_int() * 0x200 + 0x2000
        {
            self.vanish(state);
            return Ok(());
        }

        if self.action_num == 0 {
            self.action_num = 1;
            let block = self.rng.range(0..9) as u16;
            if block == 9 {
                // Big Block
                self.anim_rect = Rect::new(0, 64, 32, 96);
                self.display_bounds = Rect::new(0x2000, 0x2000, 0x2000, 0x2000);
                self.hit_bounds = Rect::new(0x1800, 0x1800, 0x1800, 0x1800);
            } else {
                // Small Blocks
                let scale = state.tile_size.as_int() as u16;
                self.anim_rect = Rect::new_size(((block % 3) + 7) * scale, (block / 3) * scale, scale, scale);
            }

            self.vel_x = self.direction.vector_x() * 2 * self.rng.range(256..512);
            self.vel_y = self.rng.range(-512..512);
        }

        if self.vel_y < 0 && self.y - (self.hit_bounds.top as i32) < 0x1000 {
            self.vel_y = -self.vel_y;
            state.create_caret(self.x, self.y - 0x1000, CaretType::LittleParticles, Direction::Left);
            state.create_caret(self.x, self.y - 0x1000, CaretType::LittleParticles, Direction::Left);
        }

        if self.vel_y > 0 && (self.hit_bounds.bottom as i32) + self.y > 0x1D000 {
            self.vel_y = -self.vel_y;
            state.create_caret(self.x, self.y + 0x1000, CaretType::LittleParticles, Direction::Left);
            state.create_caret(self.x, self.y + 0x1000, CaretType::LittleParticles, Direction::Left);
        }

        self.x += self.vel_x;
        self.y += self.vel_y;

        Ok(())
    }

    pub(crate) fn tick_n272_ironhead_block_generator(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.action_counter = self.rng.range(0..200) as u16;
        }

        if self.action_num == 1 {
            if self.action_counter > 0 {
                self.action_counter -= 1;
            } else {
                self.action_num = 0;

                let mut npc = NPC::create(271, &state.npc_table);
                npc.cond.set_alive(true);

                npc.x = self.x;
                npc.y = self.y + self.rng.range(-32..32) * 0x200;
                npc.direction = self.direction;

                let _ = npc_list.spawn(0x100, npc);
            }
        }

        Ok(())
    }

    pub(crate) fn tick_n279_large_falling_block(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        stage: &mut Stage,
    ) -> GameResult {
        match self.action_num {
            0 | 10 | 11 => {
                if self.action_num == 0 {
                    match self.direction {
                        Direction::Left => {
                            self.action_num = 100;
                            self.npc_flags.set_invulnerable(true);
                            self.anim_num = 0;
                        }
                        Direction::Up => {
                            self.anim_num = 0;
                            self.action_num = 10;
                        }
                        Direction::Right => {
                            self.action_num = 100;
                            self.npc_flags.set_invulnerable(true);
                            self.anim_num = 1;
                            self.display_bounds = Rect::new(0x1000, 0x1000, 0x1000, 0x1000);
                            self.hit_bounds = Rect::new(0x1000, 0x1000, 0x1000, 0x1000);
                        }
                        _ => (),
                    }
                }

                if self.action_num != 0 || self.direction == Direction::Up {
                    if self.action_num == 10 {
                        self.action_num = 11;
                        self.action_counter = 16;
                    }
                    
                    self.action_counter = self.action_counter.saturating_sub(2);
                    if self.action_counter == 0 {
                        self.action_num = 100;
                        self.npc_flags.set_invulnerable(true);
                    }
                }
            }
            100 => {
                self.vel_y += 0x40;
                if self.vel_y > 0x700 {
                    self.vel_y = 0x700;
                }

                if self.y > 0x10000 {
                    self.npc_flags.set_ignore_solidity(false);
                }

                if self.flags.hit_bottom_wall() {
                    self.vel_y = -0x200;
                    self.action_num = 110;
                    self.npc_flags.set_ignore_solidity(true);
                    state.sound_manager.play_sfx(26);
                    state.quake_counter = 10;
                    state.quake_rumble_counter = 10;

                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);

                    for _ in 0..4 {
                        npc.x = self.x + self.rng.range(-12..12) * 0x200;
                        npc.y = self.y + 0x2000;
                        npc.vel_x = self.rng.range(-0x155..0x155);
                        npc.vel_y = self.rng.range(-0x600..0);

                        let _ = npc_list.spawn(0x100, npc.clone());
                    }
                }
            }
            110 => {
                self.vel_y += 0x40;
                if self.y > stage.map.height as i32 * state.tile_size.as_int() * 0x200 + 0x4000 {
                    self.cond.set_alive(false);
                    return Ok(());
                }
            }
            _ => (),
        }

        let player = self.get_closest_player_ref(&players);
        self.damage = if player.y > self.y { 10 } else { 0 };

        self.y += self.vel_y;

        self.anim_rect = state.constants.npc.n279_large_falling_block[self.anim_num as usize];

        if self.action_num == 11 {
            self.anim_rect.top += self.action_counter;
            self.anim_rect.bottom -= self.action_counter;
            self.display_bounds.top = (16u32).saturating_sub(self.action_counter as u32) * 0x200;
        }

        Ok(())
    }

    pub(crate) fn tick_n292_quake(&mut self, state: &mut SharedGameState) -> GameResult {
        state.quake_counter = 10;

        Ok(())
    }

    pub(crate) fn tick_n294_quake_falling_block_generator(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
        stage: &mut Stage,
    ) -> GameResult {
        match self.action_num {
            0 => {
                for player in players {
                    if player.x < (stage.map.width as i32 - 6) * state.tile_size.as_int() * 0x200 {
                        self.action_num = 1;
                        self.action_counter = 0;
                    }
                }
            }
            1 => {
                self.action_counter += 1;
                let player = &players[0];
                if player.equip.has_booster_2_0() {
                    self.x = player.x + 0x8000;
                    if self.x < 0x34000 {
                        self.x = 0x34000;
                    }
                } else {
                    self.x = player.x + 0xC000;
                    if self.x < 0x2E000 {
                        self.x = 0x2E000;
                    }
                }

                let map_start = (stage.map.width as i32 - 10) * state.tile_size.as_int() * 0x200;
                if self.x > map_start {
                    self.x = map_start;
                }

                if self.action_counter > 24 {
                    let mut npc = NPC::create(279, &state.npc_table);

                    npc.cond.set_alive(true);
                    npc.x = self.x
                        + if player.equip.has_booster_2_0() {
                        self.rng.range(-14..14)
                    } else {
                        self.rng.range(-11..11)
                    } * state.tile_size.as_int()
                        * 0x200;
                    npc.y = player.y - 0x1C000;
                    npc.direction = if self.rng.range(0..10) & 1 != 0 { Direction::Left } else { Direction::Right };

                    let _ = npc_list.spawn(0x100, npc);

                    self.action_counter = self.rng.range(0..15) as u16;
                }
            }
            _ => (),
        }

        Ok(())
    }

    pub(crate) fn tick_n295_cloud(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = self.tsc_direction % 4;

            match self.tsc_direction {
                0 => {
                    self.vel_y = -0x1000;
                    self.display_bounds.right = 0xD000;
                    self.display_bounds.left = 0xD000;
                }
                1 => {
                    self.vel_y = -0x800;
                    self.display_bounds.right = 0x7000;
                    self.display_bounds.left = 0x7000;
                }
                2 => {
                    self.vel_y = -0x400;
                    self.display_bounds.right = 0x4000;
                    self.display_bounds.left = 0x4000;
                }
                3 => {
                    self.vel_y = -0x200;
                    self.display_bounds.right = 0x2800;
                    self.display_bounds.left = 0x2800;
                }
                4 => {
                    self.vel_x = -0x400;
                    self.display_bounds.right = 0xD000;
                    self.display_bounds.left = 0xD000;
                }
                5 => {
                    self.vel_x = -0x200;
                    self.display_bounds.right = 0x7000;
                    self.display_bounds.left = 0x7000;
                }
                6 => {
                    self.vel_x = -0x100;
                    self.display_bounds.right = 0x4000;
                    self.display_bounds.left = 0x4000;
                }
                7 => {
                    self.vel_x = -0x80;
                    self.display_bounds.right = 0x2800;
                    self.display_bounds.left = 0x2800;
                }
                _ => (),
            }
        } else if self.action_num == 1 {
            self.x += self.vel_x;
            self.y += self.vel_y;

            if self.x < -0x8000 {
                self.cond.set_alive(false);
            }

            if self.y < -0x4000 {
                self.cond.set_alive(false);
            }
        }

        self.anim_rect = state.constants.npc.n295_cloud[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n296_cloud_generator(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        self.action_counter += 1;
        if self.action_counter <= 16 {
            return Ok(());
        }

        self.action_counter = self.rng.range(0..16) as u16;
        let dir = self.rng.range(0..100) as u16 & 3;
        let mut npc = NPC::create(295, &state.npc_table);
        npc.cond.set_alive(true);

        if self.direction != Direction::Left {
            let layer = match dir {
                0 => 128,
                1 => 85,
                2 => 64,
                3 => 0,
                _ => unsafe {
                    unreachable_unchecked();
                },
            };

            npc.x = self.x;
            npc.y = self.y + self.rng.range(-7..7) * state.tile_size.as_int() * 0x200;
            npc.tsc_direction = dir + 4;

            let _ = npc_list.spawn(layer, npc);
        } else {
            let layer = match dir {
                0 => 384,
                1 => 128,
                2 => 64,
                3 => 0,
                _ => unsafe { unreachable_unchecked() },
            };

            npc.x = self.x + self.rng.range(-10..10) * state.tile_size.as_int() * 0x200;
            npc.y = self.y;
            npc.tsc_direction = dir;

            let _ = npc_list.spawn(layer, npc);
        }

        Ok(())
    }

    pub(crate) fn tick_n297_sue_dragon_mouth(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        if let Some(npc) = self.get_parent_ref_mut(npc_list) {
            self.x = npc.x + 0x2000;
            self.y = npc.y + 0x1000;
        }

        self.anim_rect = state.constants.npc.n297_sue_dragon_mouth;

        Ok(())
    }

    pub(crate) fn tick_n302_camera_focus_marker(
        &mut self,
        state: &mut SharedGameState,
        mut players: [&mut Player; 2],
        npc_list: &NPCList,
        boss: &mut BossNPC,
    ) -> GameResult {
        let player = &mut players[state.textscript_vm.executor_player.index()];

        match self.action_num {
            10 => {
                self.x = player.x;
                self.y = player.y - 0x4000;
            }
            20 => {
                match self.direction {
                    Direction::Left => self.x -= 0x400,
                    Direction::Up => self.y -= 0x400,
                    Direction::Right => self.x += 0x400,
                    Direction::Bottom => self.y += 0x400,
                    _ => (),
                }
                player.x = self.x;
                player.y = self.y;
            }
            30 => {
                self.x = player.x;
                self.y = player.y + 0xa000;
            }
            100 | 101 => {
                if self.action_num == 100 {
                    self.action_num = 101;

                    if self.tsc_direction != 0 {
                        for npc in npc_list.iter_alive() {
                            if npc.event_num == self.tsc_direction {
                                self.parent_id = npc.id;
                                break;
                            }
                        }

                        if self.parent_id == 0 {
                            self.cond.set_alive(false);
                        }
                    } else {
                        // This shouldn't get hit but it's here for completeness
                        self.parent_id = 0;
                    }
                }

                if self.tsc_direction == 0 {
                    self.x = (player.x + boss.parts[0].x) / 2;
                    self.y = (player.y + boss.parts[0].y) / 2;
                } else if let Some(npc) = self.get_parent_ref_mut(npc_list) {
                    self.x = (player.x + npc.x) / 2;
                    self.y = (player.y + npc.y) / 2;
                }
            }
            _ => (),
        }

        Ok(())
    }

    pub(crate) fn tick_n328_human_transform_machine(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_rect = state.constants.npc.n328_human_transform_machine;
        }

        Ok(())
    }

    pub(crate) fn tick_n329_laboratory_fan(&mut self, state: &mut SharedGameState) -> GameResult {
        self.anim_counter = self.anim_counter.wrapping_add(1);
        self.anim_rect = state.constants.npc.n329_laboratory_fan[(self.anim_counter as usize / 2) & 1];

        Ok(())
    }

    pub(crate) fn tick_n334_sweat(&mut self, state: &mut SharedGameState, players: [&mut Player; 2]) -> GameResult {
        let player = self.get_closest_player_mut(players);

        match self.action_num {
            0 | 10 => {
                if self.action_num == 0 {
                    self.action_num = 10;
                    if self.direction == Direction::Left {
                        self.x += 0x1400;
                        self.y -= 0x2400;
                    } else {
                        self.x = player.x - 0x1400;
                        self.y = player.y - 0x400;
                    }
                }

                self.action_counter += 1;
                self.anim_num = if self.action_counter / 8 % 2 != 0 { 0 } else { 1 };

                if self.action_counter >= 64 {
                    self.cond.set_alive(false);
                }
            }
            _ => (),
        }

        let dir_offset = if self.direction == Direction::Left { 0 } else { 2 };
        self.anim_rect = state.constants.npc.n334_sweat[self.anim_num as usize + dir_offset];

        Ok(())
    }

    pub(crate) fn tick_n349_statue(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;

            if self.direction == Direction::Left {
                self.x += 0x1000;
            } else if self.direction == Direction::Right {
                self.y += 0x2000;
            }
        }

        self.anim_rect = state.constants.npc.n349_statue;

        Ok(())
    }

    pub(crate) fn tick_n351_statue_shootable(&mut self, state: &mut SharedGameState, npc_list: &NPCList) -> GameResult {
        match self.action_num {
            0 => {
                self.action_num = 1;
                self.anim_num = self.tsc_direction / 10;
                self.x += 0x1000;
                self.y += 0x1800;
            }
            10 | 11 => {
                if self.action_num == 10 {
                    if state.get_flag(self.flag_num as usize) {
                        self.action_num = 20;
                    } else {
                        self.action_num = 11;
                        self.npc_flags.set_shootable(true);
                    }
                }

                if self.life <= 900 {
                    let mut npc = NPC::create(351, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = self.x - 0x1000;
                    npc.y = self.y - 0x1800;
                    npc.tsc_direction = 10 * self.anim_num + 40;

                    let _ = npc_list.spawn(0, npc);
                    self.cond.set_explode_die(true);
                }
            }
            20 => {
                self.anim_num += 4;
                self.action_num = 1;
            }
            _ => (),
        }

        self.anim_rect = state.constants.npc.n351_statue_shootable[self.anim_num as usize % 9];

        Ok(())
    }

    pub(crate) fn tick_n352_ending_characters(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.anim_num = 0;
            self.action_counter2 = self.tsc_direction / 100;
            self.tsc_direction %= 100;

            if self.action_counter2 <= 13 && ((1 << self.action_counter2) & 0b11001110000000) != 0 {
                self.spritesheet_id = 21;
            }

            if self.action_counter2 <= 12 && ((1 << self.action_counter2) & 0b1001000010100) != 0 {
                self.display_bounds.top = 0x2000;
            }

            if self.action_counter2 == 9 {
                self.display_bounds.left = 0x2800;
                self.display_bounds.right = 0x2800;
                self.x -= 0x200;
            }

            if self.action_counter2 == 0 {
                let mut npc = NPC::create(145, &state.npc_table);
                npc.cond.set_alive(true);
                npc.direction = Direction::Right;
                npc.parent_id = self.id;

                let _ = npc_list.spawn(0x100, npc);
            }
        }

        if self.action_num == 1 {
            self.vel_y += 0x40;
            self.clamp_fall_speed();

            if self.flags.hit_bottom_wall() {
                self.action_num = 2;
                self.anim_num = 1;
                self.vel_y = 0;
            }

            self.y += self.vel_y;
        }

        self.anim_rect = state.constants.npc.n352_ending_characters
            [(2 * self.action_counter2 as usize + self.anim_num as usize) % 28];

        Ok(())
    }

    pub(crate) fn tick_n355_quote_and_curly_on_balrog(
        &mut self,
        state: &mut SharedGameState,
        npc_list: &NPCList,
    ) -> GameResult {
        if self.action_num == 0 {
            match (self.tsc_direction, self.direction) {
                // Co-op
                (4, _) => {
                    self.spritesheet_id = 16;
                    self.anim_num = 0;

                    if let Some(npc) = self.get_parent_ref_mut(npc_list) {
                        self.x = npc.x;
                        self.y = npc.y + 0x1400;
                    }
                }
                (5, _) => {
                    self.spritesheet_id = 16;
                    self.anim_num = 2;

                    if let Some(npc) = self.get_parent_ref_mut(npc_list) {
                        self.x = npc.x + 0x1600;
                        self.y = npc.y - 0x2200;
                    }
                }
                // Curly's position changes when 2P is present
                (6, Direction::Bottom) => {
                    self.spritesheet_id = 23;
                    self.anim_num = 3;

                    if let Some(npc) = self.get_parent_ref_mut(npc_list) {
                        self.x = npc.x + 0x400;
                        self.y = npc.y - 0x2600;
                    }
                }
                // Normal
                (_, Direction::Left) => {
                    self.spritesheet_id = 16;
                    self.anim_num = 0;

                    if let Some(npc) = self.get_parent_ref_mut(npc_list) {
                        self.x = npc.x - 0x1c00;
                        self.y = npc.y + 0x1400;
                    }
                }
                (_, Direction::Up) => {
                    self.spritesheet_id = 23;
                    self.anim_num = 1;

                    if let Some(npc) = self.get_parent_ref_mut(npc_list) {
                        self.x = npc.x + 0x1c00;
                        self.y = npc.y + 0x1400;
                    }
                }
                (_, Direction::Right) => {
                    self.spritesheet_id = 16;
                    self.anim_num = 2;

                    if let Some(npc) = self.get_parent_ref_mut(npc_list) {
                        self.x = npc.x - 0xe00;
                        if state.constants.is_switch {
                            self.y = npc.y - 0x2200;
                        } else {
                            self.y = npc.y - 0x2600;
                        }
                    }
                }
                (_, Direction::Bottom) => {
                    self.spritesheet_id = 23;
                    self.anim_num = 3;

                    if let Some(npc) = self.get_parent_ref_mut(npc_list) {
                        self.x = npc.x + 0x800;
                        self.y = npc.y - 0x2600;
                    }
                }
                _ => (),
            }
        }

        let quote_offset = if self.spritesheet_id == 16 {
            state.get_skinsheet_offset() * state.tile_size.as_int() as u16 * 2
        } else {
            0
        };
        self.anim_rect = state.constants.npc.n355_quote_and_curly_on_balrog[self.anim_num as usize];
        self.anim_rect.top += quote_offset;
        self.anim_rect.bottom += quote_offset;

        Ok(())
    }

    pub(crate) fn tick_n358_misery_credits(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.animate(6, 0, 1);
        } else if self.action_num == 10 {
            self.animate(6, 3, 4);
        }

        self.anim_rect = state.constants.npc.n358_misery_credits[self.anim_num as usize];

        Ok(())
    }

    pub(crate) fn tick_n359_water_droplet_generator(
        &mut self,
        state: &mut SharedGameState,
        players: [&mut Player; 2],
        npc_list: &NPCList,
    ) -> GameResult {
        let player = self.get_closest_player_mut(players);
        if (player.x - self.x).abs() < 0x28000
            && player.y < self.y + 0x28000
            && player.y > self.y - 0x14000
            && self.rng.range(0..100) == 2
        {
            let mut npc = NPC::create(73, &state.npc_table);
            npc.cond.set_alive(true);
            npc.x = self.x + self.rng.range(-6..6) * 0x200;
            npc.y = self.y - 0xe00;

            let _ = npc_list.spawn(0, npc);
        }

        Ok(())
    }

    pub(crate) fn tick_n360_credits_thank_you(&mut self, state: &mut SharedGameState) -> GameResult {
        if self.action_num == 0 {
            self.action_num = 1;
            self.x -= 0x1000;
            self.y -= 0x1000;
        }

        self.anim_rect = state.constants.npc.n360_credits_thank_you;

        Ok(())
    }
}
