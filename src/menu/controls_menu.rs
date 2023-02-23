use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::gamepad::{self, Axis, AxisDirection, Button, PlayerControllerInputType};
use crate::framework::keyboard::ScanCode;
use crate::game::settings::{
    p1_default_keymap, p2_default_keymap, player_default_controller_button_map, ControllerType,
    PlayerControllerButtonMap, PlayerKeyMap,
};
use crate::game::shared_game_state::SharedGameState;
use crate::input::combined_menu_controller::CombinedMenuController;

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
    MainMenu,
    SelectControllerMenu,
    RebindMenu,
    ConfirmRebindMenu,
    ConfirmResetMenu,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum MainMenuEntry {
    SelectedPlayer,
    Controller,
    Rebind,
    Rumble,
    Back,
}

impl Default for MainMenuEntry {
    fn default() -> Self {
        MainMenuEntry::SelectedPlayer
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SelectControllerMenuEntry {
    Keyboard,
    Gamepad(usize),
    Back,
}

impl Default for SelectControllerMenuEntry {
    fn default() -> Self {
        SelectControllerMenuEntry::Keyboard
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum RebindMenuEntry {
    Control(ControlEntry),
    Reset,
    Back,
}

impl Default for RebindMenuEntry {
    fn default() -> Self {
        RebindMenuEntry::Control(ControlEntry::MenuOk)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ConfirmResetMenuEntry {
    Title,
    Yes,
    No,
}

impl Default for ConfirmResetMenuEntry {
    fn default() -> Self {
        ConfirmResetMenuEntry::No
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
    MenuOk,
    MenuBack,
}

impl ControlEntry {
    fn to_string(&self, state: &SharedGameState) -> String {
        match self {
            ControlEntry::Left => state.loc.t("menus.controls_menu.rebind_menu.left"),
            ControlEntry::Up => state.loc.t("menus.controls_menu.rebind_menu.up"),
            ControlEntry::Right => state.loc.t("menus.controls_menu.rebind_menu.right"),
            ControlEntry::Down => state.loc.t("menus.controls_menu.rebind_menu.down"),
            ControlEntry::PrevWeapon => state.loc.t("menus.controls_menu.rebind_menu.prev_weapon"),
            ControlEntry::NextWeapon => state.loc.t("menus.controls_menu.rebind_menu.next_weapon"),
            ControlEntry::Jump => state.loc.t("menus.controls_menu.rebind_menu.jump"),
            ControlEntry::Shoot => state.loc.t("menus.controls_menu.rebind_menu.shoot"),
            ControlEntry::Skip => state.loc.t("menus.controls_menu.rebind_menu.skip"),
            ControlEntry::Inventory => state.loc.t("menus.controls_menu.rebind_menu.inventory"),
            ControlEntry::Map => state.loc.t("menus.controls_menu.rebind_menu.map"),
            ControlEntry::Strafe => state.loc.t("menus.controls_menu.rebind_menu.strafe"),
            ControlEntry::MenuOk => state.loc.t("menus.controls_menu.rebind_menu.menu_ok"),
            ControlEntry::MenuBack => state.loc.t("menus.controls_menu.rebind_menu.menu_back"),
        }
        .to_owned()
    }
}

pub struct ControlsMenu {
    current: CurrentMenu,
    main: Menu<MainMenuEntry>,
    select_controller: Menu<SelectControllerMenuEntry>,
    rebind: Menu<RebindMenuEntry>,
    confirm_rebind: Menu<usize>,
    confirm_reset: Menu<ConfirmResetMenuEntry>,

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
        let main = Menu::new(0, 0, 220, 0);
        let select_controller = Menu::new(0, 0, 220, 0);
        let rebind = Menu::new(0, 0, 220, 0);
        let confirm_rebind = Menu::new(0, 0, 220, 0);
        let confirm_reset = Menu::new(0, 0, 160, 0);

        ControlsMenu {
            current: CurrentMenu::MainMenu,
            main,
            select_controller,
            rebind,
            confirm_rebind,
            confirm_reset,

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
        self.main.push_entry(
            MainMenuEntry::SelectedPlayer,
            MenuEntry::Options(
                state.loc.t("menus.controls_menu.select_player.entry").to_owned(),
                self.selected_player as usize,
                vec![
                    state.loc.t("menus.controls_menu.select_player.player_1").to_owned(),
                    state.loc.t("menus.controls_menu.select_player.player_2").to_owned(),
                ],
            ),
        );

        self.main.push_entry(
            MainMenuEntry::Controller,
            MenuEntry::Active(state.loc.t("menus.controls_menu.controller.entry").to_owned()),
        );
        self.main
            .push_entry(MainMenuEntry::Rebind, MenuEntry::Active(state.loc.t("menus.controls_menu.rebind").to_owned()));
        self.main.push_entry(MainMenuEntry::Rumble, MenuEntry::Hidden);
        self.main.push_entry(MainMenuEntry::Back, MenuEntry::Active(state.loc.t("common.back").to_owned()));

        self.confirm_reset.push_entry(
            ConfirmResetMenuEntry::Title,
            MenuEntry::Disabled(state.loc.t("menus.controls_menu.reset_confirm_menu_title").to_owned()),
        );
        self.confirm_reset
            .push_entry(ConfirmResetMenuEntry::Yes, MenuEntry::Active(state.loc.t("common.yes").to_owned()));
        self.confirm_reset
            .push_entry(ConfirmResetMenuEntry::No, MenuEntry::Active(state.loc.t("common.no").to_owned()));

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
        self.main.update_width(state);
        self.main.update_height(state);
        self.main.x = ((state.canvas_size.0 - self.main.width as f32) / 2.0).floor() as isize;
        self.main.y = ((state.canvas_size.1 - self.main.height as f32) / 2.0).floor() as isize;

        self.select_controller.update_width(state);
        self.select_controller.update_height(state);
        self.select_controller.x = ((state.canvas_size.0 - self.select_controller.width as f32) / 2.0).floor() as isize;
        self.select_controller.y =
            ((state.canvas_size.1 - self.select_controller.height as f32) / 2.0).floor() as isize;

        self.rebind.update_width(state);
        self.rebind.update_height(state);
        self.rebind.x = ((state.canvas_size.0 - self.rebind.width as f32) / 2.0).floor() as isize;
        self.rebind.y = ((state.canvas_size.1 - self.rebind.height as f32) / 2.0).floor() as isize;

        self.confirm_rebind.update_width(state);
        self.confirm_rebind.update_height(state);
        self.confirm_rebind.x = ((state.canvas_size.0 - self.confirm_rebind.width as f32) / 2.0).floor() as isize;
        self.confirm_rebind.y = ((state.canvas_size.1 - self.confirm_rebind.height as f32) / 2.0).floor() as isize;

        self.confirm_reset.update_width(state);
        self.confirm_reset.update_height(state);
        self.confirm_reset.x = ((state.canvas_size.0 - self.confirm_reset.width as f32) / 2.0).floor() as isize;
        self.confirm_reset.y = ((state.canvas_size.1 - self.confirm_reset.height as f32) / 2.0).floor() as isize;
    }

    fn init_key_map(&self, settings_key_map: &PlayerKeyMap) -> Vec<(ControlEntry, ScanCode)> {
        let mut map = Vec::new();

        map.push((ControlEntry::MenuOk, settings_key_map.menu_ok));
        map.push((ControlEntry::MenuBack, settings_key_map.menu_back));
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

        map.push((ControlEntry::MenuOk, settings_controller_button_map.menu_ok));
        map.push((ControlEntry::MenuBack, settings_controller_button_map.menu_back));
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

        self.rebind.push_entry(
            RebindMenuEntry::Reset,
            MenuEntry::Active(state.loc.t("menus.controls_menu.reset_confirm").to_owned()),
        );
        self.rebind.push_entry(RebindMenuEntry::Back, MenuEntry::Active(state.loc.t("common.back").to_owned()));
    }

    fn update_controller_options(&mut self, state: &SharedGameState, ctx: &Context) {
        self.select_controller.entries.clear();

        self.select_controller.push_entry(
            SelectControllerMenuEntry::Keyboard,
            MenuEntry::Active(state.loc.t("menus.controls_menu.controller.keyboard").to_owned()),
        );

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

            self.select_controller.push_entry(
                SelectControllerMenuEntry::Gamepad(i),
                MenuEntry::Active(format!("{} {}", gamepads[i].get_gamepad_name(), i + 1)),
            );
        }

        self.select_controller
            .push_entry(SelectControllerMenuEntry::Back, MenuEntry::Active(state.loc.t("common.back").to_owned()));

        let controller_type = match self.selected_player {
            Player::Player1 => state.settings.player1_controller_type,
            Player::Player2 => state.settings.player2_controller_type,
        };

        let rumble = match self.selected_player {
            Player::Player1 => state.settings.player1_rumble,
            Player::Player2 => state.settings.player2_rumble,
        };

        if let ControllerType::Gamepad(index) = controller_type {
            if index as usize >= available_gamepads {
                self.selected_controller = ControllerType::Keyboard;
                self.main.set_entry(MainMenuEntry::Rumble, MenuEntry::Hidden);
            } else {
                self.selected_controller = controller_type;
                self.main.set_entry(
                    MainMenuEntry::Rumble,
                    MenuEntry::Toggle(state.loc.t("menus.controls_menu.rumble").to_owned(), rumble),
                );
            }
        } else {
            self.selected_controller = controller_type;
            self.main.set_entry(MainMenuEntry::Rumble, MenuEntry::Hidden);
        }

        match self.selected_controller {
            ControllerType::Keyboard => self.select_controller.selected = SelectControllerMenuEntry::Keyboard,
            ControllerType::Gamepad(index) => {
                self.select_controller.selected = SelectControllerMenuEntry::Gamepad(index as usize)
            }
        }
    }

    fn update_confirm_controls_menu(&mut self, state: &SharedGameState) {
        match self.selected_control {
            Some(control) => {
                self.confirm_rebind.entries.clear();

                self.confirm_rebind.push_entry(
                    0,
                    MenuEntry::DisabledWhite(state.tt(
                        "menus.controls_menu.rebind_confirm_menu.title",
                        &[("control", control.to_string(state).as_str())],
                    )),
                );
                self.confirm_rebind.push_entry(
                    1,
                    MenuEntry::Disabled(state.loc.t("menus.controls_menu.rebind_confirm_menu.cancel").to_owned()),
                );
            }
            None => {}
        }
    }

    fn reset_controls(&mut self, state: &mut SharedGameState, ctx: &Context) -> GameResult {
        match self.selected_player {
            Player::Player1 => {
                if self.selected_controller == ControllerType::Keyboard {
                    state.settings.player1_key_map = p1_default_keymap();
                    self.player1_key_map = self.init_key_map(&state.settings.player1_key_map);
                } else {
                    state.settings.player1_controller_button_map = player_default_controller_button_map();
                    self.player1_controller_button_map =
                        self.init_controller_button_map(&state.settings.player1_controller_button_map);
                }
            }
            Player::Player2 => {
                if self.selected_controller == ControllerType::Keyboard {
                    state.settings.player2_key_map = p2_default_keymap();
                    self.player2_key_map = self.init_key_map(&state.settings.player2_key_map);
                } else {
                    state.settings.player2_controller_button_map = player_default_controller_button_map();
                    self.player2_controller_button_map =
                        self.init_controller_button_map(&state.settings.player2_controller_button_map);
                }
            }
        }

        state.settings.save(ctx)
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

        let mut did_swap_controls = false;

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
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player1_key_map.jump,
                        &mut state.settings.player1_key_map.shoot,
                        scan_code,
                    );
                }
                Player::Player2 => {
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player2_key_map.jump,
                        &mut state.settings.player2_key_map.shoot,
                        scan_code,
                    );
                }
            },
            ControlEntry::Shoot => match self.selected_player {
                Player::Player1 => {
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player1_key_map.shoot,
                        &mut state.settings.player1_key_map.jump,
                        scan_code,
                    );
                }
                Player::Player2 => {
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player2_key_map.shoot,
                        &mut state.settings.player2_key_map.jump,
                        scan_code,
                    );
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
            ControlEntry::MenuOk => match self.selected_player {
                Player::Player1 => {
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player1_key_map.menu_ok,
                        &mut state.settings.player1_key_map.menu_back,
                        scan_code,
                    );
                }
                Player::Player2 => {
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player2_key_map.menu_ok,
                        &mut state.settings.player2_key_map.menu_back,
                        scan_code,
                    );
                }
            },
            ControlEntry::MenuBack => match self.selected_player {
                Player::Player1 => {
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player1_key_map.menu_back,
                        &mut state.settings.player1_key_map.menu_ok,
                        scan_code,
                    );
                }
                Player::Player2 => {
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player2_key_map.menu_back,
                        &mut state.settings.player2_key_map.menu_ok,
                        scan_code,
                    );
                }
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

            if did_swap_controls {
                let map = match self.selected_player {
                    Player::Player1 => &state.settings.player1_key_map,
                    Player::Player2 => &state.settings.player2_key_map,
                };

                match *entry {
                    ControlEntry::Jump => *value = map.jump,
                    ControlEntry::Shoot => *value = map.shoot,
                    ControlEntry::MenuOk => *value = map.menu_ok,
                    ControlEntry::MenuBack => *value = map.menu_back,
                    _ => {}
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

        let mut did_swap_controls = false;

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
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player1_controller_button_map.jump,
                        &mut state.settings.player1_controller_button_map.shoot,
                        input_type,
                    );
                }
                Player::Player2 => {
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player2_controller_button_map.jump,
                        &mut state.settings.player2_controller_button_map.shoot,
                        input_type,
                    );
                }
            },
            ControlEntry::Shoot => match self.selected_player {
                Player::Player1 => {
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player1_controller_button_map.shoot,
                        &mut state.settings.player1_controller_button_map.jump,
                        input_type,
                    );
                }
                Player::Player2 => {
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player2_controller_button_map.shoot,
                        &mut state.settings.player2_controller_button_map.jump,
                        input_type,
                    );
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
            ControlEntry::MenuOk => match self.selected_player {
                Player::Player1 => {
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player1_controller_button_map.menu_ok,
                        &mut state.settings.player1_controller_button_map.menu_back,
                        input_type,
                    );
                }
                Player::Player2 => {
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player2_controller_button_map.menu_ok,
                        &mut state.settings.player2_controller_button_map.menu_back,
                        input_type,
                    );
                }
            },
            ControlEntry::MenuBack => match self.selected_player {
                Player::Player1 => {
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player1_controller_button_map.menu_back,
                        &mut state.settings.player1_controller_button_map.menu_ok,
                        input_type,
                    );
                }
                Player::Player2 => {
                    did_swap_controls = self.swap_if_same(
                        &mut state.settings.player2_controller_button_map.menu_back,
                        &mut state.settings.player2_controller_button_map.menu_ok,
                        input_type,
                    );
                }
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

            if did_swap_controls {
                let map = match self.selected_player {
                    Player::Player1 => &state.settings.player1_controller_button_map,
                    Player::Player2 => &state.settings.player2_controller_button_map,
                };

                match *entry {
                    ControlEntry::Jump => *value = map.jump,
                    ControlEntry::Shoot => *value = map.shoot,
                    ControlEntry::MenuOk => *value = map.menu_ok,
                    ControlEntry::MenuBack => *value = map.menu_back,
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn swap_if_same<T: Eq + Copy>(&mut self, fst: &mut T, snd: &mut T, value: T) -> bool {
        let mut swapped = false;

        if *snd == value {
            *snd = *fst;
            swapped = true;
        }

        *fst = value;

        swapped
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
            CurrentMenu::MainMenu => match self.main.tick(controller, state) {
                MenuSelectionResult::Selected(MainMenuEntry::SelectedPlayer, toggle)
                | MenuSelectionResult::Left(MainMenuEntry::SelectedPlayer, toggle, _)
                | MenuSelectionResult::Right(MainMenuEntry::SelectedPlayer, toggle, _) => {
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
                MenuSelectionResult::Selected(MainMenuEntry::Controller, _) => {
                    if self.input_busy {
                        return Ok(());
                    }

                    self.update_controller_options(state, ctx);
                    self.current = CurrentMenu::SelectControllerMenu;
                }
                MenuSelectionResult::Selected(MainMenuEntry::Rebind, _) => {
                    self.current = CurrentMenu::RebindMenu;
                }
                MenuSelectionResult::Selected(MainMenuEntry::Rumble, toggle) => {
                    if let MenuEntry::Toggle(_, value) = toggle {
                        match self.selected_player {
                            Player::Player1 => {
                                state.settings.player1_rumble = !state.settings.player1_rumble;

                                if state.settings.player1_rumble {
                                    if let ControllerType::Gamepad(idx) = self.selected_controller {
                                        gamepad::set_rumble(
                                            ctx,
                                            state,
                                            idx,
                                            0,
                                            0x5000,
                                            (state.settings.timing_mode.get_tps() / 2) as u32,
                                        )?;
                                    }
                                }

                                *value = state.settings.player1_rumble;
                            }
                            Player::Player2 => {
                                state.settings.player2_rumble = !state.settings.player2_rumble;

                                if state.settings.player2_rumble {
                                    if let ControllerType::Gamepad(idx) = self.selected_controller {
                                        gamepad::set_rumble(
                                            ctx,
                                            state,
                                            idx,
                                            0,
                                            0x5000,
                                            (state.settings.timing_mode.get_tps() / 2) as u32,
                                        )?;
                                    }
                                }

                                *value = state.settings.player2_rumble;
                            }
                        }

                        state.settings.save(ctx)?;
                    }
                }
                MenuSelectionResult::Selected(MainMenuEntry::Back, _) | MenuSelectionResult::Canceled => exit_action(),
                _ => {}
            },
            CurrentMenu::SelectControllerMenu => match self.select_controller.tick(controller, state) {
                MenuSelectionResult::Selected(SelectControllerMenuEntry::Keyboard, _) => {
                    if self.selected_player == Player::Player1 {
                        state.settings.player1_controller_type = ControllerType::Keyboard;
                    } else {
                        state.settings.player2_controller_type = ControllerType::Keyboard;
                    }

                    let _ = state.settings.save(ctx);

                    let mut new_menu_controller = CombinedMenuController::new();
                    new_menu_controller.add(state.settings.create_player1_controller());
                    new_menu_controller.add(state.settings.create_player2_controller());
                    self.input_busy = true;
                    self.main.non_interactive = true;
                    *controller = new_menu_controller;

                    self.selected_controller = ControllerType::Keyboard;
                    self.update_rebind_menu(state, ctx);

                    self.current = CurrentMenu::MainMenu;
                }
                MenuSelectionResult::Selected(SelectControllerMenuEntry::Gamepad(idx), _) => {
                    if self.selected_player == Player::Player1 {
                        state.settings.player1_controller_type = ControllerType::Gamepad(idx as u32);
                    } else {
                        state.settings.player2_controller_type = ControllerType::Gamepad(idx as u32);
                    }

                    let _ = state.settings.save(ctx);

                    let mut new_menu_controller = CombinedMenuController::new();
                    new_menu_controller.add(state.settings.create_player1_controller());
                    new_menu_controller.add(state.settings.create_player2_controller());
                    self.input_busy = true;
                    self.main.non_interactive = true;
                    *controller = new_menu_controller;

                    self.selected_controller = ControllerType::Gamepad(idx as u32);
                    self.update_rebind_menu(state, ctx);

                    self.current = CurrentMenu::MainMenu;
                }
                MenuSelectionResult::Selected(SelectControllerMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    self.current = CurrentMenu::MainMenu;
                }
                _ => {}
            },
            CurrentMenu::RebindMenu => match self.rebind.tick(controller, state) {
                MenuSelectionResult::Selected(RebindMenuEntry::Back, _) | MenuSelectionResult::Canceled => {
                    if !self.input_busy {
                        self.current = CurrentMenu::MainMenu;
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
                MenuSelectionResult::Selected(RebindMenuEntry::Reset, _) => {
                    self.confirm_reset.selected = ConfirmResetMenuEntry::default();
                    self.current = CurrentMenu::ConfirmResetMenu;
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

                            for button in pressed_gamepad_buttons.clone() {
                                if button == Button::Start {
                                    state.sound_manager.play_sfx(5);
                                    self.current = CurrentMenu::RebindMenu;
                                    return Ok(());
                                }
                            }

                            if pressed_gamepad_buttons.len() == 1 {
                                if !self.input_busy {
                                    self.input_busy = true;
                                    self.rebind.non_interactive = true;

                                    let button = *pressed_gamepad_buttons.first().unwrap();

                                    if self.selected_player.controller_type(state) != self.selected_controller {
                                        state.sound_manager.play_sfx(12);
                                    } else {
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
            CurrentMenu::ConfirmResetMenu => match self.confirm_reset.tick(controller, state) {
                MenuSelectionResult::Selected(ConfirmResetMenuEntry::Yes, _) => {
                    self.reset_controls(state, ctx)?;
                    self.update_rebind_menu(state, ctx);
                    self.input_busy = true;
                    self.rebind.non_interactive = true;
                    self.current = CurrentMenu::RebindMenu;
                }
                MenuSelectionResult::Selected(ConfirmResetMenuEntry::No, _) | MenuSelectionResult::Canceled => {
                    self.current = CurrentMenu::RebindMenu;
                }
                _ => {}
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
                self.main.non_interactive = false;
                self.rebind.non_interactive = false;
            }
        }

        Ok(())
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        match self.current {
            CurrentMenu::MainMenu => self.main.draw(state, ctx)?,
            CurrentMenu::SelectControllerMenu => self.select_controller.draw(state, ctx)?,
            CurrentMenu::RebindMenu => self.rebind.draw(state, ctx)?,
            CurrentMenu::ConfirmRebindMenu => self.confirm_rebind.draw(state, ctx)?,
            CurrentMenu::ConfirmResetMenu => self.confirm_reset.draw(state, ctx)?,
        }

        Ok(())
    }
}
