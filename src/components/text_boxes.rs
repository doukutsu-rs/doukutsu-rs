use crate::common::{Color, Rect};
use crate::engine_constants::AnimatedFace;
use crate::entity::GameEntity;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::game::frame::Frame;
use crate::game::scripting::tsc::text_script::{ConfirmSelection, TextScriptExecutionState, TextScriptLine};
use crate::game::shared_game_state::SharedGameState;
use crate::graphics::font::{Font, Symbols};

pub struct TextBoxes {
    pub item_drop_in: u8,
    pub slide_in: u8,
    pub anim_counter: usize,
    animated_face: AnimatedFace,
}

const FACE_TEX: &str = "Face";
const SWITCH_FACE_TEX: [&str; 5] = ["Face1", "Face2", "Face3", "Face4", "Face5"];

impl TextBoxes {
    pub fn new() -> TextBoxes {
        TextBoxes {
            item_drop_in: 10,
            slide_in: 7,
            anim_counter: 0,
            animated_face: AnimatedFace { face_id: 0, anim_id: 0, anim_frames: vec![(0, 0)] },
        }
    }
}

impl GameEntity<()> for TextBoxes {
    fn tick(&mut self, state: &mut SharedGameState, _custom: ()) -> GameResult {
        if state.textscript_vm.face != 0 {
            self.slide_in = self.slide_in.saturating_sub(1);
            self.anim_counter = self.anim_counter.wrapping_add(1);

            let face_num = state.textscript_vm.face % 100;
            let animation = state.textscript_vm.face % 1000 / 100;

            if state.constants.textscript.animated_face_pics
                && !state.settings.original_textures
                && (self.animated_face.anim_id != animation || self.animated_face.face_id != face_num)
            {
                self.animated_face = state
                    .constants
                    .animated_face_table
                    .clone()
                    .into_iter()
                    .find(|face| face.face_id == face_num && face.anim_id == animation)
                    .unwrap_or_else(|| AnimatedFace { face_id: face_num, anim_id: 0, anim_frames: vec![(0, 0)] });
            }

            if self.anim_counter > self.animated_face.anim_frames.first().unwrap().1 as usize {
                self.animated_face.anim_frames.rotate_left(1);
                self.anim_counter = 0;
            }
        }

        if state.textscript_vm.item != 0 {
            self.item_drop_in = self.item_drop_in.saturating_sub(1);
        }

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
            let clip_rect = Rect::new_size(
                ((left_pos + 14.0) * state.scale) as isize,
                ((top_pos + 8.0) * state.scale) as isize,
                (48.0 * state.scale) as isize,
                (48.0 * state.scale) as isize,
            );

            graphics::set_clip_rect(ctx, Some(clip_rect))?;

            // switch version uses 1xxx flag to show a flipped version of face
            let flip = state.textscript_vm.face > 1000;
            let face_num = state.textscript_vm.face % 100;
            let animation_frame = self.animated_face.anim_frames.first().unwrap().0 as usize;

            let tex_name = if state.constants.textscript.animated_face_pics && !state.settings.original_textures {
                SWITCH_FACE_TEX[animation_frame]
            } else {
                FACE_TEX
            };
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, tex_name)?;

            let face_x = (4.0 + (6 - self.slide_in) as f32 * 8.0) - 52.0;

            let final_x = left_pos + 14.0 + face_x;
            let final_y = top_pos + 8.0;
            let rect = Rect::new_size((face_num as u16 % 6) * 48, (face_num as u16 / 6) * 48, 48, 48);

            if face_num >= 1 && face_num <= 4 && state.more_rust {
                // sue
                batch.add_rect_flip_tinted(final_x, final_y, flip, false, (200, 200, 255, 255), &rect);
            } else {
                batch.add_rect_flip(final_x, final_y, flip, false, &rect);
            }

            batch.draw(ctx)?;
            graphics::set_clip_rect(ctx, None)?;
        }

        if state.textscript_vm.item != 0 {
            let item_offset = self.item_drop_in as f32 * 0.7;
            let mut rect = Rect::new(0, 0, 0, 0);

            if state.textscript_vm.item < 1000 {
                let item_id = state.textscript_vm.item as u16;

                rect.left = (item_id % 16) * 16;
                rect.right = rect.left + 16;
                rect.top = (item_id / 16) * 16;
                rect.bottom = rect.top + 16;

                let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "ArmsImage")?;
                batch.add_rect((center - 12.0).floor(), state.canvas_size.1 - off_bottom - item_offset - 104.0, &rect);
                batch.draw(ctx)?;
            } else {
                let item_id = state.textscript_vm.item as u16 - 1000;

                rect.left = (item_id % 8) * 32;
                rect.right = rect.left + 32;
                rect.top = (item_id / 8) * 16;
                rect.bottom = rect.top + 16;

                let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "ItemImage")?;
                batch.add_rect((center - 20.0).floor(), state.canvas_size.1 - off_bottom - item_offset - 104.0, &rect);
                batch.draw(ctx)?;
            }
        }

        let text_offset = if state.textscript_vm.face == 0 { 0.0 } else { 56.0 };

        let y_offset = if let TextScriptExecutionState::MsgNewLine(_, _, _, _, counter) = state.textscript_vm.state {
            16.0 - counter as f32 * 4.0
        } else {
            0.0
        };

        let lines = [&state.textscript_vm.line_1, &state.textscript_vm.line_2, &state.textscript_vm.line_3];

        let clip_rect = Rect::new_size(
            0,
            ((top_pos + 6.0) * state.scale) as isize,
            state.screen_size.0 as isize,
            (48.0 * state.scale) as isize,
        );

        graphics::set_clip_rect(ctx, Some(clip_rect))?;
        for (idx, line) in lines.iter().enumerate() {
            if !line.is_empty() {
                let symbols = Symbols { symbols: &state.textscript_vm.substitution_rect_map, texture: "TextBox" };

                state
                    .font
                    .builder()
                    .position(left_pos + text_offset + 14.0, top_pos + 10.0 + idx as f32 * 16.0 - y_offset)
                    .shadow(state.constants.textscript.text_shadow)
                    .with_symbols(Some(symbols))
                    .draw_iter(line.iter().copied(), ctx, &state.constants, &mut state.texture_set)?;
            }
        }
        graphics::set_clip_rect(ctx, None)?;

        if let TextScriptExecutionState::WaitInput(_, _, tick) = state.textscript_vm.state {
            if tick > 10 {
                let builder = state
                    .font
                    .builder()
                    .with_symbols(Some(Symbols { symbols: &state.textscript_vm.substitution_rect_map, texture: "" }));

                let (mut x, y) = match state.textscript_vm.current_line {
                    TextScriptLine::Line1 => {
                        (builder.compute_width_iter(state.textscript_vm.line_1.iter().copied()), top_pos + 10.0)
                    }
                    TextScriptLine::Line2 => {
                        (builder.compute_width_iter(state.textscript_vm.line_2.iter().copied()), top_pos + 10.0 + 16.0)
                    }
                    TextScriptLine::Line3 => {
                        (builder.compute_width_iter(state.textscript_vm.line_3.iter().copied()), top_pos + 10.0 + 32.0)
                    }
                };
                x += left_pos + text_offset + 14.0;

                graphics::draw_rect(
                    ctx,
                    Rect::new_size(
                        (x * state.scale) as isize,
                        (y * state.scale) as isize,
                        (5.0 * state.scale) as isize,
                        (state.font.line_height() * state.scale) as isize,
                    ),
                    Color::from_rgb(255, 255, 255),
                )?;
            }
        }

        Ok(())
    }
}
