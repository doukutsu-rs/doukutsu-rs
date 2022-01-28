use crate::caret::CaretType;
use crate::common::Direction;
use crate::player::{Player, TargetPlayer};
use crate::shared_game_state::SharedGameState;
use crate::weapon::bullet::BulletManager;
use crate::weapon::{Weapon, WeaponLevel};

impl Weapon {
    pub(in crate::weapon) fn tick_polar_star(
        &mut self,
        player: &Player,
        player_id: TargetPlayer,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        if !player.controller.trigger_shoot() || bullet_manager.count_bullets_multi(&[4, 5, 6], player_id) > 1 {
            return;
        }

        let btype = match self.level {
            WeaponLevel::Level1 => 4,
            WeaponLevel::Level2 => 5,
            WeaponLevel::Level3 => 6,
            WeaponLevel::None => unreachable!(),
        };

        if !self.consume_ammo(1) {
            state.sound_manager.play_sfx(37);
            return;
        }

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
                state.create_caret(player.x - 0x1800, player.y + 0x600, CaretType::Shoot, Direction::Left);
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
                state.create_caret(player.x + 0x1800, player.y + 0x600, CaretType::Shoot, Direction::Right);
            }
            _ => {}
        }

        if self.level == WeaponLevel::Level3 {
            state.sound_manager.play_sfx(49);
        } else {
            state.sound_manager.play_sfx(32);
        }
    }
}
