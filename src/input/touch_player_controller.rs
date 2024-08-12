use bitfield::bitfield;

use crate::common::Rect;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::screen_insets_scaled;
use crate::game::shared_game_state::SharedGameState;
use crate::input::player_controller::PlayerController;
use crate::input::touch_controls::TouchControlType;

/// A no-op implementation of player controller.
#[derive(Clone)]
pub struct TouchPlayerController {
    state: KeyState,
    old_state: KeyState,
    trigger: KeyState,
    prev_touch_len: usize,
}

bitfield! {
    #[allow(unused)]
    #[derive(Clone, Copy)]
    pub struct KeyState(u16);
    impl Debug;

    pub left, set_left: 0;
    pub right, set_right: 1;
    pub up, set_up: 2;
    pub down, set_down: 3;
    pub map, _: 4;
    pub inventory, set_inventory: 5;
    pub jump, set_jump: 6;
    pub shoot, set_shoot: 7;
    pub next_weapon, _: 8;
    pub prev_weapon, _: 9;
    pub pause, set_pause: 10;
}

impl TouchPlayerController {
    pub fn new() -> TouchPlayerController {
        TouchPlayerController { state: KeyState(0), old_state: KeyState(0), trigger: KeyState(0), prev_touch_len: 0 }
    }
}

impl PlayerController for TouchPlayerController {
    fn update(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        match state.touch_controls.control_type {
            TouchControlType::None => {}
            TouchControlType::Dialog => {
                self.state.set_jump(
                    state
                        .touch_controls
                        .point_in(Rect::new_size(
                            0,
                            state.canvas_size.1 as isize / 2,
                            state.canvas_size.0 as isize,
                            state.canvas_size.1 as isize / 2,
                        ))
                        .is_some(),
                );

                if state.touch_controls.points.len() > 1 && self.prev_touch_len != state.touch_controls.points.len() {
                    self.prev_touch_len = state.touch_controls.points.len();
                    self.old_state.set_jump(false);
                }

                let (_, top, right, _) = screen_insets_scaled(ctx, state.scale);

                let top = 4 + top as isize;
                let right = 4 + right as isize;

                self.state.set_inventory(
                    state
                        .touch_controls
                        .point_in(Rect::new_size(state.canvas_size.0 as isize - 48 - right, top, 48, 48))
                        .is_some(),
                );
            }
            TouchControlType::Controls => {
                let (left, _top, right, bottom) = screen_insets_scaled(ctx, state.scale);

                let left = 4 + left as isize;
                let top = 4 + bottom as isize;
                let bottom = 4 + bottom as isize;
                let right = 4 + right as isize;

                self.state.0 = 0;
                // left
                self.state.set_left(
                    self.state.left()
                        || state
                            .touch_controls
                            .point_in(Rect::new_size(left, state.canvas_size.1 as isize - bottom - 48 * 2, 48, 48))
                            .is_some(),
                );

                // up
                self.state.set_up(
                    self.state.up()
                        || state
                            .touch_controls
                            .point_in(Rect::new_size(48 + left, state.canvas_size.1 as isize - bottom - 48 * 3, 48, 48))
                            .is_some(),
                );

                // right
                self.state.set_right(
                    self.state.right()
                        || state
                            .touch_controls
                            .point_in(Rect::new_size(
                                48 * 2 + left,
                                state.canvas_size.1 as isize - bottom - 48 * 2,
                                48,
                                48,
                            ))
                            .is_some(),
                );

                // down
                self.state.set_down(
                    self.state.down()
                        || state
                            .touch_controls
                            .point_in(Rect::new_size(48 + left, state.canvas_size.1 as isize - bottom - 48, 48, 48))
                            .is_some(),
                );

                // left+up
                self.state.set_left(
                    self.state.left()
                        || state
                            .touch_controls
                            .point_in(Rect::new_size(left, state.canvas_size.1 as isize - bottom - 48 * 3, 48, 48))
                            .is_some(),
                );
                self.state.set_up(
                    self.state.up()
                        || state
                            .touch_controls
                            .point_in(Rect::new_size(left, state.canvas_size.1 as isize - bottom - 48 * 3, 48, 48))
                            .is_some(),
                );

                // right+up
                self.state.set_right(
                    self.state.right()
                        || state
                            .touch_controls
                            .point_in(Rect::new_size(
                                48 * 2 + left,
                                state.canvas_size.1 as isize - bottom - 48 * 3,
                                48,
                                48,
                            ))
                            .is_some(),
                );
                self.state.set_up(
                    self.state.up()
                        || state
                            .touch_controls
                            .point_in(Rect::new_size(
                                48 * 2 + left,
                                state.canvas_size.1 as isize - bottom - 48 * 3,
                                48,
                                48,
                            ))
                            .is_some(),
                );

                // left+down
                self.state.set_left(
                    self.state.left()
                        || state
                            .touch_controls
                            .point_in(Rect::new_size(left, state.canvas_size.1 as isize - 48 - bottom, 48, 48))
                            .is_some(),
                );
                self.state.set_down(
                    self.state.down()
                        || state
                            .touch_controls
                            .point_in(Rect::new_size(left, state.canvas_size.1 as isize - 48 - bottom, 48, 48))
                            .is_some(),
                );

                // right+down
                self.state.set_right(
                    self.state.right()
                        || state
                            .touch_controls
                            .point_in(Rect::new_size(48 * 2 + left, state.canvas_size.1 as isize - 48 - bottom, 48, 48))
                            .is_some(),
                );
                self.state.set_down(
                    self.state.down()
                        || state
                            .touch_controls
                            .point_in(Rect::new_size(48 * 2 + left, state.canvas_size.1 as isize - 48 - bottom, 48, 48))
                            .is_some(),
                );

                self.state.set_inventory(
                    self.state.inventory()
                        || state.touch_controls.consume_click_in(Rect::new_size(
                            state.canvas_size.0 as isize - 48 - right,
                            top,
                            48,
                            48,
                        )),
                );

                self.state.set_jump(
                    self.state.jump()
                        || state
                            .touch_controls
                            .point_in(Rect::new_size(
                                state.canvas_size.0 as isize - 48 - right,
                                state.canvas_size.1 as isize - (48 + 4) - bottom,
                                48,
                                48,
                            ))
                            .is_some(),
                );
                self.state.set_shoot(
                    self.state.shoot()
                        || state
                            .touch_controls
                            .point_in(Rect::new_size(
                                state.canvas_size.0 as isize - 48 - right,
                                state.canvas_size.1 as isize - (48 + 4) * 2 - bottom,
                                48,
                                48,
                            ))
                            .is_some(),
                );

                self.state.set_pause(
                    self.state.pause() || state.touch_controls.point_in(Rect::new_size(0, 0, 40, 40)).is_some(),
                );
            }
        }

        Ok(())
    }

    fn update_trigger(&mut self) {
        let mut trigger = self.state.0 ^ self.old_state.0;
        trigger &= self.state.0;
        self.old_state = self.state;
        self.trigger = KeyState(trigger);
    }

    fn move_up(&self) -> bool {
        self.state.up()
    }

    fn move_left(&self) -> bool {
        self.state.left()
    }

    fn move_down(&self) -> bool {
        self.state.down()
    }

    fn move_right(&self) -> bool {
        self.state.right()
    }

    fn prev_weapon(&self) -> bool {
        self.state.prev_weapon()
    }

    fn next_weapon(&self) -> bool {
        self.state.next_weapon()
    }

    fn map(&self) -> bool {
        self.state.map()
    }

    fn inventory(&self) -> bool {
        self.state.inventory()
    }

    fn jump(&self) -> bool {
        self.state.jump()
    }

    fn shoot(&self) -> bool {
        self.state.shoot()
    }

    fn skip(&self) -> bool {
        self.state.inventory()
    }

    fn strafe(&self) -> bool {
        // TODO
        false
    }

    fn trigger_up(&self) -> bool {
        self.trigger.up()
    }

    fn trigger_left(&self) -> bool {
        self.trigger.left()
    }

    fn trigger_down(&self) -> bool {
        self.trigger.down()
    }

    fn trigger_right(&self) -> bool {
        self.trigger.right()
    }

    fn trigger_prev_weapon(&self) -> bool {
        self.trigger.prev_weapon()
    }

    fn trigger_next_weapon(&self) -> bool {
        self.trigger.next_weapon()
    }

    fn trigger_map(&self) -> bool {
        self.trigger.map()
    }

    fn trigger_inventory(&self) -> bool {
        self.trigger.inventory()
    }

    fn trigger_jump(&self) -> bool {
        self.trigger.jump()
    }

    fn trigger_shoot(&self) -> bool {
        self.trigger.shoot()
    }

    fn trigger_skip(&self) -> bool {
        self.trigger.inventory()
    }

    fn trigger_strafe(&self) -> bool {
        // TODO
        false
    }

    fn trigger_menu_ok(&self) -> bool {
        self.trigger.jump()
    }

    fn trigger_menu_back(&self) -> bool {
        self.trigger.shoot()
    }

    fn trigger_menu_pause(&self) -> bool {
        self.trigger.pause()
    }

    fn look_up(&self) -> bool {
        self.state.up()
    }

    fn look_left(&self) -> bool {
        self.state.left()
    }

    fn look_down(&self) -> bool {
        self.state.down()
    }

    fn look_right(&self) -> bool {
        self.state.right()
    }

    fn move_analog_x(&self) -> f64 {
        if self.state.left() && self.state.right() {
            0.0
        } else if self.state.left() {
            -1.0
        } else if self.state.right() {
            1.0
        } else {
            0.0
        }
    }

    fn move_analog_y(&self) -> f64 {
        if self.state.up() && self.state.down() {
            0.0
        } else if self.state.up() {
            -1.0
        } else if self.state.down() {
            1.0
        } else {
            0.0
        }
    }

    fn set_rumble(&mut self, _low_freq: u16, _hi_freq: u16, _ticks: u32) {
        // we could probably vibrate the phone? do mobile games do that?
    }
}
