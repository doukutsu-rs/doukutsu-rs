use crate::common::Rect;
use crate::components::draw_common::{draw_number, Alignment};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::screen_insets_scaled;
use crate::inventory::Inventory;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;

pub struct HUD {
    pub alignment: Alignment,
    pub weapon_x_pos: usize,
    pub visible: bool,
    pub has_player2: bool,
    ammo: u16,
    max_ammo: u16,
    xp: u16,
    max_xp: u16,
    xp_bar_counter: u8,
    max_level: bool,
    life: u16,
    max_life: u16,
    life_bar: u16,
    life_bar_counter: u16,
    air: u16,
    air_counter: u16,
    current_level: usize,
    weapon_count: usize,
    current_weapon: isize,
    weapon_types: [u8; 16],
}

impl HUD {
    pub fn new(alignment: Alignment) -> HUD {
        HUD {
            alignment,
            weapon_x_pos: 16,
            visible: false,
            has_player2: false,
            ammo: 0,
            max_ammo: 0,
            xp: 0,
            max_xp: 0,
            xp_bar_counter: 0,
            max_level: false,
            life: 0,
            max_life: 0,
            life_bar: 0,
            life_bar_counter: 0,
            air: 0,
            air_counter: 0,
            current_level: 0,
            weapon_count: 0,
            current_weapon: 0,
            weapon_types: [0; 16],
        }
    }
}

impl GameEntity<(&Player, &mut Inventory)> for HUD {
    fn tick(&mut self, state: &mut SharedGameState, (player, inventory): (&Player, &mut Inventory)) -> GameResult {
        let (ammo, max_ammo) = inventory.get_current_ammo();
        let (xp, max_xp, max_level) = inventory.get_current_max_exp(&state.constants);

        self.ammo = ammo;
        self.max_ammo = max_ammo;
        self.xp = xp;
        self.max_xp = max_xp;
        self.xp_bar_counter = if player.xp_counter != 0 { self.xp_bar_counter.wrapping_add(1) } else { 0 };
        self.max_level = max_level;

        self.life = player.life;
        self.max_life = player.max_life;
        self.air = player.air;
        self.air_counter = player.air_counter;
        self.weapon_count = inventory.get_weapon_count();
        self.current_weapon = inventory.get_current_weapon_idx() as isize;

        self.current_level = inventory.get_current_level() as usize;

        for (a, slot) in self.weapon_types.iter_mut().enumerate() {
            *slot = if let Some(weapon) = inventory.get_weapon(a) { weapon.wtype as u8 } else { 0 };
        }

        // update health bar
        if self.life_bar < self.life as u16 {
            self.life_bar = self.life as u16;
        }

        if self.life_bar > self.life as u16 {
            self.life_bar_counter += 1;
            if self.life_bar_counter > 30 {
                self.life_bar -= 1;
            }
        } else {
            self.life_bar_counter = 0;
        }

        if self.weapon_x_pos > 16 {
            self.weapon_x_pos -= 2;
        } else if self.weapon_x_pos < 16 {
            self.weapon_x_pos += 2;
        }

        if player.cond.alive() {
            if player.controller.trigger_next_weapon() {
                state.sound_manager.play_sfx(4);
                inventory.next_weapon();
                self.weapon_x_pos = 32;
            }

            if player.controller.trigger_prev_weapon() {
                state.sound_manager.play_sfx(4);
                inventory.prev_weapon();
                self.weapon_x_pos = 0;
            }
        }

        // touch handler
        if state.settings.touch_controls && self.weapon_count != 0 {
            let mut rect;
            let weapon_offset = match self.alignment {
                Alignment::Left => 0,
                Alignment::Right => (state.canvas_size.0 - 104.0) as isize,
            };

            for a in 0..self.weapon_count {
                let mut pos_x = ((a as isize - self.current_weapon) * 16) + self.weapon_x_pos as isize;

                if pos_x < 8 {
                    pos_x += 48 + self.weapon_count as isize * 16;
                } else if pos_x >= 24 {
                    pos_x += 48;
                }

                if pos_x >= 72 + ((self.weapon_count as isize - 1) * 16) {
                    pos_x -= 48 + self.weapon_count as isize * 16;
                } else if pos_x < 72 && pos_x >= 24 {
                    pos_x -= 48;
                }

                let wtype = self.weapon_types[a];
                if wtype != 0 {
                    rect = Rect::new_size(pos_x + weapon_offset - 4, 16 - 4, 24, 24);

                    if state.touch_controls.consume_click_in(rect) {
                        state.sound_manager.play_sfx(4);
                        inventory.current_weapon = a as u16;
                        self.weapon_x_pos = 32;
                    }
                }
            }
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult {
        if !self.visible {
            return Ok(());
        }

        let (left, top, right, bottom) = screen_insets_scaled(ctx, state.scale);

        // none
        let weap_x = self.weapon_x_pos as f32;
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;

        let (bar_offset, num_offset, weapon_offset) = match self.alignment {
            Alignment::Left => (left, left, left),
            Alignment::Right => (
                state.canvas_size.0 - 112.0 - right,
                state.canvas_size.0 - 48.0 - right,
                state.canvas_size.0 - 40.0 - right,
            ),
        };
        let air_offset = if self.has_player2 {
            50.0 * match self.alignment {
                Alignment::Left => -1.0,
                Alignment::Right => 1.0,
            }
        } else {
            0.0
        };

        if self.max_ammo == 0 {
            batch.add_rect(bar_offset + weap_x + 48.0, 16.0 + top, &Rect::new_size(80, 48, 16, 8));
            batch.add_rect(bar_offset + weap_x + 48.0, 24.0 + top, &Rect::new_size(80, 48, 16, 8));
        }

        // per
        batch.add_rect(bar_offset + weap_x + 32.0, 24.0 + top, &Rect::new_size(72, 48, 8, 8));
        // lv
        batch.add_rect(num_offset + weap_x, 32.0 + top, &Rect::new_size(80, 80, 16, 8));
        // xp box
        batch.add_rect(bar_offset + weap_x + 24.0, 32.0 + top, &Rect::new_size(0, 72, 40, 8));

        if self.max_level {
            batch.add_rect(bar_offset + weap_x + 24.0, 32.0 + top, &Rect::new_size(40, 72, 40, 8));
        } else if self.max_xp > 0 {
            // xp bar
            let bar_width = (self.xp as f32 / self.max_xp as f32 * 40.0) as u16;

            batch.add_rect(bar_offset + weap_x + 24.0, 32.0 + top, &Rect::new_size(0, 80, bar_width, 8));
        }

        if (self.xp_bar_counter & 0x02) != 0 {
            batch.add_rect(bar_offset + weap_x + 24.0, 32.0 + top, &Rect::new_size(40, 80, 40, 8));
        }

        if self.max_life != 0 {
            let yellow_bar_width = (self.life_bar as f32 / self.max_life as f32 * 39.0) as u16;
            let bar_width = (self.life as f32 / self.max_life as f32 * 39.0) as u16;

            // heart/hp number box
            batch.add_rect(num_offset + 16.0, 40.0 + top, &Rect::new_size(0, 40, 24, 8));
            // life box
            batch.add_rect(bar_offset + 40.0, 40.0 + top, &Rect::new_size(24, 40, 40, 8));
            // yellow bar
            batch.add_rect(bar_offset + 40.0, 40.0 + top, &Rect::new_size(0, 32, yellow_bar_width, 8));
            // life
            batch.add_rect(bar_offset + 40.0, 40.0 + top, &Rect::new_size(0, 24, bar_width, 8));
        }

        if self.air_counter > 0 {
            let rect = if self.air % 30 > 10 { Rect::new_size(112, 72, 32, 8) } else { Rect::new_size(112, 80, 32, 8) };

            batch.add_rect(
                left + ((state.canvas_size.0 - left - right) / 2.0).floor() - 40.0 + air_offset,
                top + ((state.canvas_size.1 - top - bottom) / 2.0).floor(),
                &rect,
            );
        }

        batch.draw(ctx)?;
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "ArmsImage")?;

        if self.weapon_count != 0 {
            let mut rect = Rect::new(0, 0, 0, 16);

            for a in 0..self.weapon_count {
                let mut pos_x = ((a as isize - self.current_weapon) as f32 * 16.0) + weap_x;

                if pos_x < 8.0 {
                    pos_x += 48.0 + self.weapon_count as f32 * 16.0;
                } else if pos_x >= 24.0 {
                    pos_x += 48.0;
                }

                if pos_x >= 72.0 + ((self.weapon_count - 1) as f32 * 16.0) {
                    pos_x -= 48.0 + self.weapon_count as f32 * 16.0;
                } else if pos_x < 72.0 && pos_x >= 24.0 {
                    pos_x -= 48.0;
                }

                if self.alignment == Alignment::Right && pos_x > 32.0 {
                    pos_x -= 96.0 + self.weapon_count as f32 * 16.0;
                }

                let wtype = self.weapon_types[a];
                if wtype != 0 {
                    rect.left = wtype as u16 * 16;
                    rect.right = rect.left + 16;
                    batch.add_rect(pos_x + weapon_offset, 16.0 + top, &rect);
                }
            }
        }

        batch.draw(ctx)?;

        if self.air_counter > 0 && self.air_counter % 6 < 4 {
            draw_number(
                left + ((state.canvas_size.0 - left - right) / 2.0).floor() + 8.0 + air_offset,
                top + ((state.canvas_size.1 - top - bottom) / 2.0).floor(),
                (self.air / 10) as usize,
                Alignment::Left,
                state,
                ctx,
            )?;
        }

        if self.max_ammo != 0 {
            draw_number(num_offset + weap_x + 64.0, 16.0 + top, self.ammo as usize, Alignment::Right, state, ctx)?;
            draw_number(num_offset + weap_x + 64.0, 24.0 + top, self.max_ammo as usize, Alignment::Right, state, ctx)?;
        }
        draw_number(num_offset + weap_x + 24.0, 32.0 + top, self.current_level, Alignment::Right, state, ctx)?;
        draw_number(num_offset + 40.0, 40.0 + top, self.life_bar as usize, Alignment::Right, state, ctx)?;

        Ok(())
    }
}
