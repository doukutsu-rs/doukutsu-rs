use std::cell::Cell;

use crate::common::{Color, Rect};
use crate::components::draw_common::{draw_number, Alignment};
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::menu::save_select_menu::MenuSaveInfo;
use crate::shared_game_state::{GameDifficulty, MenuCharacter, SharedGameState};

pub mod pause_menu;
pub mod save_select_menu;
pub mod settings_menu;

#[allow(dead_code)]
#[derive(Clone)]
pub enum MenuEntry {
    Hidden,
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
}

impl MenuEntry {
    pub fn height(&self) -> f64 {
        match self {
            MenuEntry::Hidden => 0.0,
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
        }
    }

    pub fn selectable(&self) -> bool {
        match self {
            MenuEntry::Hidden => false,
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
        }
    }
}

pub enum MenuSelectionResult<'a> {
    None,
    Canceled,
    Selected(usize, &'a mut MenuEntry),
    Left(usize, &'a mut MenuEntry, i16),
    Right(usize, &'a mut MenuEntry, i16),
}

pub struct Menu {
    pub x: isize,
    pub y: isize,
    pub width: u16,
    pub height: u16,
    pub selected: usize,
    pub entries: Vec<MenuEntry>,
    anim_num: u16,
    anim_wait: u16,
    custom_cursor: Cell<bool>,
    pub draw_cursor: bool,
}

impl Menu {
    pub fn new(x: isize, y: isize, width: u16, height: u16) -> Menu {
        Menu {
            x,
            y,
            width,
            height,
            selected: 0,
            anim_num: 0,
            anim_wait: 0,
            entries: Vec::new(),
            custom_cursor: Cell::new(true),
            draw_cursor: true,
        }
    }

    pub fn push_entry(&mut self, entry: MenuEntry) {
        self.entries.push(entry);
    }

    pub fn update_width(&mut self, state: &SharedGameState) {
        let mut width = self.width as f32;

        for entry in &self.entries {
            match entry {
                MenuEntry::Hidden => {}
                MenuEntry::Active(entry) | MenuEntry::DisabledWhite(entry) | MenuEntry::Disabled(entry) => {
                    let entry_width = state.font.text_width(entry.chars(), &state.constants) + 32.0;
                    width = width.max(entry_width);
                }
                MenuEntry::Toggle(entry, _) => {
                    let mut entry_with_option = entry.clone();
                    entry_with_option.push_str(" ");

                    let longest_option_width = if state.t("common.off").len() > state.t("common.on").len() {
                        state.font.text_width(state.t("common.off").chars(), &state.constants)
                    } else {
                        state.font.text_width(state.t("common.on").chars(), &state.constants)
                    };

                    let entry_width = state.font.text_width(entry_with_option.chars(), &state.constants)
                        + longest_option_width
                        + 32.0;
                    width = width.max(entry_width);
                }
                MenuEntry::Options(entry, _, options) => {
                    let mut entry_with_option = entry.clone();
                    entry_with_option.push_str(" ");

                    let longest_option = options.iter().max_by(|&a, &b| a.len().cmp(&b.len())).unwrap();
                    entry_with_option.push_str(longest_option);

                    let entry_width = state.font.text_width(entry_with_option.chars(), &state.constants) + 32.0;
                    width = width.max(entry_width);
                }
                MenuEntry::DescriptiveOptions(entry, _, options, descriptions) => {
                    let mut entry_with_option = entry.clone();
                    entry_with_option.push_str(" ");

                    let longest_option = options.iter().max_by(|&a, &b| a.len().cmp(&b.len())).unwrap();
                    entry_with_option.push_str(longest_option);

                    let entry_width = state.font.text_width(entry_with_option.chars(), &state.constants) + 32.0;
                    width = width.max(entry_width);

                    let longest_description = descriptions.iter().max_by(|&a, &b| a.len().cmp(&b.len())).unwrap();
                    let description_width = state.font.text_width(longest_description.chars(), &state.constants) + 32.0;
                    width = width.max(description_width);
                }
                MenuEntry::OptionsBar(entry, _) => {
                    let bar_width = if state.constants.is_switch { 81.0 } else { 109.0 };
                    let entry_width = state.font.text_width(entry.chars(), &state.constants) + 32.0 + bar_width;
                    width = width.max(entry_width);
                }
                MenuEntry::SaveData(_) => {}
                MenuEntry::SaveDataSingle(_) => {}
                MenuEntry::NewSave => {}
            }
        }

        width = width.max(16.0);
        self.width = if (width + 4.0) % 8.0 != 0.0 { (width + 4.0 - width % 8.0) as u16 } else { width as u16 };
    }

    pub fn update_height(&mut self) {
        let mut height = 8.0;

        for entry in &self.entries {
            height += entry.height();
        }

        self.height = height.max(16.0) as u16;
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let ui_texture = if state.constants.is_cs_plus { "ui" } else { "TextBox" };
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, ui_texture)?;

        let mut rect;
        let mut rect2;

        rect = state.constants.title.menu_left_top;
        batch.add_rect(self.x as f32 - rect.width() as f32, self.y as f32 - rect.height() as f32, &rect);
        rect = state.constants.title.menu_right_top;
        batch.add_rect(self.x as f32 + self.width as f32, self.y as f32 - rect.height() as f32, &rect);
        rect = state.constants.title.menu_left_bottom;
        batch.add_rect(self.x as f32 - rect.width() as f32, self.y as f32 + self.height as f32, &rect);
        rect = state.constants.title.menu_right_bottom;
        batch.add_rect(self.x as f32 + self.width as f32, self.y as f32 + self.height as f32, &rect);

        rect = state.constants.title.menu_top;
        rect2 = state.constants.title.menu_bottom;
        let mut x = self.x as f32;
        let mut y = self.y as f32;
        let mut width = self.width;
        let mut height = self.height;

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
        y = self.y as f32;

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

        let mut entry_y = 0;

        if !self.entries.is_empty() {
            entry_y = self.entries[0..(self.selected)].iter().map(|e| e.height()).sum::<f64>().max(0.0) as u16;
        }

        if self.draw_cursor {
            if self.custom_cursor.get() {
                if let Ok(batch) = state.texture_set.get_or_load_batch(ctx, &state.constants, "MenuCursor") {
                    rect.left = self.anim_num * 16;
                    rect.top = 16;
                    rect.right = rect.left + 16;
                    rect.bottom = rect.top + 16;

                    batch.add_rect(self.x as f32, self.y as f32 + 3.0 + entry_y as f32, &rect);

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

                batch.add_rect(
                    self.x as f32,
                    self.y as f32 + 4.0 + entry_y as f32,
                    &character_rect[self.anim_num as usize],
                );

                batch.draw(ctx)?;
            }
        }

        y = self.y as f32 + 8.0;
        for entry in &self.entries {
            match entry {
                MenuEntry::Active(name) | MenuEntry::DisabledWhite(name) => {
                    state.font.draw_text(
                        name.chars(),
                        self.x as f32 + 20.0,
                        y,
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;
                }
                MenuEntry::Disabled(name) => {
                    state.font.draw_colored_text(
                        name.chars(),
                        self.x as f32 + 20.0,
                        y,
                        (0xa0, 0xa0, 0xff, 0xff),
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;
                }
                MenuEntry::Toggle(name, value) => {
                    let value_text = if *value { "ON" } else { "OFF" };
                    let name_text_len = state.font.text_width(name.chars(), &state.constants);

                    state.font.draw_text(
                        name.chars(),
                        self.x as f32 + 20.0,
                        y,
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;

                    state.font.draw_text(
                        value_text.chars(),
                        self.x as f32 + 25.0 + name_text_len,
                        y,
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;
                }
                MenuEntry::Options(name, index, value) => {
                    let value_text = if let Some(text) = value.get(*index) { text } else { "???" };
                    let name_text_len = state.font.text_width(name.chars(), &state.constants);

                    state.font.draw_text(
                        name.chars(),
                        self.x as f32 + 20.0,
                        y,
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;

                    state.font.draw_text(
                        value_text.chars(),
                        self.x as f32 + 25.0 + name_text_len,
                        y,
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;
                }
                MenuEntry::DescriptiveOptions(name, index, value, description) => {
                    let value_text = if let Some(text) = value.get(*index) { text } else { "???" };
                    let description_text = if let Some(text) = description.get(*index) { text } else { "???" };
                    let name_text_len = state.font.text_width(name.chars(), &state.constants);

                    state.font.draw_text(
                        name.chars(),
                        self.x as f32 + 20.0,
                        y,
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;

                    state.font.draw_text(
                        value_text.chars(),
                        self.x as f32 + 25.0 + name_text_len,
                        y,
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;

                    state.font.draw_colored_text(
                        description_text.chars(),
                        self.x as f32 + 20.0,
                        y + 16.0,
                        (0xc0, 0xc0, 0xff, 0xff),
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;
                }
                MenuEntry::OptionsBar(name, percent) => {
                    state.font.draw_text(
                        name.chars(),
                        self.x as f32 + 20.0,
                        y,
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
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
                    state.font.draw_text(
                        state.t("menus.save_menu.new").chars(),
                        self.x as f32 + 20.0,
                        y,
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;
                }
                MenuEntry::SaveData(save) | MenuEntry::SaveDataSingle(save) => {
                    let name = &state.stages[save.current_map as usize].name;
                    let bar_width = (save.life as f32 / save.max_life as f32 * 39.0) as u16;
                    let right_edge = self.x as f32 + self.width as f32 - 4.0;

                    // Lifebar
                    let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;

                    batch.add_rect(right_edge - 60.0, y, &Rect::new_size(0, 40, 24, 8));
                    batch.add_rect(right_edge - 36.0, y, &Rect::new_size(24, 40, 40, 8));
                    batch.add_rect(right_edge - 36.0, y, &Rect::new_size(0, 24, bar_width, 8));

                    state.font.draw_text(
                        name.chars(),
                        self.x as f32 + 20.0,
                        y,
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;

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

                        state.font.draw_text(
                            difficulty_name.chars(),
                            self.x as f32 + 20.0,
                            y + 10.0,
                            &state.constants,
                            &mut state.texture_set,
                            ctx,
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
                _ => {}
            }

            y += entry.height() as f32;
        }

        Ok(())
    }

    pub fn tick(
        &mut self,
        controller: &mut CombinedMenuController,
        state: &mut SharedGameState,
    ) -> MenuSelectionResult {
        if controller.trigger_back() {
            state.sound_manager.play_sfx(5);
            return MenuSelectionResult::Canceled;
        }

        if (controller.trigger_up() || controller.trigger_down()) && !self.entries.is_empty() {
            state.sound_manager.play_sfx(1);
            loop {
                if controller.trigger_down() {
                    self.selected += 1;
                    if self.selected == self.entries.len() {
                        self.selected = 0;
                    }
                } else {
                    if self.selected == 0 {
                        self.selected = self.entries.len();
                    }
                    self.selected -= 1;
                }

                if let Some(entry) = self.entries.get(self.selected) {
                    if entry.selectable() {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        let mut y = self.y as f32 + 8.0;
        for (idx, entry) in self.entries.iter_mut().enumerate() {
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
                    if (self.selected == idx && controller.trigger_ok())
                        || state.touch_controls.consume_click_in(entry_bounds) =>
                {
                    state.sound_manager.play_sfx(18);
                    return MenuSelectionResult::Selected(idx, entry);
                }
                MenuEntry::Options(_, _, _) | MenuEntry::OptionsBar(_, _)
                    if (self.selected == idx && controller.trigger_left())
                        || state.touch_controls.consume_click_in(left_entry_bounds) =>
                {
                    state.sound_manager.play_sfx(1);
                    return MenuSelectionResult::Left(self.selected, entry, -1);
                }
                MenuEntry::Options(_, _, _) | MenuEntry::OptionsBar(_, _)
                    if (self.selected == idx && controller.trigger_right())
                        || state.touch_controls.consume_click_in(right_entry_bounds) =>
                {
                    state.sound_manager.play_sfx(1);
                    return MenuSelectionResult::Right(self.selected, entry, 1);
                }
                MenuEntry::DescriptiveOptions(_, _, _, _)
                    if (self.selected == idx && controller.trigger_left())
                        || state.touch_controls.consume_click_in(left_entry_bounds) =>
                {
                    state.sound_manager.play_sfx(1);
                    return MenuSelectionResult::Left(self.selected, entry, -1);
                }
                MenuEntry::DescriptiveOptions(_, _, _, _) | MenuEntry::SaveData(_)
                    if (self.selected == idx && controller.trigger_right())
                        || state.touch_controls.consume_click_in(right_entry_bounds) =>
                {
                    state.sound_manager.play_sfx(1);
                    return MenuSelectionResult::Right(self.selected, entry, 1);
                }
                _ => {}
            }
        }

        self.anim_wait += 1;
        if self.anim_wait > 8 {
            self.anim_wait = 0;

            self.anim_num += 1;
            if self.anim_num >= 4 as u16 {
                self.anim_num = 0;
            }
        }

        MenuSelectionResult::None
    }
}
