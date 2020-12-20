use ggez::{Context, GameResult};

use crate::common::Rect;
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::input::touch_controls::TouchControlType;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;
use crate::text_script::ScriptMode;

pub struct StageSelect {
    pub current_teleport_slot: u8,
    prev_teleport_slot: u8,
    stage_select_text_y_pos: usize,
    tick: usize,
}

impl StageSelect {
    pub fn new() -> StageSelect {
        StageSelect {
            current_teleport_slot: 0,
            prev_teleport_slot: 0,
            stage_select_text_y_pos: 54,
            tick: 0,
        }
    }

    pub fn reset(&mut self) {
        self.stage_select_text_y_pos = 54;
        self.tick = 0;
    }
}

impl GameEntity<(&Player, &Player)> for StageSelect {
    fn tick(&mut self, state: &mut SharedGameState, (player1, player2): (&Player, &Player)) -> GameResult {
        state.touch_controls.control_type = TouchControlType::None;

        let slot_count = state.teleporter_slots.iter()
            .filter(|&&(index, _event_num)| index != 0)
            .count();

        if slot_count <= self.current_teleport_slot as usize {
            self.current_teleport_slot = 0;
        }

        if self.stage_select_text_y_pos > 46 {
            self.stage_select_text_y_pos -= 1;
        }

        let left_pressed = player1.controller.trigger_left() || player2.controller.trigger_left();
        let right_pressed = player1.controller.trigger_right() || player2.controller.trigger_right();
        let mut ok_pressed = player1.controller.trigger_jump() || player1.controller.trigger_menu_ok()
            || player2.controller.trigger_jump() || player2.controller.trigger_menu_ok();
        let mut cancel_pressed = player1.controller.trigger_shoot() || player2.controller.trigger_shoot();

        if left_pressed {
            if self.current_teleport_slot == 0 {
                self.current_teleport_slot = slot_count.saturating_sub(1) as u8;
            } else {
                self.current_teleport_slot -= 1;
            }
        } else if right_pressed {
            if self.current_teleport_slot == slot_count.saturating_sub(1) as u8 {
                self.current_teleport_slot = 0;
            } else {
                self.current_teleport_slot += 1;
            }
        }

        if self.prev_teleport_slot != self.current_teleport_slot {
            self.prev_teleport_slot = self.current_teleport_slot;
            state.sound_manager.play_sfx(1);
            if let Some(&(index, _event_num)) = state.teleporter_slots.get(self.current_teleport_slot as usize) {
                state.textscript_vm.start_script(1000 + index);
            } else {
                state.textscript_vm.start_script(1000);
            }
        }

        if state.settings.touch_controls {
            let slot_offset = ((state.canvas_size.0 - 40.0 * slot_count as f32) / 2.0).floor();
            let mut slot_rect;

            for i in 0..slot_count {
                slot_rect = Rect::new_size(slot_offset as isize + i as isize * 40 - 2, 64 - 8, 36, 32);

                if state.touch_controls.consume_click_in(slot_rect) {
                    if self.current_teleport_slot as usize == i {
                        ok_pressed = true;
                    } else {
                        state.sound_manager.play_sfx(1);
                        self.current_teleport_slot = i as u8;
                    }

                    break;
                }
            }


            slot_rect = Rect::new_size(state.canvas_size.0 as isize - 34, 8, 26, 26);

            if state.touch_controls.consume_click_in(slot_rect) {
                state.sound_manager.play_sfx(5);
                cancel_pressed = true;
            }
        }

        if ok_pressed || cancel_pressed {
            self.reset();
            state.textscript_vm.set_mode(ScriptMode::Map);
            state.control_flags.set_tick_world(true);
            state.control_flags.set_control_enabled(true);
            state.control_flags.set_interactions_disabled(false);

            if ok_pressed {
                if let Some(&(_index, event_num)) = state.teleporter_slots.get(self.current_teleport_slot as usize) {
                    state.textscript_vm.start_script(event_num);
                }
            }
        }

        self.tick += 1;

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "StageImage")?;

        let slot_count = state.teleporter_slots.iter()
            .filter(|&&(index, _event_num)| index != 0)
            .count();
        let slot_offset = ((state.canvas_size.0 - 40.0 * slot_count as f32) / 2.0).floor();
        let mut slot_rect = Rect::new(0, 0, 0, 0);

        for i in 0..slot_count {
            let index = state.teleporter_slots[i].0;

            slot_rect.left = 32 * (index as u16 % 8);
            slot_rect.top = 16 * (index as u16 / 8);
            slot_rect.right = slot_rect.left + 32;
            slot_rect.bottom = slot_rect.top + 16;

            batch.add_rect(slot_offset + i as f32 * 40.0, 64.0, &slot_rect);
        }

        batch.draw(ctx)?;

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;

        batch.add_rect(128.0, self.stage_select_text_y_pos as f32, &state.constants.textscript.stage_select_text);
        if slot_count > 0 {
            batch.add_rect(slot_offset + self.current_teleport_slot as f32 * 40.0, 64.0, &state.constants.textscript.cursor[self.tick / 2 % 2]);
        }

        batch.draw(ctx)?;

        if state.settings.touch_controls {
            let close_rect = Rect { left: 110, top: 110, right: 128, bottom: 128 };
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "builtin/touch")?;

            batch.add_rect(state.canvas_size.0 - 30.0, 12.0, &close_rect);
            batch.draw(ctx)?;
        }

        Ok(())
    }
}
