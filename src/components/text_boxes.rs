use crate::common::{Color, Rect};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::graphics::draw_rect;
use crate::scripting::tsc::text_script::{ConfirmSelection, TextScriptExecutionState, TextScriptLine};
use crate::shared_game_state::SharedGameState;

pub struct TextBoxes;

const FACE_TEX: &str = "Face";
const SWITCH_FACE_TEX: [&str; 4] = ["Face1", "Face2", "Face3", "Face4"];

impl TextBoxes {
    pub fn new() -> TextBoxes {
        TextBoxes
    }
}

impl GameEntity<()> for TextBoxes {
    fn tick(&mut self, _state: &mut SharedGameState, _custom: ()) -> GameResult {
        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult {
        if !state.textscript_vm.flags.render() {
            return Ok(());
        }

        let (off_left, off_top, off_right, off_bottom) =
            crate::framework::graphics::screen_insets_scaled(ctx, state.scale);

        let center = ((state.canvas_size.0 - off_left - off_right) / 2.0).floor();
        let top_pos = if state.textscript_vm.flags.position_top() {
            32.0 + off_top
        } else {
            state.canvas_size.1 as f32 - off_bottom - 66.0
        };
        let left_pos = off_left + center - 122.0;

        {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;
            if state.textscript_vm.flags.background_visible() {
                batch.add_rect(left_pos, top_pos, &state.constants.textscript.textbox_rect_top);
                for i in 1..7 {
                    batch.add_rect(left_pos, top_pos + i as f32 * 8.0, &state.constants.textscript.textbox_rect_middle);
                }
                batch.add_rect(left_pos, top_pos + 56.0, &state.constants.textscript.textbox_rect_bottom);
            }

            if state.textscript_vm.item != 0 {
                batch.add_rect(
                    center - 40.0,
                    state.canvas_size.1 - off_bottom - 112.0,
                    &state.constants.textscript.get_item_top_left,
                );
                batch.add_rect(
                    center - 40.0,
                    state.canvas_size.1 - off_bottom - 96.0,
                    &state.constants.textscript.get_item_bottom_left,
                );
                batch.add_rect(
                    center + 32.0,
                    state.canvas_size.1 - off_bottom - 112.0,
                    &state.constants.textscript.get_item_top_right,
                );
                batch.add_rect(
                    center + 32.0,
                    state.canvas_size.1 - off_bottom - 104.0,
                    &state.constants.textscript.get_item_right,
                );
                batch.add_rect(
                    center + 32.0,
                    state.canvas_size.1 - off_bottom - 96.0,
                    &state.constants.textscript.get_item_right,
                );
                batch.add_rect(
                    center + 32.0,
                    state.canvas_size.1 - off_bottom - 88.0,
                    &state.constants.textscript.get_item_bottom_right,
                );
            }

            if let TextScriptExecutionState::WaitConfirmation(_, _, _, wait, selection) = state.textscript_vm.state {
                let pos_y = if wait > 14 {
                    state.canvas_size.1 - off_bottom - 96.0 + 4.0 * (17 - wait) as f32
                } else {
                    state.canvas_size.1 - off_bottom - 96.0
                };

                batch.add_rect(center + 56.0, pos_y, &state.constants.textscript.textbox_rect_yes_no);

                if wait == 0 {
                    let pos_x = if selection == ConfirmSelection::No { 41.0 } else { 0.0 };

                    batch.add_rect(
                        center + 51.0 + pos_x,
                        pos_y + 10.0,
                        &state.constants.textscript.textbox_rect_cursor,
                    );
                }
            }

            batch.draw(ctx)?;
        }

        if state.textscript_vm.face != 0 {
            let tex_name = if state.constants.textscript.animated_face_pics { SWITCH_FACE_TEX[0] } else { FACE_TEX };
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, tex_name)?;

            // switch version uses 1xxx flag to show a flipped version of face
            let flip = state.textscript_vm.face > 1000;
            // x1xx flag shows a talking animation
            let _talking = (state.textscript_vm.face % 1000) > 100;
            let face_num = state.textscript_vm.face % 100;

            batch.add_rect_flip(
                left_pos + 14.0,
                top_pos + 8.0,
                flip,
                false,
                &Rect::new_size((face_num as u16 % 6) * 48, (face_num as u16 / 6) * 48, 48, 48),
            );

            batch.draw(ctx)?;
        }

        if state.textscript_vm.item != 0 {
            let mut rect = Rect::new(0, 0, 0, 0);

            if state.textscript_vm.item < 1000 {
                let item_id = state.textscript_vm.item as u16;

                rect.left = (item_id % 16) * 16;
                rect.right = rect.left + 16;
                rect.top = (item_id / 16) * 16;
                rect.bottom = rect.top + 16;

                let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "ArmsImage")?;
                batch.add_rect((center - 12.0).floor(), state.canvas_size.1 - off_bottom - 104.0, &rect);
                batch.draw(ctx)?;
            } else {
                let item_id = state.textscript_vm.item as u16 - 1000;

                rect.left = (item_id % 8) * 32;
                rect.right = rect.left + 32;
                rect.top = (item_id / 8) * 16;
                rect.bottom = rect.top + 16;

                let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "ItemImage")?;
                batch.add_rect((center - 20.0).floor(), state.canvas_size.1 - off_bottom - 104.0, &rect);
                batch.draw(ctx)?;
            }
        }

        let text_offset = if state.textscript_vm.face == 0 { 0.0 } else { 56.0 };

        let lines = [&state.textscript_vm.line_1, &state.textscript_vm.line_2, &state.textscript_vm.line_3];

        for (idx, line) in lines.iter().enumerate() {
            if !line.is_empty() {
                if state.constants.textscript.text_shadow {
                    state.font.draw_text_with_shadow(
                        line.iter().copied(),
                        left_pos + text_offset + 14.0,
                        top_pos + 10.0 + idx as f32 * 16.0,
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;
                } else {
                    state.font.draw_text(
                        line.iter().copied(),
                        left_pos + text_offset + 14.0,
                        top_pos + 10.0 + idx as f32 * 16.0,
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;
                }
            }
        }

        if let TextScriptExecutionState::WaitInput(_, _, tick) = state.textscript_vm.state {
            if tick > 10 {
                let (mut x, y) = match state.textscript_vm.current_line {
                    TextScriptLine::Line1 => (
                        state.font.text_width(state.textscript_vm.line_1.iter().copied(), &state.constants),
                        top_pos + 10.0,
                    ),
                    TextScriptLine::Line2 => (
                        state.font.text_width(state.textscript_vm.line_2.iter().copied(), &state.constants),
                        top_pos + 10.0 + 16.0,
                    ),
                    TextScriptLine::Line3 => (
                        state.font.text_width(state.textscript_vm.line_3.iter().copied(), &state.constants),
                        top_pos + 10.0 + 32.0,
                    ),
                };
                x += left_pos + text_offset + 14.0;

                draw_rect(
                    ctx,
                    Rect::new_size(
                        (x * state.scale) as isize,
                        (y * state.scale) as isize,
                        (5.0 * state.scale) as isize,
                        (state.font.line_height(&state.constants) * state.scale) as isize,
                    ),
                    Color::from_rgb(255, 255, 255),
                )?;
            }
        }

        Ok(())
    }
}
