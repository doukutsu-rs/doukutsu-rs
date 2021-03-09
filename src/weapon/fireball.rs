use crate::caret::CaretType;
use crate::common::Direction;
use crate::player::{Player, TargetPlayer};
use crate::shared_game_state::SharedGameState;
use crate::weapon::{Weapon, WeaponLevel};
use crate::weapon::bullet::BulletManager;

impl Weapon {
    pub(in crate::weapon) fn tick_fireball(
        &mut self,
        player: &Player,
        player_id: TargetPlayer,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        let max_bullets = self.level as usize + 1;
        if player.controller.trigger_shoot() && bullet_manager.count_bullets_multi(&[7, 8, 9], player_id) < max_bullets {
            let btype = match self.level {
                WeaponLevel::Level1 => 7,
                WeaponLevel::Level2 => 8,
                WeaponLevel::Level3 => 9,
                WeaponLevel::None => {
                    unreachable!()
                }
            };

            if !self.consume_ammo(1) {
                // todo switch to first weapon
                return;
            }

            if player.up {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0x800, player.y - 0x1000, btype, player_id, Direction::Up, &state.constants);
                        state.create_caret(player.x - 0x800, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0x800, player.y - 0x1000, btype, player_id, Direction::Up, &state.constants);
                        state.create_caret(player.x + 0x800, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    _ => {}
                }
            } else if player.down {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0x800, player.y + 0x1000, btype, player_id, Direction::Bottom, &state.constants);
                        state.create_caret(player.x - 0x800, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0x800, player.y + 0x1000, btype, player_id, Direction::Bottom, &state.constants);
                        state.create_caret(player.x + 0x800, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                    }
                    _ => {}
                }
            } else {
                match player.direction {
                    Direction::Left => {
                        bullet_manager.create_bullet(player.x - 0xc00, player.y + 0x400, btype, player_id, Direction::Left, &state.constants);
                        state.create_caret(player.x - 0x1800, player.y + 0x400, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right => {
                        bullet_manager.create_bullet(player.x + 0xc00, player.y + 0x400, btype, player_id, Direction::Right, &state.constants);
                        state.create_caret(player.x + 0x1800, player.y + 0x400, CaretType::Shoot, Direction::Right);
                    }
                    _ => {}
                }
            }

            state.sound_manager.play_sfx(34)
        }
    }
}
