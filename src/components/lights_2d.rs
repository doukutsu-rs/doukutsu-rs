use std::ops::Range;

use crate::{
    common::{interpolate_fix9_scale, Color, Direction, Rect},
    framework::{
        context::Context,
        error::GameResult,
        graphics::{self, BlendMode, FilterMode},
        render::sprite_batch::SpriteBatchCommand,
    },
    game::{
        caret::CaretType,
        physics::PhysicalEntity,
        shared_game_state::{SharedGameState, TileSize},
        weapon::{Weapon, WeaponType},
    },
    graphics::texture_set::SpriteBatch,
    scene::game_scene::GameScene,
};

pub struct Lights2D;

impl Lights2D {
    pub fn new() -> Self {
        Self
    }

    fn draw_light(&self, x: f32, y: f32, size: f32, color: (u8, u8, u8), batch: &mut Box<dyn SpriteBatch>) {
        batch.add_rect_scaled_tinted(
            x - size * 32.0,
            y - size * 32.0,
            (color.0, color.1, color.2, 255),
            size,
            size,
            &Rect::new(0, 0, 64, 64),
        )
    }

    fn draw_light_raycast(
        &self,
        tile_size: TileSize,
        world_point_x: i32,
        world_point_y: i32,
        (br, bg, bb): (u8, u8, u8),
        att: f32,
        angle: Range<i32>,
        batch: &mut Box<dyn SpriteBatch>,
        game_scene: &GameScene,
    ) {
        let px = world_point_x as f32 / 512.0;
        let py = world_point_y as f32 / 512.0;

        let fx2 = game_scene.frame.x as f32 / 512.0;
        let fy2 = game_scene.frame.y as f32 / 512.0;

        let ti = tile_size.as_int();
        let tf = tile_size.as_float();
        let tih = ti / 2;
        let tfq = tf / 4.0;
        let (br, bg, bb) = (br as f32, bg as f32, bb as f32);
        let ahalf = (angle.end - angle.start) as f32 / 2.0;

        'ray: for (i, deg) in angle.enumerate() {
            let d = deg as f32 * (std::f32::consts::PI / 180.0);
            let dx = d.cos() * -5.0;
            let dy = d.sin() * -5.0;
            let m = 1.0 - ((ahalf - i as f32).abs() / ahalf);
            let mut x = px;
            let mut y = py;
            let mut r = br;
            let mut g = bg;
            let mut b = bb;

            for i in 0..40 {
                x += dx;
                y += dy;

                const ARR: [(i32, i32); 4] = [(0, 0), (0, 1), (1, 0), (1, 1)];
                for (ox, oy) in ARR.iter() {
                    let bx = (x as i32).wrapping_div(ti).wrapping_add(*ox);
                    let by = (y as i32).wrapping_div(ti).wrapping_add(*oy);

                    let tile = game_scene.stage.map.attrib[game_scene.stage.tile_at(bx as usize, by as usize) as usize];
                    let bxmth = (bx * ti - tih) as f32;
                    let bxpth = (bx * ti + tih) as f32;
                    let bymth = (by * ti - tih) as f32;
                    let bypth = (by * ti + tih) as f32;

                    if ((tile == 0x62 || tile == 0x41 || tile == 0x43 || tile == 0x46)
                        && x >= bxmth
                        && x <= bxpth
                        && y >= bymth
                        && y <= bypth)
                        || ((tile == 0x50 || tile == 0x70)
                            && x >= bxmth
                            && x <= bxpth
                            && y <= ((by as f32 * tf) - (x - bx as f32 * tf) / 2.0 + tfq)
                            && y >= bymth)
                        || ((tile == 0x51 || tile == 0x71)
                            && x >= bxmth
                            && x <= bxpth
                            && y <= ((by as f32 * tf) - (x - bx as f32 * tf) / 2.0 - tfq)
                            && y >= bymth)
                        || ((tile == 0x52 || tile == 0x72)
                            && x >= bxmth
                            && x <= bxpth
                            && y <= ((by as f32 * tf) + (x - bx as f32 * tf) / 2.0 - tfq)
                            && y >= bymth)
                        || ((tile == 0x53 || tile == 0x73)
                            && x >= bxmth
                            && x <= bxpth
                            && y <= ((by as f32 * tf) + (x - bx as f32 * tf) / 2.0 + tfq)
                            && y >= bymth)
                        || ((tile == 0x54 || tile == 0x74)
                            && x >= bxmth
                            && x <= bxpth
                            && y >= ((by as f32 * tf) + (x - bx as f32 * tf) / 2.0 - tfq)
                            && y <= bypth)
                        || ((tile == 0x55 || tile == 0x75)
                            && x >= bxmth
                            && x <= bxpth
                            && y >= ((by as f32 * tf) + (x - bx as f32 * tf) / 2.0 + tfq)
                            && y <= bypth)
                        || ((tile == 0x56 || tile == 0x76)
                            && x >= bxmth
                            && x <= bxpth
                            && y >= ((by as f32 * tf) - (x - bx as f32 * tf) / 2.0 + tfq)
                            && y <= bypth)
                        || ((tile == 0x57 || tile == 0x77)
                            && x >= bxmth
                            && x <= bxpth
                            && y >= ((by as f32 * tf) - (x - bx as f32 * tf) / 2.0 - tfq)
                            && y <= bypth)
                    {
                        continue 'ray;
                    }
                }

                r *= att;
                g *= att;
                b *= att;

                if r <= 1.0 && g <= 1.0 && b <= 1.0 {
                    continue 'ray;
                }

                self.draw_light(
                    x - fx2,
                    y - fy2,
                    0.15 + i as f32 / 75.0,
                    ((r * m) as u8, (g * m) as u8, (b * m) as u8),
                    batch,
                );
            }
        }
    }

    pub fn draw_light_map(&self, state: &mut SharedGameState, ctx: &mut Context, game_scene: &GameScene) -> GameResult {
        {
            let maybe_canvas = state.lightmap_canvas.as_ref();

            if let Some(canvas) = maybe_canvas {
                graphics::set_render_target(ctx, Some(canvas.texture_ref()))?;
            } else {
                return Ok(());
            }
        }

        graphics::set_blend_mode(ctx, BlendMode::Add)?;

        graphics::clear(ctx, Color::from_rgb(100, 100, 110));

        for npc in game_scene.npc_list.iter_alive(&game_scene.npc_token) {
            if npc.x < (game_scene.frame.x - 128 * 0x200 - npc.display_bounds.width() as i32 * 0x200)
                || npc.x
                    > (game_scene.frame.x
                        + 128 * 0x200
                        + (state.canvas_size.0 as i32 + npc.display_bounds.width() as i32) * 0x200)
                    && npc.y < (game_scene.frame.y - 128 * 0x200 - npc.display_bounds.height() as i32 * 0x200)
                || npc.y
                    > (game_scene.frame.y
                        + 128 * 0x200
                        + (state.canvas_size.1 as i32 + npc.display_bounds.height() as i32) * 0x200)
            {
                continue;
            }

            npc.draw_lightmap(state, ctx, &game_scene.frame)?;
        }

        {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "builtin/lightmap/spot")?;

            'cc: for (player, inv) in [
                (&game_scene.player1, &game_scene.inventory_player1),
                (&game_scene.player2, &game_scene.inventory_player2),
            ]
            .iter()
            {
                if player.cond.alive() && !player.cond.hidden() && inv.get_current_weapon().is_some() {
                    if state.settings.light_cone {
                        let range = match () {
                            _ if player.up => 60..120,
                            _ if player.down => 240..300,
                            _ if player.direction == Direction::Left => -30..30,
                            _ if player.direction == Direction::Right => 150..210,
                            _ => continue 'cc,
                        };

                        let (color, att) = match inv.get_current_weapon() {
                            Some(Weapon { wtype: WeaponType::Fireball, .. }) => ((170u8, 80u8, 0u8), 0.92),
                            Some(Weapon { wtype: WeaponType::PolarStar, .. }) => ((150u8, 150u8, 160u8), 0.92),
                            Some(Weapon { wtype: WeaponType::Spur, .. }) => ((170u8, 170u8, 200u8), 0.92),
                            Some(Weapon { wtype: WeaponType::Blade, .. }) => continue 'cc,
                            _ => ((150u8, 150u8, 150u8), 0.92),
                        };

                        let (_, gun_off_y) = player.skin.get_gun_offset();

                        self.draw_light_raycast(
                            state.tile_size,
                            player.x + player.direction.vector_x() * 0x800,
                            player.y + gun_off_y * 0x200 + 0x400,
                            color,
                            att,
                            range,
                            batch,
                            game_scene,
                        );
                    } else {
                        self.draw_light(
                            interpolate_fix9_scale(
                                player.prev_x - game_scene.frame.prev_x,
                                player.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                player.prev_y - game_scene.frame.prev_y,
                                player.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            5.0,
                            (150, 150, 150),
                            batch,
                        );
                    }
                }
            }

            for bullet in game_scene.bullet_manager.bullets.iter() {
                self.draw_light(
                    interpolate_fix9_scale(
                        bullet.prev_x - game_scene.frame.prev_x,
                        bullet.x - game_scene.frame.x,
                        state.frame_time,
                    ),
                    interpolate_fix9_scale(
                        bullet.prev_y - game_scene.frame.prev_y,
                        bullet.y - game_scene.frame.y,
                        state.frame_time,
                    ),
                    0.3,
                    (200, 200, 200),
                    batch,
                );
            }

            for caret in state.carets.iter() {
                match caret.ctype {
                    CaretType::ProjectileDissipation | CaretType::Shoot => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                caret.prev_x - game_scene.frame.prev_x,
                                caret.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                caret.prev_y - game_scene.frame.prev_y,
                                caret.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            0.5,
                            (150, 150, 150),
                            batch,
                        );
                    }
                    _ => {}
                }
            }

            for npc in game_scene.npc_list.iter_alive(&game_scene.npc_token) {
                if npc.cond.hidden()
                    || (npc.x < (game_scene.frame.x - 128 * 0x200 - npc.display_bounds.width() as i32 * 0x200)
                        || npc.x
                            > (game_scene.frame.x
                                + 128 * 0x200
                                + (state.canvas_size.0 as i32 + npc.display_bounds.width() as i32) * 0x200)
                            && npc.y < (game_scene.frame.y - 128 * 0x200 - npc.display_bounds.height() as i32 * 0x200)
                        || npc.y
                            > (game_scene.frame.y
                                + 128 * 0x200
                                + (state.canvas_size.1 as i32 + npc.display_bounds.height() as i32) * 0x200))
                {
                    continue;
                }

                // NPC lighting
                match npc.npc_type {
                    1 => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            0.33,
                            (255, 255, 50),
                            batch,
                        );
                    }
                    4 if npc.direction == Direction::Up => self.draw_light(
                        interpolate_fix9_scale(
                            npc.prev_x - game_scene.frame.prev_x,
                            npc.x - game_scene.frame.x,
                            state.frame_time,
                        ),
                        interpolate_fix9_scale(
                            npc.prev_y - game_scene.frame.prev_y,
                            npc.y - game_scene.frame.y,
                            state.frame_time,
                        ),
                        1.0,
                        (200, 100, 0),
                        batch,
                    ),
                    7 => self.draw_light(
                        interpolate_fix9_scale(
                            npc.prev_x - game_scene.frame.prev_x,
                            npc.x - game_scene.frame.x,
                            state.frame_time,
                        ),
                        interpolate_fix9_scale(
                            npc.prev_y - game_scene.frame.prev_y,
                            npc.y - game_scene.frame.y,
                            state.frame_time,
                        ),
                        1.0,
                        (100, 100, 100),
                        batch,
                    ),
                    17 if npc.anim_num == 0 => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            1.25,
                            (100, 0, 0),
                            batch,
                        );
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            0.5,
                            (255, 10, 10),
                            batch,
                        );
                    }
                    20 if npc.direction == Direction::Right => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            1.5,
                            (30, 30, 130),
                            batch,
                        );

                        if npc.anim_num < 2 {
                            self.draw_light(
                                interpolate_fix9_scale(
                                    npc.prev_x - game_scene.frame.prev_x,
                                    npc.x - game_scene.frame.x,
                                    state.frame_time,
                                ),
                                interpolate_fix9_scale(
                                    npc.prev_y - game_scene.frame.prev_y,
                                    npc.y - game_scene.frame.y,
                                    state.frame_time,
                                ),
                                1.0,
                                (0, 0, 20),
                                batch,
                            );
                        }
                    }
                    22 if npc.action_num == 1 && npc.anim_num == 1 => self.draw_light(
                        interpolate_fix9_scale(
                            npc.prev_x - game_scene.frame.prev_x,
                            npc.x - game_scene.frame.x,
                            state.frame_time,
                        ),
                        interpolate_fix9_scale(
                            npc.prev_y - game_scene.frame.prev_y,
                            npc.y - game_scene.frame.y,
                            state.frame_time,
                        ),
                        3.0,
                        (0, 0, 255),
                        batch,
                    ),
                    32 | 87 => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            0.75,
                            (255, 30, 30),
                            batch,
                        );
                    }
                    211 => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            1.0,
                            (90, 0, 0),
                            batch,
                        );
                    }
                    27 => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ) + 0.5,
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            3.0,
                            (96, 0, 0),
                            batch,
                        );
                    }
                    38 => {
                        let flicker = ((npc.anim_num.wrapping_add(npc.id) ^ 5) & 3) as u8 * 24;
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            3.5,
                            (150 + flicker, 60 + flicker, 0),
                            batch,
                        );
                    }
                    69 | 81 => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            if npc.npc_type == 69 { 0.5 } else { 1.0 },
                            (200, 200, 200),
                            batch,
                        );
                    }
                    70 => {
                        let flicker = 50 + npc.anim_num as u8 * 15;
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            2.0,
                            (flicker, flicker, flicker),
                            batch,
                        );
                    }
                    85 if npc.action_num == 1 => {
                        let (color, color2) = if npc.direction == Direction::Left {
                            if state.constants.is_cs_plus {
                                ((20, 100, 20), (20, 50, 20))
                            } else {
                                ((20, 20, 100), (20, 20, 50))
                            }
                        } else {
                            ((150, 0, 0), (50, 0, 0))
                        };

                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            0.75,
                            color,
                            batch,
                        );

                        if npc.anim_num < 2 && npc.direction == Direction::Right {
                            self.draw_light(
                                interpolate_fix9_scale(
                                    npc.prev_x - game_scene.frame.prev_x,
                                    npc.x - game_scene.frame.x,
                                    state.frame_time,
                                ),
                                interpolate_fix9_scale(
                                    npc.prev_y - game_scene.frame.prev_y,
                                    npc.y - game_scene.frame.y,
                                    state.frame_time,
                                ) - 8.0,
                                2.1,
                                color2,
                                batch,
                            );
                        }
                    }
                    101 | 102 => self.draw_light(
                        interpolate_fix9_scale(
                            npc.prev_x - game_scene.frame.prev_x,
                            npc.x - game_scene.frame.x,
                            state.frame_time,
                        ),
                        interpolate_fix9_scale(
                            npc.prev_y - game_scene.frame.prev_y,
                            npc.y - game_scene.frame.y,
                            state.frame_time,
                        ),
                        1.0,
                        (100, 100, 200),
                        batch,
                    ),
                    175 if npc.action_num < 10 => {
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            1.0,
                            (128, 175, 200),
                            batch,
                        );
                    }
                    189 => self.draw_light(
                        interpolate_fix9_scale(
                            npc.prev_x - game_scene.frame.prev_x,
                            npc.x - game_scene.frame.x,
                            state.frame_time,
                        ),
                        interpolate_fix9_scale(
                            npc.prev_y - game_scene.frame.prev_y,
                            npc.y - game_scene.frame.y,
                            state.frame_time,
                        ),
                        1.0,
                        (10, 50, 255),
                        batch,
                    ),
                    270 => self.draw_light(
                        interpolate_fix9_scale(
                            npc.prev_x - game_scene.frame.prev_x,
                            npc.x - game_scene.frame.x,
                            state.frame_time,
                        ),
                        interpolate_fix9_scale(
                            npc.prev_y - game_scene.frame.prev_y,
                            npc.y - game_scene.frame.y,
                            state.frame_time,
                        ),
                        0.4,
                        (192, 0, 0),
                        batch,
                    ),
                    285 | 287 => self.draw_light(
                        interpolate_fix9_scale(
                            npc.prev_x - game_scene.frame.prev_x,
                            npc.x - game_scene.frame.x,
                            state.frame_time,
                        ),
                        interpolate_fix9_scale(
                            npc.prev_y - game_scene.frame.prev_y,
                            npc.y - game_scene.frame.y,
                            state.frame_time,
                        ),
                        1.0,
                        (150, 90, 0),
                        batch,
                    ),
                    293 => self.draw_light(
                        interpolate_fix9_scale(
                            npc.prev_x - game_scene.frame.prev_x,
                            npc.x - game_scene.frame.x,
                            state.frame_time,
                        ),
                        interpolate_fix9_scale(
                            npc.prev_y - game_scene.frame.prev_y,
                            npc.y - game_scene.frame.y,
                            state.frame_time,
                        ),
                        4.0,
                        (255, 255, 255),
                        batch,
                    ),
                    311 => {
                        let size = if npc.anim_num % 7 == 2 || npc.anim_num % 7 == 5 { 1.0 } else { 0.0 };

                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            size,
                            (255, 255, 255),
                            batch,
                        )
                    }
                    312 => self.draw_light(
                        interpolate_fix9_scale(
                            npc.prev_x - game_scene.frame.prev_x,
                            npc.x - game_scene.frame.x,
                            state.frame_time,
                        ),
                        interpolate_fix9_scale(
                            npc.prev_y - game_scene.frame.prev_y,
                            npc.y - game_scene.frame.y,
                            state.frame_time,
                        ),
                        0.5,
                        (255, 255, 255),
                        batch,
                    ),
                    319 => {
                        let color = if npc.anim_num == 2 { (255, 29, 0) } else { (234, 157, 68) };

                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            1.0,
                            color,
                            batch,
                        )
                    }
                    180 => {
                        if state.settings.light_cone {
                            // Curly's looking upward frames
                            let range = if [5, 6, 7, 8, 9].contains(&(npc.anim_num % 11)) {
                                60..120
                            } else if npc.action_num == 40 || npc.action_num == 41 {
                                0..0
                            } else if npc.direction() == Direction::Left {
                                -30..30
                            } else if npc.direction() == Direction::Right {
                                150..210
                            } else {
                                0..0
                            };

                            self.draw_light_raycast(
                                state.tile_size,
                                npc.x + npc.direction.opposite().vector_x() * 0x800,
                                npc.y + 2 * 0x200,
                                (19u8, 34u8, 117u8),
                                0.95,
                                range,
                                batch,
                                game_scene,
                            );
                        }
                    }
                    320 => {
                        if state.settings.light_cone {
                            let range = match npc.direction() {
                                Direction::Up => 60..120,
                                Direction::Bottom => 240..300,
                                Direction::Left => -30..30,
                                Direction::Right => 150..210,
                                _ => 0..0,
                            };

                            self.draw_light_raycast(
                                state.tile_size,
                                npc.x + npc.direction.opposite().vector_x() * 0x800,
                                npc.y + 2 * 0x200,
                                (19u8, 34u8, 117u8),
                                0.95,
                                range,
                                batch,
                                game_scene,
                            );
                        }
                    }
                    322 => {
                        let scale = 0.004 * (npc.action_counter as f32);

                        self.draw_light_raycast(
                            state.tile_size,
                            npc.x,
                            npc.y,
                            (255, 0, 0),
                            scale,
                            0..360,
                            batch,
                            game_scene,
                        )
                    }
                    325 => {
                        let size = 0.5 * (npc.anim_num as f32 + 1.0);
                        self.draw_light(
                            interpolate_fix9_scale(
                                npc.prev_x - game_scene.frame.prev_x,
                                npc.x - game_scene.frame.x,
                                state.frame_time,
                            ),
                            interpolate_fix9_scale(
                                npc.prev_y - game_scene.frame.prev_y,
                                npc.y - game_scene.frame.y,
                                state.frame_time,
                            ),
                            size,
                            (255, 255, 255),
                            batch,
                        )
                    }
                    _ => {}
                }
            }

            batch.draw_filtered(FilterMode::Linear, ctx)?;
        }

        graphics::set_blend_mode(ctx, BlendMode::Multiply)?;
        graphics::set_render_target(ctx, None)?;

        {
            let canvas = state.lightmap_canvas.as_mut().unwrap();
            let rect = Rect { left: 0.0, top: 0.0, right: state.screen_size.0, bottom: state.screen_size.1 };

            canvas.clear();
            canvas.add(SpriteBatchCommand::DrawRect(rect, rect));
            canvas.draw(ctx)?;

            graphics::set_render_target(ctx, Some(canvas.texture_ref()))?;
            graphics::draw_rect(
                ctx,
                Rect {
                    left: 0,
                    top: 0,
                    right: (state.screen_size.0 + 1.0) as isize,
                    bottom: (state.screen_size.1 + 1.0) as isize,
                },
                Color { r: 0.15, g: 0.12, b: 0.12, a: 1.0 },
            )?;
            graphics::set_render_target(ctx, None)?;
            graphics::set_blend_mode(ctx, BlendMode::Add)?;
            canvas.draw(ctx)?;

            graphics::set_blend_mode(ctx, BlendMode::Alpha)?;
        }

        Ok(())
    }
}
