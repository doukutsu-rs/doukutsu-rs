use crate::common::Direction;
use crate::game::caret::CaretType;
use crate::game::player::{Player, TargetPlayer};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::BulletManager;

impl Weapon {
    pub(crate) fn tick_nemesis(
        &mut self,
        player: &Player,
        player_id: TargetPlayer,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        const BULLETS: [u16; 3] = [34, 35, 36];

        if !player.controller.trigger_shoot() || bullet_manager.count_bullets_multi(&BULLETS, player_id) > 1 {
            return;
        }

        let btype = match self.level {
            WeaponLevel::Level1 => 34,
            WeaponLevel::Level2 => 35,
            WeaponLevel::Level3 => 36,
            WeaponLevel::None => unreachable!(),
        };

        if !self.consume_ammo(1) {
            state.sound_manager.play_sfx(37);
            // todo spawn "empty" text
            return;
        }

        if player.up {
            match player.direction {
                Direction::Left => {
                    bullet_manager.create_bullet(player.x - 0x200, player.y - 0x1800, btype, player_id, Direction::Up, &state.constants);
                    state.create_caret(player.x - 0x200, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                }
                Direction::Right => {
                    bullet_manager.create_bullet(player.x + 0x200, player.y - 0x1800, btype, player_id, Direction::Up, &state.constants);
                    state.create_caret(player.x + 0x200, player.y - 0x1000, CaretType::Shoot, Direction::Left);
                }
                _ => {}
            }
        } else if player.down {
            match player.direction {
                Direction::Left => {
                    bullet_manager.create_bullet(player.x - 0x200, player.y + 0x1800, btype, player_id, Direction::Bottom, &state.constants);
                    state.create_caret(player.x - 0x200, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                }
                Direction::Right => {
                    bullet_manager.create_bullet(player.x + 0x200, player.y + 0x1800, btype, player_id, Direction::Bottom, &state.constants);
                    state.create_caret(player.x + 0x200, player.y + 0x1000, CaretType::Shoot, Direction::Left);
                }
                _ => {}
            }
        } else {
            match player.direction {
                Direction::Left => {
                    bullet_manager.create_bullet(player.x - 0x2c00, player.y + 0x600, btype, player_id, Direction::Left, &state.constants);
                    state.create_caret(player.x - 0x2000, player.y + 0x600, CaretType::Shoot, Direction::Left);
                }
                Direction::Right => {
                    bullet_manager.create_bullet(player.x + 0x2c00, player.y + 0x600, btype, player_id, Direction::Right, &state.constants);
                    state.create_caret(player.x + 0x2000, player.y + 0x600, CaretType::Shoot, Direction::Right);
                }
                _ => {}
            }
        }

        match self.level {
            WeaponLevel::Level1 => state.sound_manager.play_sfx(117),
            WeaponLevel::Level2 => state.sound_manager.play_sfx(49),
            WeaponLevel::Level3 => state.sound_manager.play_sfx(60),
            _ => unreachable!(),
        }
    }
}
