use crate::common::{interpolate_fix9_scale, Direction, Rect};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::player::{Player, TargetPlayer};
use crate::shared_game_state::SharedGameState;
use crate::weapon::bullet::{Bullet, BulletManager};

pub struct WhimsicalStar {
    pub star: [Star; 3],
    pub star_count: u8,
    pub equipped: bool,
    pub active_star: u8,
}

pub struct Star {
    pub x: i32,
    pub y: i32,
    pub prev_x: i32,
    pub prev_y: i32,
    pub vel_x: i32,
    pub vel_y: i32,
}

impl Star {
    fn new(vel_x: i32, vel_y: i32) -> Star {
        Star { x: 0, y: 0, vel_x, vel_y, prev_x: 0, prev_y: 0 }
    }
}

impl WhimsicalStar {
    pub fn new() -> WhimsicalStar {
        WhimsicalStar {
            star: [Star::new(0x400, -0x200), Star::new(-0x200, 0x400), Star::new(0x200, 0x200)],
            star_count: 0,
            equipped: false,
            active_star: 0,
        }
    }

    pub fn set_prev(&mut self) {
        for iter in 0..=2 {
            self.star[iter].prev_x = self.star[iter].x;
            self.star[iter].prev_y = self.star[iter].y;
        }
    }
}

impl GameEntity<(&Player, &mut BulletManager)> for WhimsicalStar {
    fn tick(
        &mut self,
        state: &mut SharedGameState,
        (player, bullet_manager): (&Player, &mut BulletManager),
    ) -> GameResult {
        if !player.equip.has_whimsical_star() {
            return Ok(());
        } else if !self.equipped && player.equip.has_whimsical_star() {
            for iter in 0..2 {
                self.star[iter].x = player.x;
                self.star[iter].y = player.y;
            }
            self.equipped = true;
        } else {
            self.equipped = player.equip.has_whimsical_star();
        }

        self.star_count = player.stars;

        // Only one star can deal damage per tick
        self.active_star += 1;
        self.active_star %= 3;

        for iter in 0..3 {
            if iter != 0 {
                self.star[iter].vel_x += if self.star[iter - 1].x >= self.star[iter].x { 0x80 } else { -0x80 };
                self.star[iter].vel_y += if self.star[iter - 1].y >= self.star[iter].y { 0xAA } else { -0xAA };
            } else {
                self.star[iter].vel_x += if player.x >= self.star[iter].x { 0x80 } else { -0x80 };
                self.star[iter].vel_y += if player.y >= self.star[iter].y { 0xAA } else { -0xAA };
            }

            self.star[iter].vel_x = self.star[iter].vel_x.clamp(-0xA00, 0xA00);
            self.star[iter].vel_y = self.star[iter].vel_y.clamp(-0xA00, 0xA00);

            self.star[iter].x += self.star[iter].vel_x;
            self.star[iter].y += self.star[iter].vel_y;

            if iter < self.star_count as usize
                && state.control_flags.control_enabled()
                && self.active_star == iter as u8
            {
                let bullet = Bullet::new(
                    self.star[iter].x,
                    self.star[iter].y,
                    45,
                    TargetPlayer::Player1,
                    Direction::Left,
                    &state.constants,
                );
                bullet_manager.push_bullet(bullet);
            }
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult {
        if !self.equipped {
            return Ok(());
        }

        let (frame_x, frame_y) = frame.xy_interpolated(state.frame_time);

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "MyChar")?;

        const STAR_RECTS: [Rect<u16>; 3] = [
            Rect { left: 192, top: 0, right: 200, bottom: 8 },
            Rect { left: 192, top: 8, right: 200, bottom: 16 },
            Rect { left: 192, top: 16, right: 200, bottom: 24 },
        ];

        for iter in 0..self.star_count as usize {
            let x = interpolate_fix9_scale(self.star[iter].prev_x as i32, self.star[iter].x as i32, state.frame_time)
                - frame_x;
            let y = interpolate_fix9_scale(self.star[iter].prev_y as i32, self.star[iter].y as i32, state.frame_time)
                - frame_y;
            batch.add_rect(x, y, &STAR_RECTS[iter]);
        }

        batch.draw(ctx)?;

        Ok(())
    }
}
