use itertools::Itertools;

use crate::common::Color;
use crate::common::Rect;
use crate::components::background::Background;
use crate::engine_constants::ExtraSoundtrack;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::game::frame::Frame;
use crate::game::map::Map;
use crate::game::settings::ControllerType;
use crate::game::shared_game_state::{SharedGameState, TileSize};
use crate::game::stage::{BackgroundType, NpcType, Stage, StageData, StageTexturePaths, Tileset};
use crate::graphics::font::Font;
use crate::input::combined_menu_controller::CombinedMenuController;
use crate::scene::title_scene::TitleScene;
use crate::scene::Scene;

#[derive(Clone, Debug)]
pub enum JukeboxSoundtrackKind {
    Organya,
    Extra(ExtraSoundtrack),
    Custom(String),
}

impl JukeboxSoundtrackKind {
    pub fn eq_str(&self, other: &str) -> bool {
        match self {
            JukeboxSoundtrackKind::Organya => other == "organya",
            JukeboxSoundtrackKind::Extra(s) => other == s.id,
            JukeboxSoundtrackKind::Custom(s) => other == s,
        }
    }

    pub fn to_localized_string(&self, state: &mut SharedGameState) -> String {
        match self {
            JukeboxSoundtrackKind::Organya => state.loc.t("soundtrack.organya").to_owned(),
            JukeboxSoundtrackKind::Extra(s) => state.loc.t(format!("soundtrack.{}", s.id).as_str()).to_owned(),
            JukeboxSoundtrackKind::Custom(s) => s.clone(),
        }
    }

    pub fn to_id(&self) -> String {
        match self {
            JukeboxSoundtrackKind::Organya => "organya".to_owned(),
            JukeboxSoundtrackKind::Extra(s) => s.id.clone(),
            JukeboxSoundtrackKind::Custom(s) => s.clone(),
        }
    }
}

pub struct JukeboxScene {
    selected_song: u16,
    song_list: Vec<String>,
    soundtracks: Vec<JukeboxSoundtrackKind>,
    selected_soundtrack: usize,
    controller: CombinedMenuController,
    background: Background,
    frame: Frame,
    stage: Stage,
    textures: StageTexturePaths,
    previous_pause_on_focus_loss_setting: bool,
}

impl JukeboxScene {
    pub fn new() -> JukeboxScene {
        let fake_stage = Stage {
            map: Map { width: 0, height: 0, tiles: vec![], attrib: [0; 0x100], tile_size: TileSize::Tile16x16 },
            data: StageData {
                name: String::new(),
                name_jp: String::new(),
                map: String::new(),
                boss_no: 0,
                tileset: Tileset { name: "0".to_string() },
                pxpack_data: None,
                background: crate::game::stage::Background::new("bkMoon"),
                background_type: BackgroundType::Outside,
                background_color: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
                npc1: NpcType::new("0"),
                npc2: NpcType::new("0"),
            },
        };

        let mut textures = StageTexturePaths::new();
        textures.update(&fake_stage);

        JukeboxScene {
            selected_song: 0,
            song_list: Vec::new(),
            soundtracks: Vec::new(),
            selected_soundtrack: 0,
            controller: CombinedMenuController::new(),
            background: Background::new(),
            frame: Frame::new(),
            stage: fake_stage,
            textures,
            previous_pause_on_focus_loss_setting: true,
        }
    }
}

impl Scene for JukeboxScene {
    fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.controller.add(state.settings.create_player1_controller());
        self.controller.add(state.settings.create_player2_controller());

        self.song_list = state
            .constants
            .music_table
            .iter()
            .filter(|song| !song.contains("fanfale") && !song.contains("ika"))
            .cloned()
            .collect();

        let mut soundtrack_entries = state
            .constants
            .soundtracks
            .iter()
            .filter(|s| s.available)
            .map(|s| JukeboxSoundtrackKind::Extra(s.clone()))
            .collect_vec();
        soundtrack_entries.push(JukeboxSoundtrackKind::Organya);

        if let Ok(dir) = filesystem::read_dir(ctx, "/Soundtracks/") {
            for entry in dir {
                if filesystem::is_dir(ctx, &entry) {
                    let filename = entry.file_name().unwrap().to_string_lossy().to_string();

                    if soundtrack_entries
                        .iter()
                        .find(|s| matches!(s, JukeboxSoundtrackKind::Custom(s) if s == &filename))
                        .is_none()
                    {
                        soundtrack_entries.push(JukeboxSoundtrackKind::Custom(filename.clone()));
                    }
                }
            }
        }

        self.soundtracks = soundtrack_entries.clone();

        let selected_soundtrack_index =
            self.soundtracks.iter().position(|s| s.eq_str(&state.settings.soundtrack)).unwrap_or(0);
        self.selected_soundtrack = selected_soundtrack_index;

        self.previous_pause_on_focus_loss_setting = state.settings.pause_on_focus_loss;
        state.settings.pause_on_focus_loss = false;

        #[cfg(feature = "discord-rpc")]
        state.discord_rpc.set_in_jukebox()?;

        Ok(())
    }

    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.controller.update(state, ctx)?;
        self.controller.update_trigger();

        self.background.tick()?;

        let mut song = self.selected_song as i16
            + if self.controller.trigger_right() {
                1
            } else if self.controller.trigger_left() {
                -1
            } else if self.controller.trigger_down() {
                8
            } else if self.controller.trigger_up() {
                -8
            } else {
                0
            };

        if song < 0 {
            song += self.song_list.len() as i16;
        } else {
            song %= self.song_list.len() as i16;
        };

        self.selected_song = song as u16;

        if self.controller.trigger_ok() {
            let song_id = state
                .constants
                .music_table
                .iter()
                .position(|song_comp| song_comp == &self.song_list[song as usize])
                .unwrap_or(0);

            state.sound_manager.play_song(song_id, &state.constants, &state.settings, ctx, false)?;
        }

        if self.controller.trigger_shift_left() {
            self.selected_soundtrack = self.selected_soundtrack.checked_sub(1).unwrap_or(self.soundtracks.len() - 1);
            state.settings.soundtrack = self.soundtracks[self.selected_soundtrack].to_id();
            state.sound_manager.reload_songs(&state.constants, &state.settings, ctx)?;
        }

        if self.controller.trigger_shift_right() {
            self.selected_soundtrack = (self.selected_soundtrack + 1) % self.soundtracks.len();
            state.settings.soundtrack = self.soundtracks[self.selected_soundtrack].to_id();
            state.sound_manager.reload_songs(&state.constants, &state.settings, ctx)?;
        }

        if self.controller.trigger_back() {
            state.settings.pause_on_focus_loss = self.previous_pause_on_focus_loss_setting;
            state.next_scene = Some(Box::new(TitleScene::new()));
        }

        // todo Touch controls

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.background.draw(state, ctx, &self.frame, &self.textures, &self.stage)?;

        let block_size = 32.0;
        let buffer = 4.0;
        let init_x = (state.canvas_size.0 / 2.0).floor() - (block_size * 4.0) - 10.0;
        let init_y = (state.canvas_size.1 / 2.0).floor() - (block_size * 2.0);

        let num_songs = self.song_list.len();
        let mut rect = Rect { left: 0_u16, top: 0, right: 0, bottom: 0 };

        fn selected(mut rect: Rect<u16>, offset: u16) -> Rect<u16> {
            rect.left += offset;
            rect.right += offset;
            rect
        }

        // Draw Song Boxes
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "uimusic")?;

        for iter in 0..num_songs {
            rect.left = (iter as u16 % 8) * 24;
            rect.top = (iter as u16 / 8) * 24;
            rect.right = rect.left + 24;
            rect.bottom = rect.top + 24;

            batch.add_rect(
                init_x + (iter % 8) as f32 * (block_size + buffer),
                init_y + (iter / 8) as f32 * (block_size + buffer),
                &rect,
            );
        }

        batch.draw(ctx)?;

        // Draw Selection Boxes
        let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "ui")?;

        for iter in 0..num_songs {
            let left = (iter as u16 % 8) as f32 * (block_size + buffer) - 4.0;
            let top = (iter as u16 / 8) as f32 * (block_size + buffer) - 4.0;
            let right = left + block_size - buffer;
            let bottom = top + block_size - buffer;

            let selected_offset = if iter == self.selected_song as usize { 16 } else { 0 };

            batch.add_rect(
                init_x + left,
                init_y + top,
                &selected(state.constants.title.menu_left_top, selected_offset),
            );
            batch.add_rect(
                init_x + right,
                init_y + top,
                &selected(state.constants.title.menu_right_top, selected_offset),
            );
            batch.add_rect(
                init_x + left,
                init_y + bottom,
                &selected(state.constants.title.menu_left_bottom, selected_offset),
            );
            batch.add_rect(
                init_x + right,
                init_y + bottom,
                &selected(state.constants.title.menu_right_bottom, selected_offset),
            );

            let mut rect = state.constants.title.menu_top;
            let mut rect2 = state.constants.title.menu_bottom;
            let mut x = init_x + left + rect.height() as f32;
            let mut y = init_y + top + rect.height() as f32;
            let mut width = (block_size - buffer) as u16 - rect2.height();
            let mut height = (block_size - buffer) as u16 - rect2.height();

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

                batch.add_rect(x, y - rect.height() as f32, &selected(rect, selected_offset));
                batch.add_rect(x, y + block_size - buffer - rect.height() as f32, &selected(rect2, selected_offset));
                x += rect.width() as f32;
            }

            x = init_x + left;
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

                batch.add_rect(x, y, &selected(rect, selected_offset));
                batch.add_rect(x + block_size - buffer, y, &selected(rect2, selected_offset));
                y += rect.height() as f32;
            }
        }

        batch.draw(ctx)?;

        // Write Soundtrack name

        let text = self.soundtracks[self.selected_soundtrack].to_localized_string(state);
        state.font.builder().center(state.canvas_size.0).y(20.0).shadow(true).draw(
            text.as_str(),
            ctx,
            &state.constants,
            &mut state.texture_set,
        )?;

        // Write soundtrack switch indicators

        if state.settings.touch_controls || state.settings.player1_controller_type == ControllerType::Keyboard {
            let prev_chevron = "<";
            let next_chevron = ">";
            let next_chev_width = state.font.builder().compute_width(next_chevron);

            state.font.builder().position(init_x, 20.0).draw(
                prev_chevron,
                ctx,
                &state.constants,
                &mut state.texture_set,
            )?;
            state.font.builder().position(state.canvas_size.0 - init_x - next_chev_width, 20.0).draw(
                next_chevron,
                ctx,
                &state.constants,
                &mut state.texture_set,
            )?;
        } else {
            let batch = state.texture_set.get_or_load_batch(ctx, &state.constants, "buttons")?;

            let gamepad_sprite_offset = match state.settings.player1_controller_type {
                ControllerType::Keyboard => 1,
                ControllerType::Gamepad(index) => ctx.gamepad_context.get_gamepad_sprite_offset(index as usize),
            };

            let prev_rect = state
                .settings
                .player1_controller_button_map
                .prev_weapon
                .get_rect(gamepad_sprite_offset, &state.constants);
            let next_rect = state
                .settings
                .player1_controller_button_map
                .next_weapon
                .get_rect(gamepad_sprite_offset, &state.constants);

            batch.add_rect(init_x, 15.0, &prev_rect);
            batch.add_rect(state.canvas_size.0 - init_x - next_rect.width() as f32, 15.0, &next_rect);

            batch.draw(ctx)?;
        }

        Ok(())
    }
}
