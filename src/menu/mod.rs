use std::cell::Cell;

use crate::common::Rect;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::shared_game_state::SharedGameState;

pub mod settings_menu;

pub struct MenuSaveInfo {}

pub enum MenuEntry {
    Hidden,
    Active(String),
    DisabledWhite(String),
    Disabled(String),
    Toggle(String, bool),
    Options(String, usize, Vec<String>),
    SaveData(MenuSaveInfo),
    NewSave,
}

impl MenuEntry {
    pub fn height(&self) -> f64 {
        match self {
            MenuEntry::Hidden => 0.0,
            MenuEntry::Active(_) => 14.0,
            MenuEntry::DisabledWhite(_) => 14.0,
            MenuEntry::Disabled(_) => 14.0,
            MenuEntry::Toggle(_, _) => 14.0,
            MenuEntry::Options(_, _, _) => 14.0,
            MenuEntry::SaveData(_) => 30.0,
            MenuEntry::NewSave => 30.0,
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
            MenuEntry::SaveData(_) => true,
            MenuEntry::NewSave => true,
        }
    }
}

pub enum MenuSelectionResult<'a> {
    None,
    Canceled,
    Selected(usize, &'a mut MenuEntry),
    Left(usize, &'a mut MenuEntry),
    Right(usize, &'a mut MenuEntry),
}

pub struct Menu {
    pub x: isize,
    pub y: isize,
    pub width: u16,
    pub height: u16,
    pub selected: usize,
    pub entries: Vec<MenuEntry>,
    entry_y: u16,
    anim_num: u16,
    anim_wait: u16,
    custom_cursor: Cell<bool>,
}

static QUOTE_FRAMES: [u16; 4] = [0, 1, 0, 2];

impl Menu {
    pub fn new(x: isize, y: isize, width: u16, height: u16) -> Menu {
        Menu {
            x,
            y,
            width,
            height,
            selected: 0,
            entry_y: 0,
            anim_num: 0,
            anim_wait: 0,
            entries: Vec::new(),
            custom_cursor: Cell::new(true),
        }
    }

    pub fn push_entry(&mut self, entry: MenuEntry) {
        self.entries.push(entry);
    }

    pub fn update_height(&mut self) {
        let mut height = 6.0;

        for entry in self.entries.iter() {
            height += entry.height();
        }

        self.height = height.max(6.0) as u16;
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;

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

        if self.custom_cursor.get() {
            if let Ok(batch) = state.texture_set.get_or_load_batch(ctx, &state.constants, "MenuCursor") {
                rect.left = self.anim_num * 16;
                rect.top = 16;
                rect.right = rect.left + 16;
                rect.bottom = rect.top + 16;

                batch.add_rect(self.x as f32, self.y as f32 + 3.0 + self.entry_y as f32, &rect);

                batch.draw(ctx)?;
            } else {
                self.custom_cursor.set(false);
            }
        }

        if !self.custom_cursor.get() {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "MyChar")?;

            rect.left = QUOTE_FRAMES[self.anim_num as usize] * 16;
            rect.top = 16;
            rect.right = rect.left + 16;
            rect.bottom = rect.top + 16;

            batch.add_rect(self.x as f32, self.y as f32 + 2.0 + self.entry_y as f32, &rect);

            batch.draw(ctx)?;
        }

        y = self.y as f32 + 6.0;
        for entry in self.entries.iter() {
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
                    let val_text_len = state.font.text_width(value_text.chars(), &state.constants);

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
                        self.x as f32 + self.width as f32 - val_text_len,
                        y,
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;
                }
                MenuEntry::Options(name, index, value) => {
                    let value_text = if let Some(text) = value.get(*index) { text.as_str() } else { "???" };
                    let val_text_len = state.font.text_width(value_text.chars(), &state.constants);

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
                        self.x as f32 + self.width as f32 - val_text_len,
                        y,
                        &state.constants,
                        &mut state.texture_set,
                        ctx,
                    )?;
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

        if !self.entries.is_empty() {
            self.entry_y = self.entries[0..(self.selected)].iter().map(|e| e.height()).sum::<f64>().max(0.0) as u16;
        }

        let mut y = self.y as f32 + 6.0;
        for (idx, entry) in self.entries.iter_mut().enumerate() {
            let entry_bounds = Rect::new_size(self.x, y as isize, self.width as isize, entry.height() as isize);
            y += entry.height() as f32;

            match entry {
                MenuEntry::Active(_) | MenuEntry::Toggle(_, _) | MenuEntry::Options(_, _, _)
                    if (self.selected == idx && controller.trigger_ok())
                        || state.touch_controls.consume_click_in(entry_bounds) =>
                {
                    state.sound_manager.play_sfx(18);
                    return MenuSelectionResult::Selected(idx, entry);
                }
                MenuEntry::Options(_, _, _) if controller.trigger_left() => {
                    state.sound_manager.play_sfx(1);
                    return MenuSelectionResult::Left(self.selected, entry);
                }
                MenuEntry::Options(_, _, _) if controller.trigger_right() => {
                    state.sound_manager.play_sfx(1);
                    return MenuSelectionResult::Right(self.selected, entry);
                }
                _ => {}
            }
        }

        // todo nikumaru counter support
        self.anim_wait += 1;
        if self.anim_wait > 8 {
            self.anim_wait = 0;

            self.anim_num += 1;
            if self.anim_num >= QUOTE_FRAMES.len() as u16 {
                self.anim_num = 0;
            }
        }

        MenuSelectionResult::None
    }
}
