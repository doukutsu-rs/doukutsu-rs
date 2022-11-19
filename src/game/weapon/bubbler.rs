use crate::common::Direction;
use crate::game::caret::CaretType;
use crate::game::player::{Player, TargetPlayer};
use crate::game::shared_game_state::SharedGameState;
use crate::game::weapon::{Weapon, WeaponLevel};
use crate::game::weapon::bullet::BulletManager;

impl Weapon {
    pub(crate) fn tick_bubbler(
        &mut self,
        player: &Player,
        player_id: TargetPlayer,
        bullet_manager: &mut BulletManager,
        state: &mut SharedGameState,
    ) {
        const BULLETS: [u16; 3] = [19, 20, 21];

        if self.level == WeaponLevel::Level1 {
            if !player.controller.trigger_shoot() {
                self.counter2 += 1;
                if self.counter2 > 20 {
                    self.counter2 = 0;
                    self.refill_ammo(1);
                }

                return;
            }

            if bullet_manager.count_bullets_multi(&BULLETS, player_id) > 3 {
                return;
            }

            if !self.consume_ammo(1) {
                self.draw_empty(state, player.x, player.y);
                return;
            }

            let btype = 19;

            match player.direction {
                Direction::Left if player.up => {
                    bullet_manager.create_bullet(
                        player.x - 0x200,
                        player.y - 0x400,
                        btype,
                        player_id,
                        Direction::Up,
                        &state.constants,
                    );
                    state.create_caret(player.x - 0x200, player.y - 0x400, CaretType::Shoot, Direction::Left);
                }
                Direction::Right if player.up => {
                    bullet_manager.create_bullet(
                        player.x + 0x200,
                        player.y - 0x400,
                        btype,
                        player_id,
                        Direction::Up,
                        &state.constants,
                    );
                    state.create_caret(player.x + 0x200, player.y - 0x400, CaretType::Shoot, Direction::Left);
                }
                Direction::Left if player.down => {
                    bullet_manager.create_bullet(
                        player.x - 0x200,
                        player.y + 0x400,
                        btype,
                        player_id,
                        Direction::Bottom,
                        &state.constants,
                    );
                    state.create_caret(player.x - 0x200, player.y + 0x400, CaretType::Shoot, Direction::Left);
                }
                Direction::Right if player.down => {
                    bullet_manager.create_bullet(
                        player.x + 0x200,
                        player.y + 0x400,
                        btype,
                        player_id,
                        Direction::Bottom,
                        &state.constants,
                    );
                    state.create_caret(player.x + 0x200, player.y + 0x400, CaretType::Shoot, Direction::Left);
                }
                Direction::Left => {
                    bullet_manager.create_bullet(
                        player.x - 0x1800,
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
                        player.x + 0x1800,
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

            state.sound_manager.play_sfx(48);
        } else {
            if bullet_manager.count_bullets_multi(&BULLETS, player_id) > 15 {
                return;
            }

            let btype = if self.level == WeaponLevel::Level2 { 20 } else { 21 };

            if !player.controller.shoot() {
                self.counter1 = 6;
                self.counter2 += 1;

                if self.counter2 > 1 {
                    self.counter2 = 0;
                    self.refill_ammo(1);
                }
                return;
            }

            self.counter2 = 0; // recharge time counter
            self.counter1 += 1; // autofire counter

            if self.counter1 > 6 {
                self.counter1 = 0;

                if !self.consume_ammo(1) {
                    self.draw_empty(state, player.x, player.y);
                    return;
                }

                match player.direction {
                    Direction::Left if player.up => {
                        bullet_manager.create_bullet(
                            player.x - 0x600,
                            player.y - 0x1000,
                            btype,
                            player_id,
                            Direction::Up,
                            &state.constants,
                        );
                        state.create_caret(player.x - 0x600, player.y - 0x2000, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right if player.up => {
                        bullet_manager.create_bullet(
                            player.x + 0x600,
                            player.y - 0x1000,
                            btype,
                            player_id,
                            Direction::Up,
                            &state.constants,
                        );
                        state.create_caret(player.x + 0x600, player.y - 0x2000, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Left if player.down => {
                        bullet_manager.create_bullet(
                            player.x - 0x600,
                            player.y + 0x1000,
                            btype,
                            player_id,
                            Direction::Bottom,
                            &state.constants,
                        );
                        state.create_caret(player.x - 0x600, player.y + 0x2000, CaretType::Shoot, Direction::Left);
                    }
                    Direction::Right if player.down => {
                        bullet_manager.create_bullet(
                            player.x + 0x600,
                            player.y + 0x1000,
                            btype,
                            player_id,
                            Direction::Bottom,
                            &state.constants,
                        );
                        state.create_caret(player.x + 0x600, player.y + 0x2000, CaretType::Shoot, Direction::Left);
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

                state.sound_manager.play_sfx(48);
            }
        }
    }
}
