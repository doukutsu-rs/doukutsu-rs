use crate::caret::CaretType;
use crate::common::Direction;
use crate::player::{Player, TargetPlayer};
use crate::shared_game_state::SharedGameState;
use crate::weapon::bullet::BulletManager;
use crate::weapon::{Weapon, WeaponLevel};

impl Weapon {
    pub(in crate::weapon) fn tick_machine_gun(
        &mut self,
        player: &mut Player,
        player_id: TargetPlayer,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        const BULLETS: [u16; 3] = [10, 11, 12];

        if !player.controller.shoot() {
            self.counter1 = 6;
            self.counter2 += 1;

            if (player.equip.has_turbocharge() && self.counter2 > 1) || self.counter2 > 4 {
                self.counter2 = 0;
                self.refill_ammo(1);
            }
            return;
        }

        if bullet_manager.count_bullets_multi(&BULLETS, player_id) > 4 {
            return;
        }

        self.counter2 = 0; // recharge time counter
        self.counter1 += 1; // autofire counter

        if self.counter1 > 5 {
            self.counter1 = 0;

            let btype = match self.level {
                WeaponLevel::Level1 => 10,
                WeaponLevel::Level2 => 11,
                WeaponLevel::Level3 => 12,
                WeaponLevel::None => unreachable!(),
            };

            if !self.consume_ammo(1) {
                state.sound_manager.play_sfx(37);
                // todo spawn "empty" text
                return;
            }

            match () {
                _ if player.up => {
                    if self.level == WeaponLevel::Level3 {
                        player.vel_y += 0x100;
                    }

                    match player.direction {
                        Direction::Left => {
                            bullet_manager.create_bullet(player.x - 0x600, player.y - 0x1000, btype, player_id, Direction::Up, &state.constants);
                            state.create_caret(player.x - 0x600, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                        }
                        Direction::Right => {
                            bullet_manager.create_bullet(player.x + 0x600, player.y - 0x1000, btype, player_id, Direction::Up, &state.constants);
                            state.create_caret(player.x + 0x600, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                        }
                        _ => {}
                    }
                }
                _ if player.down => {
                    if self.level == WeaponLevel::Level3 {
                        if player.vel_y > 0 {
                            player.vel_y /= 2;
                        }
                        if player.vel_y > -0x400 {
                            player.vel_y = (player.vel_y - 0x200).max(-0x400);
                        }
                    }

                    match player.direction {
                        Direction::Left => {
                            bullet_manager.create_bullet(player.x - 0x600, player.y + 0x1000, btype, player_id, Direction::Bottom, &state.constants);
                            state.create_caret(player.x - 0x600, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                        }
                        Direction::Right => {
                            bullet_manager.create_bullet(player.x + 0x600, player.y + 0x1000, btype, player_id, Direction::Bottom, &state.constants);
                            state.create_caret(player.x + 0x600, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                        }
                        _ => {}
                    }
                }
                _ => match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0x1800, player.y + 0x600, btype, player_id, Direction::Left, &state.constants);
                        state.create_caret(player.x - 0x1800, player.y + 0x600, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0x1800, player.y + 0x600, btype, player_id, Direction::Right, &state.constants);
                        state.create_caret(player.x + 0x1800, player.y + 0x600, CaretType::Shoot, Direction::Right);
                    }
                    _ => {}
                },
            }

            if self.level == WeaponLevel::Level3 {
                state.sound_manager.play_sfx(49);
            } else {
                state.sound_manager.play_sfx(32);
            }
        }
    }
}
