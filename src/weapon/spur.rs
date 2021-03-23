use crate::caret::CaretType;
use crate::common::Direction;
use crate::player::{Player, TargetPlayer};
use crate::shared_game_state::SharedGameState;
use crate::weapon::bullet::BulletManager;
use crate::weapon::{Weapon, WeaponLevel};

impl Weapon {
    pub(in crate::weapon) fn tick_spur(
        &mut self,
        player: &mut Player,
        player_id: TargetPlayer,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        const BULLETS: [u16; 6] = [44, 45, 46, 47, 48, 49];

        let mut shoot = false;
        let btype;

        if player.controller.shoot() {
            self.add_xp(if player.equip.has_turbocharge() { 3 } else { 2 }, player, state);
            self.counter1 += 1;

            if (self.counter1 / 2 % 2) != 0 {
                match self.level {
                    WeaponLevel::Level1 => {
                        state.sound_manager.play_sfx(59);
                    }
                    WeaponLevel::Level2 => {
                        state.sound_manager.play_sfx(60);
                    }
                    WeaponLevel::Level3 => {
                        if let (_, _, false) = self.get_max_exp(&state.constants) {
                            state.sound_manager.play_sfx(61);
                        }
                    }
                    WeaponLevel::None => unreachable!(),
                }
            }
        } else {
            if self.counter1 > 0 {
                shoot = true;
                self.counter1 = 0;
            }
        }

        if let (_, _, true) = self.get_max_exp(&state.constants) {
            if self.counter2 == 0 {
                self.counter2 = 1;
                state.sound_manager.play_sfx(65);
            }
        } else {
            self.counter2 = 0;
        }

        let level = self.level;
        if !player.controller.shoot() {
            self.reset_xp();
        }

        match level {
            WeaponLevel::Level1 => {
                btype = 6;
                shoot = false;
            }
            WeaponLevel::Level2 => btype = 37,
            WeaponLevel::Level3 => {
                if self.counter2 == 1 {
                    btype = 39;
                } else {
                    btype = 38;
                }
            }
            WeaponLevel::None => unreachable!(),
        }

        if bullet_manager.count_bullets_multi(&BULLETS, player_id) > 0 || !(player.controller.trigger_shoot() || shoot)
        {
            return;
        }

        if !self.consume_ammo(1) {
            state.sound_manager.play_sfx(37);
        } else {
            match player.direction {
                Direction::Left if player.up => {
                    bullet_manager.create_bullet(
                        player.x - 0x200,
                        player.y - 0x1000,
                        btype,
                        player_id,
                        Direction::Up,
                        &state.constants,
                    );
                    state.create_caret(player.x - 0x200, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                }
                Direction::Right if player.up => {
                    bullet_manager.create_bullet(
                        player.x + 0x200,
                        player.y - 0x1000,
                        btype,
                        player_id,
                        Direction::Up,
                        &state.constants,
                    );
                    state.create_caret(player.x + 0x200, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                }
                Direction::Left if player.down => {
                    bullet_manager.create_bullet(
                        player.x - 0x200,
                        player.y + 0x1000,
                        btype,
                        player_id,
                        Direction::Bottom,
                        &state.constants,
                    );
                    state.create_caret(player.x - 0x200, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                }
                Direction::Right if player.down => {
                    bullet_manager.create_bullet(
                        player.x + 0x200,
                        player.y + 0x1000,
                        btype,
                        player_id,
                        Direction::Bottom,
                        &state.constants,
                    );
                    state.create_caret(player.x + 0x200, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                }
                Direction::Left => {
                    bullet_manager.create_bullet(
                        player.x - 0xc00,
                        player.y + 0x600,
                        btype,
                        player_id,
                        Direction::Left,
                        &state.constants,
                    );
                    state.create_caret(player.x - 0xc00, player.y + 0x600, CaretType::Shoot, Direction::Left);
                }
                Direction::Right => {
                    bullet_manager.create_bullet(
                        player.x + 0xc00,
                        player.y + 0x600,
                        btype,
                        player_id,
                        Direction::Right,
                        &state.constants,
                    );
                    state.create_caret(player.x + 0xc00, player.y + 0x600, CaretType::Shoot, Direction::Right);
                }
                _ => {}
            }

            let sound = match btype {
                6 => 49,
                37 => 62,
                38 => 63,
                39 => 64,
                _ => 0,
            };

            state.sound_manager.play_sfx(sound);
        }
    }
}
