use ggez::{Context, GameResult};

use crate::common::Rect;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::shared_game_state::SharedGameState;

pub enum MenuEntry {
    Active(String),
    Disabled(String),
    Toggle(String, bool),
}

impl MenuEntry {
    pub fn height(&self) -> f64 {
        14.0
    }
}

pub enum MenuSelectionResult<'a> {
    None,
    Canceled,
    Selected(usize, &'a mut MenuEntry),
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
            anim_num: 0,
            anim_wait: 0,
            entries: Vec::new(),
        }
    }

    pub fn push_entry(&mut self, entry: MenuEntry) {
        self.entries.push(entry);
    }

    pub fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;

        let mut rect = Rect::new(0, 0, 0, 0);
        let mut rect2 = Rect::new(0, 0, 0, 0);

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

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "MyChar")?;

        rect.left = QUOTE_FRAMES[self.anim_num as usize] * 16;
        rect.top = 16;
        rect.right = rect.left + 16;
        rect.bottom = rect.top + 16;

        batch.add_rect(self.x as f32,
                       self.y as f32 + 2.0 + (self.selected as f32 * 14.0),
                       &rect);

        batch.draw(ctx)?;

        y = self.y as f32 + 6.0;
        for entry in self.entries.iter() {
            match entry {
                MenuEntry::Active(name) => {
                    state.font.draw_text(name.chars(), self.x as f32 + 20.0, y, &state.constants, &mut state.texture_set, ctx)?;
                }
                MenuEntry::Disabled(name) => {
                    state.font.draw_colored_text(name.chars(), self.x as f32 + 20.0, y, (0xa0, 0xa0, 0xff, 0xff), &state.constants, &mut state.texture_set, ctx)?;
                }
                MenuEntry::Toggle(name, value) => {
                    let value_text = if *value { "ON" } else { "OFF" };
                    let val_text_len = state.font.text_width(value_text.chars(), &state.constants);

                    state.font.draw_text(name.chars(), self.x as f32 + 20.0, y, &state.constants, &mut state.texture_set, ctx)?;

                    state.font.draw_text(value_text.chars(), self.x as f32 + self.width as f32 - val_text_len, y, &state.constants, &mut state.texture_set, ctx)?;
                }
            }

            y += entry.height() as f32;
        }

        Ok(())
    }

    pub fn tick(&mut self, controller: &mut CombinedMenuController, state: &mut SharedGameState) -> MenuSelectionResult {
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
                    match entry {
                        MenuEntry::Active(_) => { break; }
                        MenuEntry::Toggle(_, _) => { break; }
                        _ => {}
                    }
                } else {
                    break;
                }
            }
        }

        let mut y = self.y as f32 + 6.0;
        for (idx, entry) in self.entries.iter_mut().enumerate() {
            let entry_bounds = Rect::new_size(self.x, y as isize, self.width as isize, entry.height() as isize);
            y += entry.height() as f32;

            if !((controller.trigger_ok() && self.selected == idx)
                || state.touch_controls.consume_click_in(entry_bounds)) {
                continue;
            }

            match entry {
                MenuEntry::Active(_) | MenuEntry::Toggle(_, _) => {
                    self.selected = idx;
                    state.sound_manager.play_sfx(18);
                    return MenuSelectionResult::Selected(idx, entry);
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
