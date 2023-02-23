use std::cell::Cell;

use crate::common::{Color, Rect};
use crate::components::draw_common::{draw_number, Alignment};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::game::shared_game_state::{GameDifficulty, MenuCharacter, SharedGameState};
use crate::graphics::font::Font;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::save_select_menu::MenuSaveInfo;

pub mod controls_menu;
pub mod coop_menu;
pub mod pause_menu;
pub mod save_select_menu;
pub mod settings_menu;

const MENU_MIN_PADDING: f32 = 30.0;

#[derive(Clone, Debug)]
pub enum ControlMenuData {
    String(String),
    Rect(Rect<u16>),
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum MenuEntry {
    Hidden,
    Title(String, bool, bool),    // text, centered, white
    LongText(String, bool, bool), // text, centered, white
    Active(String),
    DisabledWhite(String),
    Disabled(String),
    Toggle(String, bool),
    Options(String, usize, Vec<String>),
    DescriptiveOptions(String, usize, Vec<String>, Vec<String>),
    OptionsBar(String, f32),
    SaveData(MenuSaveInfo),
    SaveDataSingle(MenuSaveInfo),
    NewSave,
    PlayerSkin,
    Control(String, ControlMenuData),
    Spacer(f64),
}

impl MenuEntry {
    pub fn height(&self) -> f64 {
        match self {
            MenuEntry::Hidden => 0.0,
            MenuEntry::Title(_, _, _) => 16.0,    // individual line
            MenuEntry::LongText(_, _, _) => 16.0, // individual line
            MenuEntry::Active(_) => 16.0,
            MenuEntry::DisabledWhite(_) => 16.0,
            MenuEntry::Disabled(_) => 16.0,
            MenuEntry::Toggle(_, _) => 16.0,
            MenuEntry::Options(_, _, _) => 16.0,
            MenuEntry::DescriptiveOptions(_, _, _, _) => 32.0,
            MenuEntry::OptionsBar(_, _) => 16.0,
            MenuEntry::SaveData(_) => 32.0,
            MenuEntry::SaveDataSingle(_) => 32.0,
            MenuEntry::NewSave => 32.0,
            MenuEntry::PlayerSkin => 24.0,
            MenuEntry::Control(_, _) => 16.0,
            MenuEntry::Spacer(height) => *height,
        }
    }

    pub fn selectable(&self) -> bool {
        match self {
            MenuEntry::Hidden => false,
            MenuEntry::Title(_, _, _) => false,
            MenuEntry::LongText(_, _, _) => false,
            MenuEntry::Active(_) => true,
            MenuEntry::DisabledWhite(_) => false,
            MenuEntry::Disabled(_) => false,
            MenuEntry::Toggle(_, _) => true,
            MenuEntry::Options(_, _, _) => true,
            MenuEntry::DescriptiveOptions(_, _, _, _) => true,
            MenuEntry::OptionsBar(_, _) => true,
            MenuEntry::SaveData(_) => true,
            MenuEntry::SaveDataSingle(_) => true,
            MenuEntry::NewSave => true,
            MenuEntry::PlayerSkin => true,
            MenuEntry::Control(_, _) => true,
            MenuEntry::Spacer(_) => false,
        }
    }
}

pub enum MenuSelectionResult<'a, T: std::cmp::PartialEq> {
    None,
    Canceled,
    Selected(T, &'a mut MenuEntry),
    Left(T, &'a mut MenuEntry, i16),
    Right(T, &'a mut MenuEntry, i16),
}

pub struct Menu<T: std::cmp::PartialEq> {
    pub x: isize,
    pub y: isize,
    pub width: u16,
    pub height: u16,
    pub selected: T,
    pub entries: Vec<(T, MenuEntry)>,
    pub height_overrides: Vec<(T, f64)>,
    anim_num: u16,
    anim_wait: u16,
    custom_cursor: Cell<bool>,
    pub draw_cursor: bool,
    pub non_interactive: bool,
    pub center_options: bool,
}

impl<T: std::cmp::PartialEq + std::default::Default + Clone> Menu<T> {
    pub fn new(x: isize, y: isize, width: u16, height: u16) -> Menu<T> {
        Menu {
            x,
            y,
            width,
            height,
            selected: T::default(),
            anim_num: 0,
            anim_wait: 0,
            entries: Vec::new(),
            height_overrides: Vec::new(),
            custom_cursor: Cell::new(true),
            draw_cursor: true,
            non_interactive: false,
            center_options: false,
        }
    }

    pub fn push_entry(&mut self, id: T, entry: MenuEntry) {
        self.entries.push((id, entry));
    }

    pub fn set_entry(&mut self, id: T, entry: MenuEntry) {
        for i in 0..self.entries.len() {
            if self.entries[i].0 == id {
                self.entries[i].1 = entry;
                return;
            }
        }
    }

    pub fn set_id(&mut self, old_id: T, new_id: T) {
        for i in 0..self.entries.len() {
            if self.entries[i].0 == old_id {
                self.entries[i].0 = new_id;
                return;
            }
        }
    }

    pub fn update_width(&mut self, state: &SharedGameState) {
        let mut width = self.width as f32;

        for (_, entry) in &self.entries {
            match entry {
                MenuEntry::Hidden => {}
                MenuEntry::Active(entry) | MenuEntry::DisabledWhite(entry) | MenuEntry::Disabled(entry) => {
                    let entry_width = state.font.builder().compute_width(&entry) + 32.0;
                    width = width.max(entry_width);
                }
                MenuEntry::Title(entry, _, _) | MenuEntry::LongText(entry, _, _) => {
                    let entry_width = state.font.builder().compute_width(&entry).min(state.canvas_size.0) + 32.0;
                    width = width.max(entry_width);
                }
                MenuEntry::Toggle(entry, _) => {
                    let mut entry_with_option = entry.clone();
                    entry_with_option.push_str(" ");

                    let longest_option_width = if state.loc.t("common.off").len() > state.loc.t("common.on").len() {
                        state.font.builder().compute_width(state.loc.t("common.off"))
                    } else {
                        state.font.builder().compute_width(state.loc.t("common.on"))
                    };

                    let entry_width =
                        state.font.builder().compute_width(&entry_with_option) + longest_option_width + 32.0;
                    width = width.max(entry_width);
                }
                MenuEntry::Options(entry, _, options) => {
                    let mut entry_with_option = entry.clone();
                    entry_with_option.push_str(" ");

                    let longest_option = options.iter().max_by(|&a, &b| a.len().cmp(&b.len())).unwrap();
                    entry_with_option.push_str(longest_option);

                    let entry_width = state.font.builder().compute_width(&entry_with_option) + 32.0;
                    width = width.max(entry_width);
                }
                MenuEntry::DescriptiveOptions(entry, _, options, descriptions) => {
                    let mut entry_with_option = entry.clone();
                    entry_with_option.push_str(" ");

                    let longest_option = options.iter().max_by(|&a, &b| a.len().cmp(&b.len())).unwrap();
                    entry_with_option.push_str(longest_option);

                    let entry_width = state.font.builder().compute_width(&entry_with_option) + 32.0;
                    width = width.max(entry_width);

                    let longest_description = descriptions.iter().max_by(|&a, &b| a.len().cmp(&b.len())).unwrap();
                    let description_width = state.font.builder().compute_width(longest_description) + 32.0;
                    width = width.max(description_width);
                }
                MenuEntry::OptionsBar(entry, _) => {
                    let bar_width = if state.constants.is_switch { 81.0 } else { 109.0 };
                    let entry_width = state.font.builder().compute_width(entry) + 32.0 + bar_width;
                    width = width.max(entry_width);
                }
                MenuEntry::SaveData(_) => {}
                MenuEntry::SaveDataSingle(_) => {}
                MenuEntry::NewSave => {}
                MenuEntry::PlayerSkin => {}
                MenuEntry::Control(_, _) => {}
                MenuEntry::Spacer(_) => {}
            }
        }

        width = width.max(16.0).min(state.canvas_size.0 - MENU_MIN_PADDING);
        self.width = if (width + 4.0) % 8.0 != 0.0 { (width + 4.0 - width % 8.0) as u16 } else { width as u16 };
    }

    pub fn update_height(&mut self, state: &SharedGameState) {
        let mut height = 8.0;

        for (id, entry) in &self.entries {
            match entry {
                MenuEntry::Title(text, _, _) | MenuEntry::LongText(text, _, _) => {
                    let text_width = state.font.builder().compute_width(text) + 32.0;
                    let lines = (text_width / state.canvas_size.0).ceil();

                    let actual_entry_height = lines as f64 * entry.height();

                    self.height_overrides.push((id.clone(), actual_entry_height));

                    height += actual_entry_height;
                }
                _ => {
                    height += entry.height();
                }
            }
        }

        self.height = height.max(16.0) as u16;
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let ui_texture = if state.constants.is_cs_plus { "ui" } else { "TextBox" };
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, ui_texture)?;

        let mut rect;
        let mut rect2;

        let selected_y = self.get_selected_entry_y() as f32;

        let mut computed_y = (self.y as f32).max(MENU_MIN_PADDING);

        if (selected_y + MENU_MIN_PADDING) > state.canvas_size.1 - MENU_MIN_PADDING {
            computed_y -= (selected_y + MENU_MIN_PADDING) - (state.canvas_size.1 - MENU_MIN_PADDING) + 4.0;
        }

        let mut x = self.x as f32;
        let mut y = computed_y;
        let mut width = self.width;
        let mut height = self.height;

        rect = state.constants.title.menu_left_top;
        batch.add_rect(self.x as f32 - rect.width() as f32, y - rect.height() as f32, &rect);
        rect = state.constants.title.menu_right_top;
        batch.add_rect(self.x as f32 + self.width as f32, y - rect.height() as f32, &rect);
        rect = state.constants.title.menu_left_bottom;
        batch.add_rect(self.x as f32 - rect.width() as f32, y + self.height as f32, &rect);
        rect = state.constants.title.menu_right_bottom;
        batch.add_rect(self.x as f32 + self.width as f32, y + self.height as f32, &rect);

        rect = state.constants.title.menu_top;
        rect2 = state.constants.title.menu_bottom;

        while width > 0 {
            rect.right = if width >= rect.width() {
                width = width.saturating_sub(rect.width());
                rect.right
            } else {
                let old_width = width;
                width = 0;
                rect.left + old_width
            };
            rect2.right = rect.right;

            batch.add_rect(x, y - rect.height() as f32, &rect);
            batch.add_rect(x, y + self.height as f32, &rect2);
            x += rect.width() as f32;
        }

        x = self.x as f32;
        rect = state.constants.title.menu_left;
        rect2 = state.constants.title.menu_right;
        while height > 0 {
            rect.bottom = if height >= rect.height() {
                height = height.saturating_sub(rect.height());
                rect.bottom
            } else {
                let old_height = height;
                height = 0;
                rect.top + old_height
            };
            rect2.bottom = rect.bottom;

            batch.add_rect(x - rect.width() as f32, y, &rect);
            batch.add_rect(x + self.width as f32, y, &rect2);
            y += rect.height() as f32;
        }

        height = self.height;
        y = computed_y;

        while height > 0 {
            rect = state.constants.title.menu_middle;
            width = self.width;
            x = self.x as f32;

            rect.bottom = if height >= rect.height() {
                height = height.saturating_sub(rect.height());
                rect.bottom
            } else {
                let old_height = height;
                height = 0;
                rect.top + old_height
            };

            while width > 0 {
                rect.right = if width >= rect.width() {
                    width = width.saturating_sub(rect.width());
                    rect.right
                } else {
                    let old_width = width;
                    width = 0;
                    rect.left + old_width
                };

                batch.add_rect(x, y, &rect);

                x += rect.width() as f32;
            }

            y += rect.height() as f32;
        }

        batch.draw(ctx)?;

        let options_x = if self.center_options {
            let mut longest_option_width = 20.0;

            for (_, entry) in &self.entries {
                match entry {
                    MenuEntry::Options(text, _, _) | MenuEntry::Active(text) => {
                        let text_width = state.font.builder().compute_width(text) + 32.0;
                        if text_width > longest_option_width {
                            longest_option_width = text_width;
                        }
                    }
                    _ => {}
                }
            }

            (state.canvas_size.0 / 2.0) - (longest_option_width / 2.0)
        } else {
            self.x as f32
        };

        if self.draw_cursor {
            if self.custom_cursor.get() {
                if let Ok(batch) = state.texture_set.get_or_load_batch(ctx, &state.constants, "MenuCursor") {
                    rect.left = self.anim_num * 16;
                    rect.top = 16;
                    rect.right = rect.left + 16;
                    rect.bottom = rect.top + 16;

                    batch.add_rect(options_x, computed_y + 3.0 + selected_y, &rect);

                    batch.draw(ctx)?;
                } else {
                    self.custom_cursor.set(false);
                }
            }

            if !self.custom_cursor.get() {
                let menu_texture: &str;
                let character_rect: [Rect<u16>; 4];

                match state.menu_character {
                    MenuCharacter::Quote => {
                        menu_texture = "MyChar";
                        character_rect = state.constants.title.cursor_quote;
                    }
                    MenuCharacter::Curly => {
                        menu_texture = "Npc/NpcRegu";
                        character_rect = state.constants.title.cursor_curly;
                    }
                    MenuCharacter::Toroko => {
                        menu_texture = "Npc/NpcRegu";
                        character_rect = state.constants.title.cursor_toroko;
                    }
                    MenuCharacter::King => {
                        menu_texture = "Npc/NpcRegu";
                        character_rect = state.constants.title.cursor_king;
                    }
                    MenuCharacter::Sue => {
                        menu_texture = "Npc/NpcRegu";
                        character_rect = state.constants.title.cursor_sue;
                    }
                }

                let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, menu_texture)?;

                batch.add_rect(options_x, computed_y + 4.0 + selected_y, &character_rect[self.anim_num as usize]);

                batch.draw(ctx)?;
            }
        }

        y = computed_y + 8.0;
        for (_, entry) in &self.entries {
            match entry {
                MenuEntry::Active(name) | MenuEntry::DisabledWhite(name) => {
                    state.font.builder().position(options_x + 20.0, y).draw(
                        name,
                        ctx,
                        &state.constants,
                        &mut state.texture_set,
                    )?;
                }
                MenuEntry::Title(text, is_centered, is_white) | MenuEntry::LongText(text, is_centered, is_white) => {
                    let mut lines = Vec::new();
                    let mut line = String::new();

                    // we should probably abstract this away in some capacity
                    let separator = match state.loc.code.as_str() {
                        "jp" => "",
                        _ => " ",
                    };

                    for word in text.split(separator) {
                        let combined_word = line.clone() + separator + word;
                        let line_length = state.font.builder().compute_width(&combined_word) + 32.0;

                        if line_length > state.canvas_size.0 as f32 {
                            lines.push(line);
                            line = String::new();
                        }

                        line.push_str(word);
                        line.push_str(separator);
                    }

                    lines.push(line);

                    let mut local_y = y;

                    for line in lines.iter() {
                        let x = if *is_centered {
                            (state.canvas_size.0 as f32 - state.font.builder().compute_width(&line)) / 2.0
                        } else {
                            self.x as f32 + 20.0
                        };

                        let mut builder = state.font.builder().position(x, local_y);

                        if !*is_white {
                            builder = builder.color((0xa0, 0xa0, 0xff, 0xff));
                        }

                        builder.draw(&line, ctx, &state.constants, &mut state.texture_set)?;

                        local_y += entry.height() as f32;
                    }

                    y += entry.height() as f32 * (lines.len() - 1) as f32;
                }
                MenuEntry::Disabled(name) => {
                    state.font.builder().position(self.x as f32 + 20.0, y).color((0xa0, 0xa0, 0xff, 0xff)).draw(
                        name,
                        ctx,
                        &state.constants,
                        &mut state.texture_set,
                    )?;
                }
                MenuEntry::Toggle(name, value) => {
                    let value_text = if *value { "ON" } else { "OFF" };
                    let name_text_len = state.font.builder().compute_width(name);

                    state.font.builder().position(self.x as f32 + 20.0, y).draw(
                        name,
                        ctx,
                        &state.constants,
                        &mut state.texture_set,
                    )?;

                    state.font.builder().position(self.x as f32 + 25.0 + name_text_len, y).draw(
                        value_text,
                        ctx,
                        &state.constants,
                        &mut state.texture_set,
                    )?;
                }
                MenuEntry::Options(name, index, value) => {
                    let value_text = if let Some(text) = value.get(*index) { text } else { "???" };
                    let name_text_len = state.font.builder().compute_width(name);

                    state.font.builder().position(options_x + 20.0, y).draw(
                        name,
                        ctx,
                        &state.constants,
                        &mut state.texture_set,
                    )?;

                    state.font.builder().position(options_x + 25.0 + name_text_len, y).draw(
                        value_text,
                        ctx,
                        &state.constants,
                        &mut state.texture_set,
                    )?;
                }
                MenuEntry::DescriptiveOptions(name, index, value, description) => {
                    let value_text = if let Some(text) = value.get(*index) { text } else { "???" };
                    let description_text = if let Some(text) = description.get(*index) { text } else { "???" };
                    let name_text_len = state.font.builder().compute_width(name);

                    state.font.builder().position(self.x as f32 + 20.0, y).draw(
                        name,
                        ctx,
                        &state.constants,
                        &mut state.texture_set,
                    )?;

                    state.font.builder().position(self.x as f32 + 25.0 + name_text_len, y).draw(
                        value_text,
                        ctx,
                        &state.constants,
                        &mut state.texture_set,
                    )?;

                    state
                        .font
                        .builder()
                        .position(self.x as f32 + 20.0, y + 16.0)
                        .color((0xc0, 0xc0, 0xff, 0xff))
                        .draw(description_text, ctx, &state.constants, &mut state.texture_set)?;
                }
                MenuEntry::OptionsBar(name, percent) => {
                    state.font.builder().position(self.x as f32 + 20.0, y).draw(
                        name,
                        ctx,
                        &state.constants,
                        &mut state.texture_set,
                    )?;

                    if state.constants.is_switch || state.constants.is_cs_plus {
                        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "ui")?;
                        let bar_width = if state.constants.is_switch { 81.0 } else { 109.0 };

                        let rect = Rect::new(0, 18, (bar_width - (bar_width * (1.0 - percent))) as u16, 32);

                        batch.add_rect(
                            (self.x + self.width as isize) as f32 - (bar_width + (2.0 * state.scale)),
                            y - (state.scale * 2.0),
                            &rect,
                        );
                        batch.draw(ctx)?;
                    } else {
                        let scale = state.scale;

                        let bar_rect = Rect::new_size(
                            ((self.x + self.width as isize - 80) as f32 * scale) as isize,
                            (y * scale) as isize,
                            (75.0 * scale * percent) as isize,
                            (8.0 * scale) as isize,
                        );

                        graphics::draw_rect(
                            ctx,
                            Rect::new_size(
                                bar_rect.left + (2.0 * scale) as isize,
                                bar_rect.top + (2.0 * scale) as isize,
                                (75.0 * scale) as isize,
                                (8.0 * scale) as isize,
                            ),
                            Color::new(0.0, 0.0, 0.0, 1.0),
                        )?;

                        graphics::draw_rect(ctx, bar_rect, Color::new(1.0, 1.0, 1.0, 1.0))?;
                    }
                }
                MenuEntry::NewSave => {
                    state.font.builder().position(self.x as f32 + 20.0, y).draw(
                        state.loc.t("menus.save_menu.new"),
                        ctx,
                        &state.constants,
                        &mut state.texture_set,
                    )?;
                }
                MenuEntry::PlayerSkin => {
                    state.font.builder().position(self.x as f32 + 20.0, y).draw(
                        state.loc.t("menus.skin_menu.label"),
                        ctx,
                        &state.constants,
                        &mut state.texture_set,
                    )?;

                    let spritesheet_name =
                        state.constants.player_skin_paths[state.player2_skin_location.texture_index as usize].as_str();

                    let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, spritesheet_name)?;
                    batch.add_rect(
                        self.x as f32 + 88.0,
                        y - 4.0,
                        &Rect::new_size(0, (state.player2_skin_location.offset).saturating_mul(2 * 16), 16, 16),
                    );
                    batch.draw(ctx)?;
                }
                MenuEntry::SaveData(save) | MenuEntry::SaveDataSingle(save) => {
                    let valid_save = state.stages.get(save.current_map as usize).is_some();
                    let name = if valid_save {
                        state.stages.get(save.current_map as usize).unwrap().name.as_str()
                    } else {
                        state.loc.t("menus.save_menu.invalid_save")
                    };
                    let bar_width = (save.life as f32 / save.max_life as f32 * 39.0) as u16;
                    let right_edge = self.x as f32 + self.width as f32 - 4.0;

                    state.font.builder().position(self.x as f32 + 20.0, y).draw(
                        name,
                        ctx,
                        &state.constants,
                        &mut state.texture_set,
                    )?;

                    if valid_save {
                        // Lifebar
                        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;

                        batch.add_rect(right_edge - 60.0, y, &Rect::new_size(0, 40, 24, 8));
                        batch.add_rect(right_edge - 36.0, y, &Rect::new_size(24, 40, 40, 8));
                        batch.add_rect(right_edge - 36.0, y, &Rect::new_size(0, 24, bar_width, 8));

                        // Difficulty
                        if state.constants.is_cs_plus {
                            let difficulty = GameDifficulty::from_primitive(save.difficulty);

                            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "MyChar")?;
                            batch.add_rect(
                                self.x as f32 + 20.0,
                                y + 10.0,
                                &Rect::new_size(0, (difficulty as u16).saturating_mul(2 * 16), 16, 16),
                            );
                            batch.draw(ctx)?;
                        } else {
                            let mut difficulty_name: String = "Difficulty: ".to_owned();

                            match save.difficulty {
                                0 => difficulty_name.push_str("Normal"),
                                2 => difficulty_name.push_str("Easy"),
                                4 => difficulty_name.push_str("Hard"),
                                _ => difficulty_name.push_str("(unknown)"),
                            }

                            state.font.builder().position(self.x as f32 + 20.0, y + 10.0).draw(
                                difficulty_name.as_str(),
                                ctx,
                                &state.constants,
                                &mut state.texture_set,
                            )?;
                        }

                        // Weapons
                        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "ArmsImage")?;

                        for weapon_slot in 0..save.weapon_count {
                            let wtype = save.weapon_id[weapon_slot];
                            let pos_x = weapon_slot as f32 * 16.0 - (16 * save.weapon_count.saturating_sub(4)) as f32;
                            let mut rect = Rect::new(0, 0, 0, 16);
                            if wtype != 0 {
                                rect.left = wtype as u16 * 16;
                                rect.right = rect.left + 16;
                                batch.add_rect(right_edge + pos_x - 60.0, y + 8.0, &rect);
                            }
                        }

                        batch.draw(ctx)?;

                        draw_number(right_edge - 36.0, y, save.life as usize, Alignment::Right, state, ctx)?;
                    }
                }
                MenuEntry::Control(name, data) => {
                    state.font.builder().position(self.x as f32 + 20.0, y).draw(
                        name,
                        ctx,
                        &state.constants,
                        &mut state.texture_set,
                    )?;

                    match data {
                        ControlMenuData::String(value) => {
                            let text_width = state.font.builder().compute_width(value);

                            state
                                .font
                                .builder()
                                .position(self.x as f32 + self.width as f32 - 5.0 - text_width, y)
                                .draw(value, ctx, &state.constants, &mut state.texture_set)?;
                        }
                        ControlMenuData::Rect(value) => {
                            let rect_width = value.width() as f32;
                            let y = y + rect.height() as f32 / 2.0 - state.font.line_height() + 4.0;

                            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "buttons")?;
                            batch.add_rect(self.x as f32 + self.width as f32 - 5.0 - rect_width, y, &value);
                            batch.draw(ctx)?;
                        }
                    }
                }
                _ => {}
            }

            y += entry.height() as f32;
        }

        if self.height as f32 > state.canvas_size.1 && self.selected != self.entries.last().unwrap().0 {
            // draw down triangle
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "triangles")?;
            batch.add_rect(self.x as f32 + 6.0, state.canvas_size.1 - 10.0, &Rect::new_size(0, 0, 5, 5));
            batch.draw(ctx)?;
        }

        if computed_y < 0.0 {
            // draw up triangle
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "triangles")?;
            batch.add_rect(self.x as f32 + 6.0, 7.0, &Rect::new_size(5, 0, 5, 5));
            batch.draw(ctx)?;
        }

        Ok(())
    }

    pub fn tick(
        &mut self,
        controller: &mut CombinedMenuController,
        state: &mut SharedGameState,
    ) -> MenuSelectionResult<T> {
        self.anim_wait += 1;
        if self.anim_wait > 8 {
            self.anim_wait = 0;

            self.anim_num += 1;
            if self.anim_num >= 4 as u16 {
                self.anim_num = 0;
            }
        }

        if self.non_interactive {
            return MenuSelectionResult::None;
        }

        if controller.trigger_back() {
            state.sound_manager.play_sfx(5);
            return MenuSelectionResult::Canceled;
        }

        if (controller.trigger_up() || controller.trigger_down()) && !self.entries.is_empty() {
            state.sound_manager.play_sfx(1);

            let mut selected = self.entries.iter().position(|(idx, _)| *idx == self.selected).ok_or(0).unwrap();

            loop {
                if controller.trigger_down() {
                    selected += 1;
                    if selected == self.entries.len() {
                        selected = 0;
                    }
                } else {
                    if selected == 0 {
                        selected = self.entries.len();
                    }
                    selected -= 1;
                }

                if let Some((id, entry)) = self.entries.get(selected) {
                    if entry.selectable() {
                        self.selected = id.clone();
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        let mut y = self.y as f32 + 8.0;
        for (id, entry) in self.entries.iter_mut() {
            let idx = id.clone();
            let entry_bounds = Rect::new_size(self.x, y as isize, self.width as isize, entry.height() as isize);
            let right_entry_bounds =
                Rect::new_size(self.x + self.width as isize, y as isize, self.width as isize, entry.height() as isize);
            let left_entry_bounds =
                Rect::new_size(self.x - self.width as isize, y as isize, self.width as isize, entry.height() as isize);
            y += entry.height() as f32;

            match entry {
                MenuEntry::Active(_)
                | MenuEntry::Toggle(_, _)
                | MenuEntry::Options(_, _, _)
                | MenuEntry::DescriptiveOptions(_, _, _, _)
                | MenuEntry::SaveData(_)
                | MenuEntry::NewSave
                | MenuEntry::PlayerSkin
                    if (self.selected == idx && controller.trigger_ok())
                        || state.touch_controls.consume_click_in(entry_bounds) =>
                {
                    state.sound_manager.play_sfx(18);
                    self.selected = idx.clone();
                    return MenuSelectionResult::Selected(idx, entry);
                }
                MenuEntry::Options(_, _, _) | MenuEntry::OptionsBar(_, _)
                    if (self.selected == idx && controller.trigger_left())
                        || state.touch_controls.consume_click_in(left_entry_bounds) =>
                {
                    state.sound_manager.play_sfx(1);
                    return MenuSelectionResult::Left(self.selected.clone(), entry, -1);
                }
                MenuEntry::Options(_, _, _) | MenuEntry::OptionsBar(_, _)
                    if (self.selected == idx && controller.trigger_right())
                        || state.touch_controls.consume_click_in(right_entry_bounds) =>
                {
                    state.sound_manager.play_sfx(1);
                    return MenuSelectionResult::Right(self.selected.clone(), entry, 1);
                }
                MenuEntry::DescriptiveOptions(_, _, _, _)
                    if (self.selected == idx && controller.trigger_left())
                        || state.touch_controls.consume_click_in(left_entry_bounds) =>
                {
                    state.sound_manager.play_sfx(1);
                    return MenuSelectionResult::Left(self.selected.clone(), entry, -1);
                }
                MenuEntry::DescriptiveOptions(_, _, _, _) | MenuEntry::SaveData(_)
                    if (self.selected == idx && controller.trigger_right())
                        || state.touch_controls.consume_click_in(right_entry_bounds) =>
                {
                    state.sound_manager.play_sfx(1);
                    return MenuSelectionResult::Right(self.selected.clone(), entry, 1);
                }
                MenuEntry::Control(_, _) => {
                    if self.selected == idx && controller.trigger_ok()
                        || state.touch_controls.consume_click_in(entry_bounds)
                    {
                        state.sound_manager.play_sfx(18);
                        self.selected = idx.clone();
                        return MenuSelectionResult::Selected(idx, entry);
                    }
                }
                _ => {}
            }
        }

        MenuSelectionResult::None
    }

    fn get_selected_entry_y(&self) -> u16 {
        let mut entry_y: u16 = 0;

        if !self.entries.is_empty() {
            let mut sum = 0.0;

            for (id, entry) in &self.entries {
                if *id == self.selected {
                    break;
                }

                let entry_height = match self.height_overrides.iter().find(|(entry_id, _)| *entry_id == *id) {
                    Some((_, height)) => *height,
                    None => entry.height(),
                };

                sum += entry_height;
            }

            entry_y = sum as u16;
        }

        entry_y
    }
}
