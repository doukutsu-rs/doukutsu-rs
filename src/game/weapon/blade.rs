use crate::common::Direction;
use crate::game::player::{Player, TargetPlayer};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::BulletManager;

impl Weapon {
    pub(crate) fn tick_blade(&mut self, player: &Player, player_id: TargetPlayer, bullet_manager: &mut BulletManager, state: &mut SharedGameState) {
        const BULLETS: [u16; 3] = [25, 26, 27];

        if !player.controller.trigger_shoot() || bullet_manager.count_bullets_multi(&BULLETS, player_id) > 0 {
            return;
        }

        let btype = match self.level {
            WeaponLevel::Level1 => 25,
            WeaponLevel::Level2 => 26,
            WeaponLevel::Level3 => 27,
            WeaponLevel::None => unreachable!(),
        };

        match player.direction {
            Direction::Left if player.up => {
                bullet_manager.create_bullet(player.x - 0x200, player.y + 0x800, btype, player_id, Direction::Up, &state.constants);
            }
            Direction::Right if player.up => {
                bullet_manager.create_bullet(player.x + 0x200, player.y + 0x800, btype, player_id, Direction::Up, &state.constants);
            }
            Direction::Left if player.down => {
                bullet_manager.create_bullet(player.x - 0x200, player.y - 0xc00, btype, player_id, Direction::Bottom, &state.constants);
            }
            Direction::Right if player.down => {
                bullet_manager.create_bullet(player.x + 0x200, player.y - 0xc00, btype, player_id, Direction::Bottom, &state.constants);
            }
            Direction::Left => {
                bullet_manager.create_bullet(player.x + 0xc00, player.y - 0x600, btype, player_id, Direction::Left, &state.constants);
            }
            Direction::Right => {
                bullet_manager.create_bullet(player.x - 0xc00, player.y - 0x600, btype, player_id, Direction::Right, &state.constants);
            }
            _ => {}
        }

        state.sound_manager.play_sfx(34);
    }
}
