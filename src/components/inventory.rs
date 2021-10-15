use crate::common::Rect;
use crate::components::draw_common::{draw_number, Alignment};
use crate::entity::GameEntity;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::input::touch_controls::TouchControlType;
use crate::inventory::Inventory;
use crate::player::Player;
use crate::shared_game_state::SharedGameState;
use crate::scripting::tsc::text_script::ScriptMode;
use crate::weapon::{WeaponLevel, WeaponType};

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum InventoryFocus {
    None,
    Weapons,
    Items,
}

#[derive(Copy, Clone)]
struct InvWeaponData {
    wtype: WeaponType,
    level: WeaponLevel,
    ammo: u16,
    max_ammo: u16,
}

pub struct InventoryUI {
    tick: usize,
    text_y_pos: u16,
    selected_weapon: u16,
    selected_item: u16,
    weapon_count: u16,
    item_count: u16,
    weapon_data: [InvWeaponData; 8],
    item_data: [(u16, u16); 32],
    focus: InventoryFocus,
}

impl InventoryUI {
    pub fn new() -> InventoryUI {
        InventoryUI {
            text_y_pos: 16,
            tick: 0,
            selected_weapon: 0,
            selected_item: 0,
            weapon_count: 0,
            item_count: 0,
            weapon_data: [InvWeaponData { wtype: WeaponType::None, level: WeaponLevel::None, ammo: 0, max_ammo: 0 }; 8],
            item_data: [(0u16, 0u16); 32],
            focus: InventoryFocus::None,
        }
    }

    fn get_item_event_number(&self, inventory: &Inventory) -> u16 {
        inventory.get_item_idx(self.selected_item as usize).map(|i| i.0 + 5000).unwrap_or(5000)
    }

    fn get_item_event_number_action(&self, inventory: &Inventory) -> u16 {
        inventory.get_item_idx(self.selected_item as usize).map(|i| i.0 + 6000).unwrap_or(6000)
    }

    fn exit(&mut self, state: &mut SharedGameState, player: &mut Player, inventory: &mut Inventory) {
        self.focus = InventoryFocus::None;
        inventory.current_item = 0;
        self.text_y_pos = 16;
        state.textscript_vm.reset();
        state.textscript_vm.set_mode(ScriptMode::Map);
        player.controller.update_trigger();
    }
}

impl GameEntity<(&mut Context, &mut Player, &mut Inventory)> for InventoryUI {
    fn tick(
        &mut self,
        state: &mut SharedGameState,
        (ctx, player, inventory): (&mut Context, &mut Player, &mut Inventory),
    ) -> GameResult<()> {
        let (off_left, off_top, off_right, _) = crate::framework::graphics::screen_insets_scaled(ctx, state.scale);
        let mut slot_rect =
            Rect::new_size(state.canvas_size.0 as isize - 34 - off_right as isize, 8 + off_top as isize, 26, 26);

        state.touch_controls.control_type = TouchControlType::None;

        if state.control_flags.control_enabled()
            && (player.controller.trigger_inventory()
                || player.controller.trigger_menu_back()
                || (state.settings.touch_controls && state.touch_controls.consume_click_in(slot_rect)))
        {
            self.exit(state, player, inventory);
            return Ok(());
        }

        if self.text_y_pos > 8 {
            self.text_y_pos -= 1;
        }

        self.weapon_count = 0;
        for (idx, weapon) in self.weapon_data.iter_mut().enumerate() {
            if let Some(weapon_data) = inventory.get_weapon(idx) {
                weapon.wtype = weapon_data.wtype;
                weapon.level = weapon_data.level;
                weapon.ammo = weapon_data.ammo;
                weapon.max_ammo = weapon_data.max_ammo;

                self.weapon_count += 1;
            } else {
                weapon.wtype = WeaponType::None;
                break;
            }
        }

        self.item_count = 0;
        for (idx, (item_id, amount)) in self.item_data.iter_mut().enumerate() {
            if let Some(item_data) = inventory.get_item_idx(idx) {
                *item_id = item_data.0;
                *amount = item_data.1;
                self.item_count += 1;
            } else {
                *item_id = 0;
                break;
            }
        }

        fn get_weapon_event_number(inventory: &Inventory) -> u16 {
            inventory.get_current_weapon().map(|w| w.wtype as u16 + 1000).unwrap_or(1000)
        }

        self.selected_item = inventory.current_item;
        self.selected_weapon = inventory.current_weapon;

        let count_x = state.constants.textscript.inventory_item_count_x as u16;

        match self.focus {
            InventoryFocus::None => {
                self.focus = InventoryFocus::Weapons;
                state.textscript_vm.start_script(get_weapon_event_number(inventory));
            }
            InventoryFocus::Weapons if state.control_flags.control_enabled() => {
                if player.controller.trigger_left() {
                    state.sound_manager.play_sfx(4);
                    inventory.prev_weapon();
                    state.textscript_vm.start_script(get_weapon_event_number(inventory));
                }

                if player.controller.trigger_right() {
                    state.sound_manager.play_sfx(4);
                    inventory.next_weapon();
                    state.textscript_vm.start_script(get_weapon_event_number(inventory));
                }

                if player.controller.trigger_up() || player.controller.trigger_down() {
                    self.focus = InventoryFocus::Items;
                    state.textscript_vm.start_script(self.get_item_event_number(inventory));
                }
            }
            InventoryFocus::Items if self.item_count != 0 && state.control_flags.control_enabled() => {
                if player.controller.trigger_left() {
                    state.sound_manager.play_sfx(1);

                    if (self.selected_item % count_x) != 0 {
                        self.selected_item -= 1;
                    } else {
                        self.selected_item += count_x - 1;
                    }

                    state.textscript_vm.start_script(self.get_item_event_number(inventory));
                }

                if player.controller.trigger_right() {
                    match () {
                        _ if self.selected_item == self.item_count + 1 => {
                            self.selected_item = count_x * (self.selected_item / count_x);
                        }
                        _ if (self.selected_item % count_x) + 1 == count_x => {
                            self.selected_item = self.selected_item.saturating_sub(count_x) + 1;
                        }
                        _ => self.selected_item += 1,
                    }

                    state.sound_manager.play_sfx(1);
                    state.textscript_vm.start_script(self.get_item_event_number(inventory));
                }

                if player.controller.trigger_up() {
                    if self.selected_item < count_x {
                        self.focus = InventoryFocus::Weapons;

                        state.sound_manager.play_sfx(4);
                        state.textscript_vm.start_script(get_weapon_event_number(inventory));
                    } else {
                        self.selected_item -= count_x;

                        state.sound_manager.play_sfx(1);
                        state.textscript_vm.start_script(self.get_item_event_number(inventory));
                    }
                }

                if player.controller.trigger_down() {
                    if self.selected_item / 6 == self.item_count.saturating_sub(1) / 6 {
                        self.focus = InventoryFocus::Weapons;

                        state.sound_manager.play_sfx(4);
                        state.textscript_vm.start_script(get_weapon_event_number(inventory));
                    } else {
                        self.selected_item += count_x;

                        state.sound_manager.play_sfx(1);
                        state.textscript_vm.start_script(self.get_item_event_number(inventory));
                    }
                }

                if player.controller.trigger_menu_ok() {
                    state.textscript_vm.start_script(self.get_item_event_number_action(inventory));
                }

                self.selected_item = self.selected_item.min(self.item_count - 1);
                inventory.current_item = self.selected_item;
            }
            _ => {}
        }

        if state.settings.touch_controls && state.control_flags.control_enabled() {
            let x = ((((state.canvas_size.0 - off_left - off_right) - 244.0) / 2.0).floor() + off_left) as isize;
            let y = 8 + off_top as isize;

            for i in 0..self.weapon_count {
                slot_rect = Rect::new_size(x + 12 + i as isize * 40, y + 16, 40, 40);

                if state.touch_controls.consume_click_in(slot_rect) {
                    self.focus = InventoryFocus::Weapons;
                    state.sound_manager.play_sfx(4);
                    self.selected_weapon = i;
                    inventory.current_weapon = i;
                    state.textscript_vm.start_script(get_weapon_event_number(inventory));
                    self.exit(state, player, inventory);
                }
            }

            for i in 0..self.item_count {
                slot_rect =
                    Rect::new_size(x + 12 + (i % count_x) as isize * 32, y + 68 + (i / count_x) as isize * 16, 32, 16);

                if state.touch_controls.consume_click_in(slot_rect) {
                    state.sound_manager.play_sfx(1);

                    if self.focus == InventoryFocus::Items && inventory.current_item == i {
                        state.textscript_vm.start_script(self.get_item_event_number_action(inventory));
                    } else {
                        self.selected_item = i;
                        inventory.current_item = self.selected_item;
                        self.focus = InventoryFocus::Items;
                        state.textscript_vm.start_script(self.get_item_event_number(inventory));
                    }
                }
            }
        }

        self.tick = self.tick.wrapping_add(1);

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context, _frame: &Frame) -> GameResult<()> {
        let mut tmp_rect = Rect { left: 0, top: 0, right: 0, bottom: 0 };
        let (off_left, off_top, off_right, _) = crate::framework::graphics::screen_insets_scaled(ctx, state.scale);
        let x = (((state.canvas_size.0 - off_left - off_right) - 244.0) / 2.0).floor() + off_left;
        let y = 8.0 + off_top;

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "TextBox")?;
        for i in 0..=18 {
            let rect = match i {
                0 => &state.constants.textscript.inventory_rect_top,
                18 => &state.constants.textscript.inventory_rect_bottom,
                _ => &state.constants.textscript.inventory_rect_middle,
            };

            batch.add_rect(x, y + i as f32 * 8.0, rect);
        }

        batch.add_rect(x + 12.0, y + self.text_y_pos as f32, &state.constants.textscript.inventory_text_arms);

        batch.add_rect(x + 12.0, y + 52.0 + self.text_y_pos as f32, &state.constants.textscript.inventory_text_item);

        let (item_cursor_frame, weapon_cursor_frame) = match self.focus {
            InventoryFocus::None => (1, 1),
            InventoryFocus::Weapons => (1, self.tick & 1),
            InventoryFocus::Items => (self.tick & 1, 1),
        };

        batch.add_rect(
            x + 12.0 + self.selected_weapon as f32 * 40.0,
            y + 16.0,
            &state.constants.textscript.cursor_inventory_weapon[weapon_cursor_frame],
        );

        let count_x = state.constants.textscript.inventory_item_count_x as usize;
        batch.add_rect(
            x + 12.0 + (self.selected_item as usize % count_x) as f32 * 32.0,
            y + 68.0 + (self.selected_item as usize / count_x) as f32 * 16.0,
            &state.constants.textscript.cursor_inventory_item[item_cursor_frame],
        );

        for (idx, weapon) in self.weapon_data.iter().enumerate() {
            if weapon.wtype == WeaponType::None {
                break;
            }

            // lv
            batch.add_rect(x + 12.0 + idx as f32 * 40.0, y + 32.0, &Rect::new_size(80, 80, 16, 8));
            // per
            batch.add_rect(x + 12.0 + idx as f32 * 40.0, y + 48.0, &Rect::new_size(72, 48, 8, 8));

            if weapon.max_ammo == 0 {
                batch.add_rect(x + 28.0 + idx as f32 * 40.0, y + 40.0, &Rect::new_size(80, 48, 16, 8));
                batch.add_rect(x + 28.0 + idx as f32 * 40.0, y + 48.0, &Rect::new_size(80, 48, 16, 8));
            }
        }

        batch.draw(ctx)?;

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "ArmsImage")?;
        for (idx, weapon) in self.weapon_data.iter().enumerate() {
            if weapon.wtype == WeaponType::None {
                break;
            }

            tmp_rect.left = (weapon.wtype as u16 % 16) * 16;
            tmp_rect.top = (weapon.wtype as u16 / 16) * 16;
            tmp_rect.right = tmp_rect.left + 16;
            tmp_rect.bottom = tmp_rect.top + 16;

            batch.add_rect(x + 12.0 + idx as f32 * 40.0, y + 16.0, &tmp_rect);
        }

        batch.draw(ctx)?;

        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "ItemImage")?;

        for (idx, (item_id, _amount)) in self.item_data.iter().enumerate() {
            if *item_id == 0 {
                break;
            }

            tmp_rect.left = (*item_id % 8) * 32;
            tmp_rect.top = (*item_id / 8) * 16;
            tmp_rect.right = tmp_rect.left + 32;
            tmp_rect.bottom = tmp_rect.top + 16;

            batch.add_rect(
                x + 12.0 + (idx % count_x) as f32 * 32.0,
                y + 68.0 + (idx / count_x) as f32 * 16.0,
                &tmp_rect,
            );
        }

        batch.draw(ctx)?;

        for (idx, weapon) in self.weapon_data.iter().enumerate() {
            if weapon.wtype == WeaponType::None {
                break;
            }

            draw_number(x + 44.0 + idx as f32 * 40.0, y + 32.0, weapon.level as usize, Alignment::Right, state, ctx)?;

            if weapon.max_ammo != 0 {
                draw_number(
                    x + 44.0 + idx as f32 * 40.0,
                    y + 40.0,
                    weapon.ammo as usize,
                    Alignment::Right,
                    state,
                    ctx,
                )?;
                draw_number(
                    x + 44.0 + idx as f32 * 40.0,
                    y + 48.0,
                    weapon.max_ammo as usize,
                    Alignment::Right,
                    state,
                    ctx,
                )?;
            }
        }

        if state.settings.touch_controls {
            let close_rect = Rect { left: 110, top: 110, right: 128, bottom: 128 };
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "builtin/touch")?;

            batch.add_rect(state.canvas_size.0 - off_right - 30.0, 12.0 + off_top, &close_rect);
            batch.draw(ctx)?;
        }

        Ok(())
    }
}
