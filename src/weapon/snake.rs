use crate::caret::CaretType;
use crate::common::Direction;
use crate::player::{Player, TargetPlayer};
use crate::shared_game_state::SharedGameState;
use crate::weapon::bullet::{Bullet, BulletManager};
use crate::weapon::{Weapon, WeaponLevel};

impl Weapon {
    pub(in crate::weapon) fn tick_snake(&mut self, player: &Player, player_id: TargetPlayer, bullet_manager: &mut BulletManager, state: &mut SharedGameState) {
        if !player.controller.trigger_shoot() || bullet_manager.count_bullets_multi(&[1, 2, 3], player_id) > 3 {
            return;
        }

        let btype = match self.level {
            WeaponLevel::Level1 => 1,
            WeaponLevel::Level2 => 2,
            WeaponLevel::Level3 => 3,
            WeaponLevel::None => unreachable!(),
        };

        if !self.consume_ammo(1) {
            // todo switch to first weapon
            return;
        }

        self.counter1 = self.counter1.wrapping_add(1);

        match player.direction {
            Direction::Left if player.up => {
                let mut bullet = Bullet::new(player.x - 0x600, player.y - 10 * 0x200, btype, player_id, Direction::Up, &state.constants);
                bullet.target_x = self.counter1 as i32;
                bullet_manager.push_bullet(bullet);
                state.create_caret(player.x - 0x600, player.y - 10 * 0x200, CaretType::Shoot, Direction::Left);
            }
            Direction::Right if player.up => {
                let mut bullet = Bullet::new(player.x + 0x600, player.y - 10 * 0x200, btype, player_id, Direction::Up, &state.constants);
                bullet.target_x = self.counter1 as i32;
                bullet_manager.push_bullet(bullet);
                state.create_caret(player.x + 0x600, player.y - 10 * 0x200, CaretType::Shoot, Direction::Left);
            }
            Direction::Left if player.down => {
                let mut bullet = Bullet::new(player.x - 0x600, player.y + 10 * 0x200, btype, player_id, Direction::Bottom, &state.constants);
                bullet.target_x = self.counter1 as i32;
                bullet_manager.push_bullet(bullet);
                state.create_caret(player.x - 0x600, player.y + 10 * 0x200, CaretType::Shoot, Direction::Left);
            }
            Direction::Right if player.down => {
                let mut bullet = Bullet::new(player.x + 0x600, player.y + 10 * 0x200, btype, player_id, Direction::Bottom, &state.constants);
                bullet.target_x = self.counter1 as i32;
                bullet_manager.push_bullet(bullet);
                state.create_caret(player.x + 0x600, player.y + 10 * 0x200, CaretType::Shoot, Direction::Left);
            }
            Direction::Left => {
                let mut bullet = Bullet::new(player.x - 0xc00, player.y + 0x400, btype, player_id, Direction::Left, &state.constants);
                bullet.target_x = self.counter1 as i32;
                bullet_manager.push_bullet(bullet);
                state.create_caret(player.x - 0x1800, player.y + 0x400, CaretType::Shoot, Direction::Left);
            }
            Direction::Right => {
                let mut bullet = Bullet::new(player.x + 0xc00, player.y + 0x400, btype, player_id, Direction::Right, &state.constants);
                bullet.target_x = self.counter1 as i32;
                bullet_manager.push_bullet(bullet);
                state.create_caret(player.x + 0x1800, player.y + 0x400, CaretType::Shoot, Direction::Right);
            }
            _ => {}
        }

        state.sound_manager.play_sfx(33);
    }
}
