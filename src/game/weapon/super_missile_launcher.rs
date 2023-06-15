use crate::common::Direction;
use crate::game::caret::CaretType;
use crate::game::player::{Player, TargetPlayer};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::{Bullet, BulletManager};

impl Weapon {
    pub(crate) fn tick_super_missile_launcher(
        &mut self,
        player: &mut Player,
        player_id: TargetPlayer,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        const BULLETS: [u16; 6] = [28, 29, 30, 31, 32, 33];

        if !player.controller.trigger_shoot() {
            return;
        }

        let btype = match self.level {
            WeaponLevel::Level1 => 28,
            WeaponLevel::Level2 => 29,
            WeaponLevel::Level3 => 30,
            WeaponLevel::None => unreachable!(),
        };

        match self.level {
            WeaponLevel::Level1 if bullet_manager.count_bullets_multi(&BULLETS, player_id) > 0 => {
                return;
            }
            WeaponLevel::Level2 if bullet_manager.count_bullets_multi(&BULLETS, player_id) > 1 => {
                return;
            }
            WeaponLevel::Level3 if bullet_manager.count_bullets_multi(&BULLETS, player_id) > 3 => {
                return;
            }
            _ => {}
        }

        if !self.consume_ammo(1) {
            self.draw_empty(state, player.x, player.y);
            return;
        }

        match player.direction {
            Direction::Left if player.up => {
                let mut bullet =
                    Bullet::new(player.x - 0x200, player.y - 0x1000, btype, player_id, Direction::Up, &state.constants);

                bullet_manager.push_bullet(bullet.clone());

                state.create_caret(player.x - 0x200, player.y - 0x1000, CaretType::Shoot, Direction::Left);

                if self.level == WeaponLevel::Level3 {
                    bullet.x = player.x + 0x600;
                    bullet.y = player.y;
                    bullet.counter2 = 1;
                    bullet_manager.push_bullet(bullet.clone());
                    
                    bullet.x = player.x - 0x600;
                    bullet.counter2 = 2;
                    bullet_manager.push_bullet(bullet);
                }
            }
            Direction::Right if player.up => {
                let mut bullet =
                    Bullet::new(player.x + 0x200, player.y - 0x1000, btype, player_id, Direction::Up, &state.constants);

                bullet_manager.push_bullet(bullet.clone());

                state.create_caret(player.x + 0x200, player.y - 0x1000, CaretType::Shoot, Direction::Left);

                if self.level == WeaponLevel::Level3 {
                    bullet.x = player.x + 0x600;
                    bullet.y = player.y;
                    bullet.counter2 = 1;
                    bullet_manager.push_bullet(bullet.clone());

                    bullet.x = player.x - 0x600;
                    bullet.counter2 = 2;
                    bullet_manager.push_bullet(bullet);
                }
            }
            Direction::Left if player.down => {
                let mut bullet = Bullet::new(
                    player.x - 0x200,
                    player.y + 0x1000,
                    btype,
                    player_id,
                    Direction::Bottom,
                    &state.constants,
                );

                bullet_manager.push_bullet(bullet.clone());

                state.create_caret(player.x - 0x200, player.y + 0x1000, CaretType::Shoot, Direction::Left);

                if self.level == WeaponLevel::Level3 {
                    bullet.x = player.x + 0x600;
                    bullet.y = player.y;
                    bullet.counter2 = 1;
                    bullet_manager.push_bullet(bullet.clone());

                    bullet.x = player.x - 0x600;
                    bullet.counter2 = 2;
                    bullet_manager.push_bullet(bullet);
                }
            }
            Direction::Right if player.down => {
                let mut bullet = Bullet::new(
                    player.x + 0x200,
                    player.y + 0x1000,
                    btype,
                    player_id,
                    Direction::Bottom,
                    &state.constants,
                );

                bullet_manager.push_bullet(bullet.clone());

                state.create_caret(player.x + 0x200, player.y + 0x1000, CaretType::Shoot, Direction::Left);

                if self.level == WeaponLevel::Level3 {
                    bullet.x = player.x - 0x600;
                    bullet.y = player.y;
                    bullet.counter2 = 1;
                    bullet_manager.push_bullet(bullet.clone());

                    bullet.x = player.x + 0x600;
                    bullet.counter2 = 2;
                    bullet_manager.push_bullet(bullet);
                }
            }
            Direction::Left => {
                let yoffset = (self.level == WeaponLevel::Level3) as i32 * 0x200;
                let mut bullet =
                    Bullet::new(player.x - 0xc00, player.y + yoffset, btype, player_id, Direction::Left, &state.constants);

                bullet_manager.push_bullet(bullet.clone());

                state.create_caret(player.x - 0x1800, player.y + yoffset, CaretType::Shoot, Direction::Left);

                if self.level == WeaponLevel::Level3 {
                    bullet.x = player.x;
                    bullet.y = player.y - 0x1000;
                    bullet.counter2 = 1;
                    bullet_manager.push_bullet(bullet.clone());

                    bullet.x = player.x + 0x800;
                    bullet.y = player.y - 0x200;
                    bullet.counter2 = 2;
                    bullet_manager.push_bullet(bullet);
                }
            }
            Direction::Right => {
                let yoffset = (self.level == WeaponLevel::Level3) as i32 * 0x200;
                let mut bullet =
                    Bullet::new(player.x + 0xc00, player.y + yoffset, btype, player_id, Direction::Right, &state.constants);

                bullet_manager.push_bullet(bullet.clone());

                state.create_caret(player.x + 0x1800, player.y + yoffset, CaretType::Shoot, Direction::Left);

                if self.level == WeaponLevel::Level3 {
                    bullet.x = player.x;
                    bullet.y = player.y - 0x1000;
                    bullet.counter2 = 1;
                    bullet_manager.push_bullet(bullet.clone());

                    bullet.x = player.x - 0x800;
                    bullet.y = player.y - 0x200;
                    bullet.counter2 = 2;
                    bullet_manager.push_bullet(bullet);
                }
            }
            _ => {}
        }

        state.sound_manager.play_sfx(32)
    }
}
