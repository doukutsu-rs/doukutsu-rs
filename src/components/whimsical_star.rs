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
    pub tex: String,
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
    pub rect: Rect<u16>,
}

impl Star {
    fn new(vel_x: i32, vel_y: i32) -> Star {
        Star { x: 0, y: 0, vel_x, vel_y, prev_x: 0, prev_y: 0, rect: Rect::new(0, 0, 0, 0) }
    }
}

impl WhimsicalStar {
    pub fn new() -> WhimsicalStar {
        WhimsicalStar {
            star: [Star::new(0x400, -0x200), Star::new(-0x200, 0x400), Star::new(0x200, 0x200)],
            tex: "MyChar".to_string(),
            star_count: 0,
            equipped: false,
            active_star: 0,
        }
    }

    pub fn init(&mut self, player: &Player) {
        self.tex = player.skin.get_skin_texture_name().to_string();
        for (iter, star) in &mut self.star.iter_mut().enumerate() {
            star.rect = player.skin.get_whimsical_star_rect(iter);
        }
    }

    pub fn set_prev(&mut self) {
        for star in &mut self.star {
            star.prev_x = star.x;
            star.prev_y = star.y;
        }
    }
}

impl GameEntity<(&Player, &mut BulletManager)> for WhimsicalStar {
    fn tick(
        &mut self,
        state: &mut SharedGameState,
        (player, bullet_manager): (&Player, &mut BulletManager),
    ) -> GameResult {
        if !self.equipped && player.equip.has_whimsical_star() {
            for star in &mut self.star {
                star.x = player.x;
                star.y = player.y;
            }
            self.equipped = true;
        }

        if !player.equip.has_whimsical_star() {
            self.equipped = false;
            return Ok(());
        }

        self.star_count = player.stars;

        let mut prev_x = player.x;
        let mut prev_y = player.y;

        for star in &mut self.star {
            star.vel_x += if prev_x >= star.x { 0x80 } else { -0x80 };
            star.vel_y += if prev_y >= star.y { 0xAA } else { -0xAA };

            star.vel_x = star.vel_x.clamp(-0xA00, 0xA00);
            star.vel_y = star.vel_y.clamp(-0xA00, 0xA00);

            star.x += star.vel_x;
            star.y += star.vel_y;

            prev_x = star.x;
            prev_y = star.y;
        }

        // Only one star can deal damage per tick
        self.active_star += 1;
        self.active_star %= 3;

        if self.active_star < self.star_count && state.control_flags.control_enabled() {
            let bullet = Bullet::new(
                self.star[self.active_star as usize].x,
                self.star[self.active_star as usize].y,
                45,
                TargetPlayer::Player1,
                Direction::Left,
                &state.constants,
            );
            bullet_manager.push_bullet(bullet);
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, frame: &Frame) -> GameResult {
        if !self.equipped {
            return Ok(());
        }

        let (frame_x, frame_y) = frame.xy_interpolated(state.frame_time);

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, &self.tex)?;

        let (active_stars, _) = self.star.split_at(self.star_count as usize);

        for star in active_stars {
            let x = interpolate_fix9_scale(star.prev_x as i32, star.x as i32, state.frame_time) - frame_x;
            let y = interpolate_fix9_scale(star.prev_y as i32, star.y as i32, state.frame_time) - frame_y;
            batch.add_rect(x, y, &star.rect);
        }

        batch.draw(ctx)?;

        Ok(())
    }
}
