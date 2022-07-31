use std::collections::HashMap;

use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::gamepad::{self, Axis, AxisDirection, Button, PlayerControllerInputType};
use crate::framework::keyboard::ScanCode;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::settings::{ControllerType, PlayerControllerButtonMap, PlayerKeyMap};
use crate::shared_game_state::SharedGameState;

use super::{ControlMenuData, Menu, MenuEntry, MenuSelectionResult};

const FORBIDDEN_SCANCODES: [ScanCode; 12] = [
    ScanCode::F1,
    ScanCode::F2,
    ScanCode::F3,
    ScanCode::F4,
    ScanCode::F5,
    ScanCode::F6,
    ScanCode::F7,
    ScanCode::F8,
    ScanCode::F9,
    ScanCode::F10,
    ScanCode::F11,
    ScanCode::F12,
];

#[derive(PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
enum CurrentMenu {
    ControllerMenu,
    RebindMenu,
    ConfirmRebindMenu,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ControllerMenuEntry {
    SelectedPlayer,
    Controller,
    Rebind,
    Back,
}

impl Default for ControllerMenuEntry {
    fn default() -> Self {
        ControllerMenuEntry::SelectedPlayer
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum RebindMenuEntry {
    Control(ControlEntry),
    Back,
}

impl Default for RebindMenuEntry {
    fn default() -> Self {
        RebindMenuEntry::Control(ControlEntry::Up)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Player {
    Player1,
    Player2,
}

impl Player {
    fn controller_type(self, state: &SharedGameState) -> ControllerType {
        match self {
            Player::Player1 => state.settings.player1_controller_type,
            Player::Player2 => state.settings.player2_controller_type,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum ControlEntry {
    Left,
    Up,
    Right,
    Down,
    PrevWeapon,
    NextWeapon,
    Jump,
    Shoot,
    Skip,
    Inventory,
    Map,
    Strafe,
}

impl ControlEntry {
    fn to_string(&self, state: &SharedGameState) -> String {
        match self {
            ControlEntry::Left => state.t("menus.controls_menu.rebind_menu.left"),
            ControlEntry::Up => state.t("menus.controls_menu.rebind_menu.up"),
            ControlEntry::Right => state.t("menus.controls_menu.rebind_menu.right"),
            ControlEntry::Down => state.t("menus.controls_menu.rebind_menu.down"),
            ControlEntry::PrevWeapon => state.t("menus.controls_menu.rebind_menu.prev_weapon"),
            ControlEntry::NextWeapon => state.t("menus.controls_menu.rebind_menu.next_weapon"),
            ControlEntry::Jump => state.t("menus.controls_menu.rebind_menu.jump"),
            ControlEntry::Shoot => state.t("menus.controls_menu.rebind_menu.shoot"),
            ControlEntry::Skip => state.t("menus.controls_menu.rebind_menu.skip"),
            ControlEntry::Inventory => state.t("menus.controls_menu.rebind_menu.inventory"),
            ControlEntry::Map => state.t("menus.controls_menu.rebind_menu.map"),
            ControlEntry::Strafe => state.t("menus.controls_menu.rebind_menu.strafe"),
        }
    }
}

pub struct ControlsMenu {
    current: CurrentMenu,
    controller: Menu<ControllerMenuEntry>,
    rebind: Menu<RebindMenuEntry>,
    confirm_rebind: Menu<usize>,

    selected_player: Player,
    selected_controller: ControllerType,
    selected_control: Option<ControlEntry>,

    player1_key_map: Vec<(ControlEntry, ScanCode)>,
    player2_key_map: Vec<(ControlEntry, ScanCode)>,
    player1_controller_button_map: Vec<(ControlEntry, PlayerControllerInputType)>,
    player2_controller_button_map: Vec<(ControlEntry, PlayerControllerInputType)>,

    input_busy: bool,
}

impl ControlsMenu {
    pub fn new() -> ControlsMenu {
        let controller = Menu::new(0, 0, 220, 0);
        let rebind = Menu::new(0, 0, 220, 0);
        let confirm_rebind = Menu::new(0, 0, 220, 0);

        ControlsMenu {
            current: CurrentMenu::ControllerMenu,
            controller,
            rebind,
            confirm_rebind,

            selected_player: Player::Player1,
            selected_controller: ControllerType::Keyboard,
            selected_control: None,

            player1_key_map: Vec::new(),
            player2_key_map: Vec::new(),
            player1_controller_button_map: Vec::new(),
            player2_controller_button_map: Vec::new(),

            input_busy: false,
        }
    }

    pub fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.controller.push_entry(
            ControllerMenuEntry::SelectedPlayer,
            MenuEntry::Options(
                state.t("menus.controls_menu.select_player.entry"),
                self.selected_player as usize,
                vec![
                    state.t("menus.controls_menu.select_player.player_1"),
                    state.t("menus.controls_menu.select_player.player_2"),
                ],
            ),
        );

        self.controller.push_entry(ControllerMenuEntry::Controller, MenuEntry::Hidden);
        self.controller
            .push_entry(ControllerMenuEntry::Rebind, MenuEntry::Active(state.t("menus.controls_menu.rebind")));
        self.controller.push_entry(ControllerMenuEntry::Back, MenuEntry::Active(state.t("common.back")));

        self.player1_key_map = self.init_key_map(&state.settings.player1_key_map);
        self.player2_key_map = self.init_key_map(&state.settings.player2_key_map);
        self.player1_controller_button_map =
            self.init_controller_button_map(&state.settings.player1_controller_button_map);
        self.player2_controller_button_map =
            self.init_controller_button_map(&state.settings.player2_controller_button_map);

        self.confirm_rebind.draw_cursor = false;
        self.confirm_rebind.non_interactive = true;

        self.update_controller_options(state, ctx);
        self.update_rebind_menu(state, ctx);
        self.update_sizes(state);

        Ok(())
    }

    fn update_sizes(&mut self, state: &SharedGameState) {
        self.controller.update_width(state);
        self.controller.update_height();
        self.controller.x = ((state.canvas_size.0 - self.controller.width as f32) / 2.0).floor() as isize;
        self.controller.y = ((state.canvas_size.1 - self.controller.height as f32) / 2.0).floor() as isize;

        self.rebind.update_width(state);
        self.rebind.update_height();
        self.rebind.x = ((state.canvas_size.0 - self.rebind.width as f32) / 2.0).floor() as isize;
        self.rebind.y = ((state.canvas_size.1 - self.rebind.height as f32) / 2.0).floor() as isize;

        self.confirm_rebind.update_width(state);
        self.confirm_rebind.update_height();
        self.confirm_rebind.x = ((state.canvas_size.0 - self.confirm_rebind.width as f32) / 2.0).floor() as isize;
        self.confirm_rebind.y = ((state.canvas_size.1 - self.confirm_rebind.height as f32) / 2.0).floor() as isize;
    }

    fn init_key_map(&self, settings_key_map: &PlayerKeyMap) -> Vec<(ControlEntry, ScanCode)> {
        let mut map = Vec::new();

        map.push((ControlEntry::Up, settings_key_map.up));
        map.push((ControlEntry::Down, settings_key_map.down));
        map.push((ControlEntry::Left, settings_key_map.left));
        map.push((ControlEntry::Right, settings_key_map.right));
        map.push((ControlEntry::Jump, settings_key_map.jump));
        map.push((ControlEntry::Shoot, settings_key_map.shoot));
        map.push((ControlEntry::PrevWeapon, settings_key_map.prev_weapon));
        map.push((ControlEntry::NextWeapon, settings_key_map.next_weapon));
        map.push((ControlEntry::Inventory, settings_key_map.inventory));
        map.push((ControlEntry::Map, settings_key_map.map));
        map.push((ControlEntry::Skip, settings_key_map.skip));
        map.push((ControlEntry::Strafe, settings_key_map.strafe));

        map
    }

    fn init_controller_button_map(
        &self,
        settings_controller_button_map: &PlayerControllerButtonMap,
    ) -> Vec<(ControlEntry, PlayerControllerInputType)> {
        let mut map = Vec::new();

        map.push((ControlEntry::Up, settings_controller_button_map.up));
        map.push((ControlEntry::Down, settings_controller_button_map.down));
        map.push((ControlEntry::Left, settings_controller_button_map.left));
        map.push((ControlEntry::Right, settings_controller_button_map.right));
        map.push((ControlEntry::Jump, settings_controller_button_map.jump));
        map.push((ControlEntry::Shoot, settings_controller_button_map.shoot));
        map.push((ControlEntry::PrevWeapon, settings_controller_button_map.prev_weapon));
        map.push((ControlEntry::NextWeapon, settings_controller_button_map.next_weapon));
        map.push((ControlEntry::Inventory, settings_controller_button_map.inventory));
        map.push((ControlEntry::Map, settings_controller_button_map.map));
        map.push((ControlEntry::Skip, settings_controller_button_map.skip));
        map.push((ControlEntry::Strafe, settings_controller_button_map.strafe));

        map
    }

    fn update_rebind_menu(&mut self, state: &SharedGameState, ctx: &Context) {
        self.rebind.entries.clear();

        match self.selected_player {
            Player::Player1 => {
                if self.selected_controller == ControllerType::Keyboard {
                    for (k, v) in self.player1_key_map.iter() {
                        self.rebind.push_entry(
                            RebindMenuEntry::Control(*k),
                            MenuEntry::Control(
                                k.to_string(state).to_owned(),
                                ControlMenuData::String(format!("{:?}", v)),
                            ),
                        );
                    }
                } else {
                    for (k, v) in self.player1_controller_button_map.iter() {
                        let gamepad_sprite_offset = match state.settings.player1_controller_type {
                            ControllerType::Keyboard => 1,
                            ControllerType::Gamepad(index) => {
                                ctx.gamepad_context.get_gamepad_sprite_offset(index as usize)
                            }
                        };

                        self.rebind.push_entry(
                            RebindMenuEntry::Control(*k),
                            MenuEntry::Control(
                                k.to_string(state).to_owned(),
                                ControlMenuData::Rect(v.get_rect(gamepad_sprite_offset, &state.constants)),
                            ),
                        );
                    }
                }
            }
            Player::Player2 => {
                if self.selected_controller == ControllerType::Keyboard {
                    for (k, v) in self.player2_key_map.iter() {
                        self.rebind.push_entry(
                            RebindMenuEntry::Control(*k),
                            MenuEntry::Control(
                                k.to_string(state).to_owned(),
                                ControlMenuData::String(format!("{:?}", v)),
                            ),
                        );
                    }
                } else {
                    for (k, v) in self.player2_controller_button_map.iter() {
                        let gamepad_sprite_offset = match state.settings.player2_controller_type {
                            ControllerType::Keyboard => 1,
                            ControllerType::Gamepad(index) => {
                                ctx.gamepad_context.get_gamepad_sprite_offset(index as usize)
                            }
                        };

                        self.rebind.push_entry(
                            RebindMenuEntry::Control(*k),
                            MenuEntry::Control(
                                k.to_string(state).to_owned(),
                                ControlMenuData::Rect(v.get_rect(gamepad_sprite_offset, &state.constants)),
                            ),
                        );
                    }
                }
            }
        }

        self.rebind.push_entry(RebindMenuEntry::Back, MenuEntry::Active(state.t("common.back")));
    }

    fn update_controller_options(&mut self, state: &SharedGameState, ctx: &Context) {
        let mut controllers = Vec::new();
        controllers.push(state.t("menus.controls_menu.controller.keyboard"));

        let gamepads = gamepad::get_gamepads(ctx);

        let other_player_controller_type = match self.selected_player {
            Player::Player1 => state.settings.player2_controller_type,
            Player::Player2 => state.settings.player1_controller_type,
        };

        let mut available_gamepads = gamepads.len();

        for i in 0..gamepads.len() {
            if let ControllerType::Gamepad(index) = other_player_controller_type {
                if index as usize == i {
                    available_gamepads -= 1;
                    continue;
                }
            }

            controllers.push(format!("{} {}", gamepads[i].get_gamepad_name(), i + 1));
        }

        let controller_type = match self.selected_player {
            Player::Player1 => state.settings.player1_controller_type,
            Player::Player2 => state.settings.player2_controller_type,
        };

        if let ControllerType::Gamepad(index) = controller_type {
            if index as usize >= available_gamepads {
                self.selected_controller = ControllerType::Keyboard;
            } else {
                self.selected_controller = controller_type;
            }
        } else {
            self.selected_controller = controller_type;
        }

        let controller_idx = match self.selected_controller {
            ControllerType::Keyboard => 0,
            ControllerType::Gamepad(idx) => idx as usize + 1,
        };

        self.controller.set_entry(
            ControllerMenuEntry::Controller,
            MenuEntry::Options(
                state.t("menus.controls_menu.controller.entry"),
                controller_idx as usize,
                controllers.clone(),
            ),
        );
    }

    fn update_confirm_controls_menu(&mut self, state: &SharedGameState) {
        match self.selected_control {
            Some(control) => {
                self.confirm_rebind.entries.clear();

                self.confirm_rebind.push_entry(
                    0,
                    MenuEntry::DisabledWhite(state.tt(
                        "menus.controls_menu.rebind_confirm_menu.title",
                        HashMap::from([("control".to_string(), control.to_string(state))]),
                    )),
                );
                self.confirm_rebind
                    .push_entry(1, MenuEntry::Disabled(state.t("menus.controls_menu.rebind_confirm_menu.cancel")));
            }
            None => {}
        }
    }

    fn is_key_occupied(&self, scan_code: ScanCode) -> bool {
        let other_player_keymap = match self.selected_player {
            Player::Player1 => &self.player2_key_map,
            Player::Player2 => &self.player1_key_map,
        };

        for (_, v) in other_player_keymap.iter() {
            if *v == scan_code {
                return true;
            }
        }

        false
    }

    fn set_key(&mut self, state: &mut SharedGameState, scan_code: ScanCode, ctx: &Context) -> GameResult {
        if self.selected_control.is_none() {
            return Ok(());
        }

        let mut jump_shoot_swapped = false;

        match self.selected_control.unwrap() {
            ControlEntry::Left => match self.selected_player {
                Player::Player1 => state.settings.player1_key_map.left = scan_code,
                Player::Player2 => state.settings.player2_key_map.left = scan_code,
            },
            ControlEntry::Up => match self.selected_player {
                Player::Player1 => state.settings.player1_key_map.up = scan_code,
                Player::Player2 => state.settings.player2_key_map.up = scan_code,
            },
            ControlEntry::Right => match self.selected_player {
                Player::Player1 => state.settings.player1_key_map.right = scan_code,
                Player::Player2 => state.settings.player2_key_map.right = scan_code,
            },
            ControlEntry::Down => match self.selected_player {
                Player::Player1 => state.settings.player1_key_map.down = scan_code,
                Player::Player2 => state.settings.player2_key_map.down = scan_code,
            },
            ControlEntry::PrevWeapon => match self.selected_player {
                Player::Player1 => state.settings.player1_key_map.prev_weapon = scan_code,
                Player::Player2 => state.settings.player2_key_map.prev_weapon = scan_code,
            },
            ControlEntry::NextWeapon => match self.selected_player {
                Player::Player1 => state.settings.player1_key_map.next_weapon = scan_code,
                Player::Player2 => state.settings.player2_key_map.next_weapon = scan_code,
            },
            ControlEntry::Jump => match self.selected_player {
                Player::Player1 => {
                    if state.settings.player1_key_map.shoot == scan_code {
                        state.settings.player1_key_map.shoot = state.settings.player1_key_map.jump;
                        jump_shoot_swapped = true;
                    }

                    state.settings.player1_key_map.jump = scan_code;
                }
                Player::Player2 => {
                    if state.settings.player2_key_map.shoot == scan_code {
                        state.settings.player2_key_map.shoot = state.settings.player2_key_map.jump;
                    }

                    state.settings.player2_key_map.jump = scan_code;
                }
            },
            ControlEntry::Shoot => match self.selected_player {
                Player::Player1 => {
                    if state.settings.player1_key_map.jump == scan_code {
                        state.settings.player1_key_map.jump = state.settings.player1_key_map.shoot;
                        jump_shoot_swapped = true;
                    }

                    state.settings.player1_key_map.jump = scan_code;
                }
                Player::Player2 => {
                    if state.settings.player2_key_map.jump == scan_code {
                        state.settings.player2_key_map.jump = state.settings.player2_key_map.shoot;
                    }

                    state.settings.player2_key_map.shoot = scan_code;
                }
            },
            ControlEntry::Skip => match self.selected_player {
                Player::Player1 => state.settings.player1_key_map.skip = scan_code,
                Player::Player2 => state.settings.player2_key_map.skip = scan_code,
            },
            ControlEntry::Inventory => match self.selected_player {
                Player::Player1 => state.settings.player1_key_map.inventory = scan_code,
                Player::Player2 => state.settings.player2_key_map.inventory = scan_code,
            },
            ControlEntry::Map => match self.selected_player {
                Player::Player1 => state.settings.player1_key_map.map = scan_code,
                Player::Player2 => state.settings.player2_key_map.map = scan_code,
            },
            ControlEntry::Strafe => match self.selected_player {
                Player::Player1 => state.settings.player1_key_map.strafe = scan_code,
                Player::Player2 => state.settings.player2_key_map.strafe = scan_code,
            },
        }

        state.settings.save(ctx)?;

        let keymap = match self.selected_player {
            Player::Player1 => &mut self.player1_key_map,
            Player::Player2 => &mut self.player2_key_map,
        };

        for (entry, value) in keymap.iter_mut() {
            if *entry == self.selected_control.unwrap() {
                *value = scan_code;
            }

            if jump_shoot_swapped {
                let map = match self.selected_player {
                    Player::Player1 => &state.settings.player1_key_map,
                    Player::Player2 => &state.settings.player2_key_map,
                };

                if *entry == ControlEntry::Jump {
                    *value = map.jump;
                } else if *entry == ControlEntry::Shoot {
                    *value = map.shoot;
                }
            }
        }

        Ok(())
    }

    fn set_controller_input(
        &mut self,
        state: &mut SharedGameState,
        input_type: PlayerControllerInputType,
        ctx: &Context,
    ) -> GameResult {
        if self.selected_control.is_none() {
            return Ok(());
        }

        let mut jump_shoot_swapped = false;

        match self.selected_control.unwrap() {
            ControlEntry::Left => match self.selected_player {
                Player::Player1 => state.settings.player1_controller_button_map.left = input_type,
                Player::Player2 => state.settings.player2_controller_button_map.left = input_type,
            },
            ControlEntry::Up => match self.selected_player {
                Player::Player1 => state.settings.player1_controller_button_map.up = input_type,
                Player::Player2 => state.settings.player2_controller_button_map.up = input_type,
            },
            ControlEntry::Right => match self.selected_player {
                Player::Player1 => state.settings.player1_controller_button_map.right = input_type,
                Player::Player2 => state.settings.player2_controller_button_map.right = input_type,
            },
            ControlEntry::Down => match self.selected_player {
                Player::Player1 => state.settings.player1_controller_button_map.down = input_type,
                Player::Player2 => state.settings.player2_controller_button_map.down = input_type,
            },
            ControlEntry::PrevWeapon => match self.selected_player {
                Player::Player1 => state.settings.player1_controller_button_map.prev_weapon = input_type,
                Player::Player2 => state.settings.player2_controller_button_map.prev_weapon = input_type,
            },
            ControlEntry::NextWeapon => match self.selected_player {
                Player::Player1 => state.settings.player1_controller_button_map.next_weapon = input_type,
                Player::Player2 => state.settings.player2_controller_button_map.next_weapon = input_type,
            },
            ControlEntry::Jump => match self.selected_player {
                Player::Player1 => {
                    if state.settings.player1_controller_button_map.shoot == input_type {
                        state.settings.player1_controller_button_map.shoot =
                            state.settings.player1_controller_button_map.jump;
                        jump_shoot_swapped = true;
                    }

                    state.settings.player1_controller_button_map.jump = input_type;
                }
                Player::Player2 => {
                    if state.settings.player2_controller_button_map.shoot == input_type {
                        state.settings.player2_controller_button_map.shoot =
                            state.settings.player2_controller_button_map.jump;
                        jump_shoot_swapped = true;
                    }

                    state.settings.player2_controller_button_map.jump = input_type;
                }
            },
            ControlEntry::Shoot => match self.selected_player {
                Player::Player1 => {
                    if state.settings.player1_controller_button_map.jump == input_type {
                        state.settings.player1_controller_button_map.jump =
                            state.settings.player1_controller_button_map.shoot;
                        jump_shoot_swapped = true;
                    }

                    state.settings.player1_controller_button_map.jump = input_type;
                }
                Player::Player2 => {
                    if state.settings.player2_controller_button_map.jump == input_type {
                        state.settings.player2_controller_button_map.jump =
                            state.settings.player2_controller_button_map.shoot;
                        jump_shoot_swapped = true;
                    }

                    state.settings.player2_controller_button_map.shoot = input_type;
                }
            },
            ControlEntry::Skip => match self.selected_player {
                Player::Player1 => state.settings.player1_controller_button_map.skip = input_type,
                Player::Player2 => state.settings.player2_controller_button_map.skip = input_type,
            },
            ControlEntry::Inventory => match self.selected_player {
                Player::Player1 => state.settings.player1_controller_button_map.inventory = input_type,
                Player::Player2 => state.settings.player2_controller_button_map.inventory = input_type,
            },
            ControlEntry::Map => match self.selected_player {
                Player::Player1 => state.settings.player1_controller_button_map.map = input_type,
                Player::Player2 => state.settings.player2_controller_button_map.map = input_type,
            },
            ControlEntry::Strafe => match self.selected_player {
                Player::Player1 => state.settings.player1_controller_button_map.strafe = input_type,
                Player::Player2 => state.settings.player2_controller_button_map.strafe = input_type,
            },
        }

        state.settings.save(ctx)?;

        let button_map = match self.selected_player {
            Player::Player1 => &mut self.player1_controller_button_map,
            Player::Player2 => &mut self.player2_controller_button_map,
        };

        for (entry, value) in button_map.iter_mut() {
            if *entry == self.selected_control.unwrap() {
                *value = input_type;
            }

            if jump_shoot_swapped {
                let map = match self.selected_player {
                    Player::Player1 => &state.settings.player1_controller_button_map,
                    Player::Player2 => &state.settings.player2_controller_button_map,
                };

                if *entry == ControlEntry::Jump {
                    *value = map.jump;
                } else if *entry == ControlEntry::Shoot {
                    *value = map.shoot;
                }
            }
        }

        Ok(())
    }

    fn normalize_gamepad_input(&self, input: PlayerControllerInputType) -> PlayerControllerInputType {
        match input {
            PlayerControllerInputType::ButtonInput(Button::DPadUp) => {
                PlayerControllerInputType::Either(Button::DPadUp, Axis::LeftY, AxisDirection::Up)
            }
            PlayerControllerInputType::ButtonInput(Button::DPadDown) => {
                PlayerControllerInputType::Either(Button::DPadDown, Axis::LeftY, AxisDirection::Down)
            }
            PlayerControllerInputType::ButtonInput(Button::DPadLeft) => {
                PlayerControllerInputType::Either(Button::DPadLeft, Axis::LeftX, AxisDirection::Left)
            }
            PlayerControllerInputType::ButtonInput(Button::DPadRight) => {
                PlayerControllerInputType::Either(Button::DPadRight, Axis::LeftX, AxisDirection::Right)
            }
            PlayerControllerInputType::AxisInput(Axis::LeftY, AxisDirection::Up) => {
                PlayerControllerInputType::Either(Button::DPadUp, Axis::LeftY, AxisDirection::Up)
            }
            PlayerControllerInputType::AxisInput(Axis::LeftY, AxisDirection::Down) => {
                PlayerControllerInputType::Either(Button::DPadDown, Axis::LeftY, AxisDirection::Down)
            }
            PlayerControllerInputType::AxisInput(Axis::LeftX, AxisDirection::Left) => {
                PlayerControllerInputType::Either(Button::DPadLeft, Axis::LeftX, AxisDirection::Left)
            }
            PlayerControllerInputType::AxisInput(Axis::LeftX, AxisDirection::Right) => {
                PlayerControllerInputType::Either(Button::DPadRight, Axis::LeftX, AxisDirection::Right)
            }
            PlayerControllerInputType::AxisInput(Axis::RightY, AxisDirection::Up) => {
                PlayerControllerInputType::Either(Button::DPadUp, Axis::RightY, AxisDirection::Up)
            }
            PlayerControllerInputType::AxisInput(Axis::RightY, AxisDirection::Down) => {
                PlayerControllerInputType::Either(Button::DPadDown, Axis::RightY, AxisDirection::Down)
            }
            PlayerControllerInputType::AxisInput(Axis::RightX, AxisDirection::Left) => {
                PlayerControllerInputType::Either(Button::DPadLeft, Axis::RightX, AxisDirection::Left)
            }
            PlayerControllerInputType::AxisInput(Axis::RightX, AxisDirection::Right) => {
                PlayerControllerInputType::Either(Button::DPadRight, Axis::RightX, AxisDirection::Right)
            }
            _ => input,
        }
    }

    pub fn tick(
        &mut self,
        exit_action: &mut dyn FnMut(),
        controller: &mut CombinedMenuController,
        state: &mut SharedGameState,
        ctx: &mut Context,
    ) -> GameResult {
        self.update_sizes(state);

        match self.current {
            CurrentMenu::ControllerMenu => match self.controller.tick(controller, state) {
                MenuSelectionResult::Selected(ControllerMenuEntry::SelectedPlayer, toggle)
                | MenuSelectionResult::Left(ControllerMenuEntry::SelectedPlayer, toggle, _)
                | MenuSelectionResult::Right(ControllerMenuEntry::SelectedPlayer, toggle, _) => {
                    if let MenuEntry::Options(_, value, _) = toggle {
                        let (new_player, new_value) = match *value {
                            0 => (Player::Player2, 1),
                            1 => (Player::Player1, 0),
                            _ => unreachable!(),
                        };

                        *value = new_value;

                        self.selected_player = new_player;
                        self.selected_controller = new_player.controller_type(state);

                        self.update_controller_options(state, ctx);
                        self.update_rebind_menu(state, ctx);
                    }
                }
                MenuSelectionResult::Selected(ControllerMenuEntry::Controller, toggle)
                | MenuSelectionResult::Right(ControllerMenuEntry::Controller, toggle, _) => {
                    if self.input_busy {
                        return Ok(());
                    }

                    if let MenuEntry::Options(_, value, entries) = toggle {
                        if *value == entries.len() - 1 {
                            self.selected_controller = ControllerType::Keyboard;
                            *value = 0;
                        } else {
                            self.selected_controller = ControllerType::Gamepad(*value as u32);
                            *value = *value + 1;
                        }
                    }

                    if self.selected_player == Player::Player1 {
                        state.settings.player1_controller_type = self.selected_controller;
                    } else {
                        state.settings.player2_controller_type = self.selected_controller;
                    }

                    let _ = state.settings.save(ctx);

                    let mut new_menu_controller = CombinedMenuController::new();
                    new_menu_controller.add(state.settings.create_player1_controller());
                    new_menu_controller.add(state.settings.create_player2_controller());
                    self.input_busy = true;
                    self.controller.non_interactive = true;
                    *controller = new_menu_controller;

                    self.update_rebind_menu(state, ctx);
                }
                MenuSelectionResult::Left(ControllerMenuEntry::Controller, toggle, _) => {
                    if self.input_busy {
                        return Ok(());
                    }

                    if let MenuEntry::Options(_, value, entries) = toggle {
                        if *value == 1 {
                            self.selected_controller = ControllerType::Keyboard;
                            *value = 0;
                        } else {
                            self.selected_controller = ControllerType::Gamepad(*value as u32);

                            if *value == 0 {
                                *value = entries.len() - 1;
                            } else {
                                *value = *value - 1;
                            }
                        }
                    }

                    if self.selected_player == Player::Player1 {
                        state.settings.player1_controller_type = self.selected_controller;
                    } else {
                        state.settings.player2_controller_type = self.selected_controller;
                    }

                    let _ = state.settings.save(ctx);

                    let mut new_menu_controller = CombinedMenuController::new();
                    new_menu_controller.add(state.settings.create_player1_controller());
                    new_menu_controller.add(state.settings.create_player2_controller());
                    self.input_busy = true;
                    self.controller.non_interactive = true;
                    *controller = new_menu_controller;

                    self.update_rebind_menu(state, ctx);
                }
                MenuSelectionResult::Selected(ControllerMenuEntry::Rebind, _) => {
                    self.current = CurrentMenu::RebindMenu;
                }
                MenuSelectionResult::Selected(ControllerMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    exit_action()
                }
                _ => {}
            },
            CurrentMenu::RebindMenu => match self.rebind.tick(controller, state) {
                MenuSelectionResult::Selected(RebindMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    if !self.input_busy {
                        self.current = CurrentMenu::ControllerMenu;
                    }
                }
                MenuSelectionResult::Selected(RebindMenuEntry::Control(control), _) => {
                    if !self.input_busy {
                        self.selected_control = Some(control);
                        self.update_confirm_controls_menu(state);
                        self.input_busy = true;
                        self.current = CurrentMenu::ConfirmRebindMenu;
                    }
                }
                _ => {}
            },
            CurrentMenu::ConfirmRebindMenu => match self.confirm_rebind.tick(controller, state) {
                _ => {
                    let pressed_keys: Vec<_> = ctx.keyboard_context.pressed_keys().into_iter().collect();

                    for key in pressed_keys.clone() {
                        if *key == ScanCode::Escape {
                            state.sound_manager.play_sfx(5);
                            self.current = CurrentMenu::RebindMenu;
                            return Ok(());
                        }
                    }

                    match self.selected_controller {
                        ControllerType::Keyboard => {
                            if pressed_keys.len() == 1 {
                                if !self.input_busy {
                                    self.input_busy = true;
                                    self.rebind.non_interactive = true;

                                    let key = **pressed_keys.first().unwrap();

                                    if self.is_key_occupied(key)
                                        || FORBIDDEN_SCANCODES.contains(&key)
                                        || self.selected_controller != ControllerType::Keyboard
                                    {
                                        state.sound_manager.play_sfx(12);
                                    } else {
                                        self.set_key(state, key, ctx)?;
                                        self.update_rebind_menu(state, ctx);
                                        self.selected_control = None;
                                        state.sound_manager.play_sfx(18);
                                        self.current = CurrentMenu::RebindMenu;
                                    }
                                }
                            }
                        }
                        ControllerType::Gamepad(idx) => {
                            let pressed_gamepad_buttons: Vec<_> =
                                ctx.gamepad_context.pressed_buttons(idx).into_iter().collect();

                            if pressed_gamepad_buttons.len() == 1 {
                                if !self.input_busy {
                                    self.input_busy = true;
                                    self.rebind.non_interactive = true;

                                    if self.selected_player.controller_type(state) != self.selected_controller {
                                        state.sound_manager.play_sfx(12);
                                    } else {
                                        let button = *pressed_gamepad_buttons.first().unwrap();
                                        let normalized_input = self
                                            .normalize_gamepad_input(PlayerControllerInputType::ButtonInput(button));

                                        self.set_controller_input(state, normalized_input, ctx)?;
                                        self.update_rebind_menu(state, ctx);
                                        self.selected_control = None;
                                        state.sound_manager.play_sfx(18);
                                        self.current = CurrentMenu::RebindMenu;
                                    }
                                }
                            }

                            let active_axes: Vec<_> = ctx.gamepad_context.active_axes(idx).into_iter().collect();

                            if active_axes.len() == 1 {
                                if !self.input_busy {
                                    self.input_busy = true;
                                    self.rebind.non_interactive = true;

                                    if self.selected_player.controller_type(state) != self.selected_controller {
                                        state.sound_manager.play_sfx(12);
                                    } else {
                                        let (axis, value) = *active_axes.first().unwrap();
                                        let direction = AxisDirection::from_axis_data(axis, value);
                                        let normalized_input = self.normalize_gamepad_input(
                                            PlayerControllerInputType::AxisInput(axis, direction),
                                        );

                                        self.set_controller_input(state, normalized_input, ctx)?;
                                        self.update_rebind_menu(state, ctx);
                                        self.selected_control = None;
                                        state.sound_manager.play_sfx(18);
                                        self.current = CurrentMenu::RebindMenu;
                                    }
                                }
                            }

                            if pressed_keys.is_empty() && pressed_gamepad_buttons.is_empty() && active_axes.is_empty() {
                                self.input_busy = false;
                            }
                        }
                    }
                }
            },
        }

        if self.input_busy {
            let pressed_keys = ctx.keyboard_context.pressed_keys();
            let mut input_busy = pressed_keys.len() > 0;

            let gamepads = ctx.gamepad_context.get_gamepads();
            for idx in 0..gamepads.len() {
                let pressed_gamepad_buttons = ctx.gamepad_context.pressed_buttons(idx as u32);
                let active_axes = ctx.gamepad_context.active_axes(idx as u32);

                input_busy = input_busy || !pressed_gamepad_buttons.is_empty() || !active_axes.is_empty();
            }

            self.input_busy = input_busy;

            if !self.input_busy {
                self.controller.non_interactive = false;
                self.rebind.non_interactive = false;
            }
        }

        Ok(())
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        match self.current {
            CurrentMenu::ControllerMenu => self.controller.draw(state, ctx)?,
            CurrentMenu::RebindMenu => self.rebind.draw(state, ctx)?,
            CurrentMenu::ConfirmRebindMenu => self.confirm_rebind.draw(state, ctx)?,
        }

        Ok(())
    }
}
