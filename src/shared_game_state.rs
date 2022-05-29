use std::collections::HashMap;
use std::{cmp, ops::Div};

use bitvec::vec::BitVec;
use chrono::{Datelike, Local};

use crate::bmfont_renderer::BMFontRenderer;
use crate::caret::{Caret, CaretType};
use crate::common::{ControlFlags, Direction, FadeState};
use crate::components::draw_common::{draw_number, Alignment};
use crate::engine_constants::EngineConstants;
use crate::framework::backend::BackendTexture;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::{create_texture_mutable, set_render_target};
use crate::framework::keyboard::ScanCode;
use crate::framework::vfs::OpenOptions;
use crate::framework::{filesystem, graphics};
#[cfg(feature = "hooks")]
use crate::hooks::init_hooks;
use crate::i18n::Locale;
use crate::input::touch_controls::TouchControls;
use crate::mod_list::ModList;
use crate::mod_requirements::ModRequirements;
use crate::npc::NPCTable;
use crate::profile::GameProfile;
use crate::rng::XorShift;
use crate::scene::game_scene::GameScene;
use crate::scene::title_scene::TitleScene;
use crate::scene::Scene;
#[cfg(feature = "scripting-lua")]
use crate::scripting::lua::LuaScriptingState;
use crate::scripting::tsc::credit_script::{CreditScript, CreditScriptVM};
use crate::scripting::tsc::text_script::{ScriptMode, TextScript, TextScriptExecutionState, TextScriptVM};
use crate::settings::Settings;
use crate::sound::SoundManager;
use crate::stage::StageData;
use crate::texture_set::TextureSet;

#[derive(PartialEq, Eq, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum TimingMode {
    _50Hz,
    _60Hz,
    FrameSynchronized,
}

#[derive(PartialEq, Eq, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum WindowMode {
    Windowed,
    Fullscreen,
}

impl WindowMode {
    #[cfg(feature = "backend-sdl")]
    pub fn get_sdl2_fullscreen_type(&self) -> sdl2::video::FullscreenType {
        match self {
            WindowMode::Windowed => sdl2::video::FullscreenType::Off,
            WindowMode::Fullscreen => sdl2::video::FullscreenType::Desktop,
        }
    }

    pub fn should_display_mouse_cursor(&self) -> bool {
        match self {
            WindowMode::Windowed => true,
            WindowMode::Fullscreen => false,
        }
    }
}

impl TimingMode {
    pub fn get_delta(self) -> usize {
        match self {
            TimingMode::_50Hz => 1000000000 / 50,
            TimingMode::_60Hz => 1000000000 / 60,
            TimingMode::FrameSynchronized => 0,
        }
    }

    pub fn get_delta_millis(self) -> f64 {
        match self {
            TimingMode::_50Hz => 1000.0 / 50.0,
            TimingMode::_60Hz => 1000.0 / 60.0,
            TimingMode::FrameSynchronized => 0.0,
        }
    }

    pub fn get_tps(self) -> usize {
        match self {
            TimingMode::_50Hz => 50,
            TimingMode::_60Hz => 60,
            TimingMode::FrameSynchronized => 0,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, num_derive::FromPrimitive)]
pub enum GameDifficulty {
    Normal = 0,
    Easy = 2,
    Hard = 4,
}

impl GameDifficulty {
    pub fn from_primitive(val: u8) -> GameDifficulty {
        return num_traits::FromPrimitive::from_u8(val).unwrap_or(GameDifficulty::Normal);
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, num_derive::FromPrimitive, serde::Serialize, serde::Deserialize)]
pub enum Language {
    English,
    Japanese,
}

impl Language {
    pub fn to_language_code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Japanese => "jp",
        }
    }

    pub fn to_string(self) -> String {
        match self {
            Language::English => "English".to_string(),
            Language::Japanese => "Japanese".to_string(),
        }
    }

    pub fn font(self) -> FontData {
        match self {
            Language::English => FontData::new("csfont.fnt".to_owned(), 0.5, 0.0),
            // Use default as fallback if no proper JP font is found
            Language::Japanese => FontData::new("0.fnt".to_owned(), 1.0, 0.0),
        }
    }

    pub fn from_primitive(val: usize) -> Language {
        return num_traits::FromPrimitive::from_usize(val).unwrap_or(Language::English);
    }

    pub fn values() -> Vec<Language> {
        vec![Language::English, Language::Japanese]
    }
}

#[derive(Clone, Debug)]
pub struct FontData {
    pub path: String,
    pub scale: f32,
    pub space_offset: f32,
}

impl FontData {
    pub fn new(path: String, scale: f32, space_offset: f32) -> FontData {
        FontData { path, scale, space_offset }
    }
}

pub struct Fps {
    pub frame_count: u32,
    pub fps: u32,
    pub tick_count: u32,
    pub tps: u32,
    last_capture: u128,
}

impl Fps {
    pub fn new() -> Fps {
        Fps { frame_count: 0, fps: 0, tick_count: 0, tps: 0, last_capture: 0 }
    }

    pub fn act(&mut self, state: &mut SharedGameState, ctx: &mut Context, time: u128) -> GameResult {
        if time - self.last_capture > 1000000000 {
            self.fps = self.frame_count;
            self.frame_count = 0;
            self.tps = self.tick_count;
            self.tick_count = 0;
            self.last_capture = time;
        } else {
            self.frame_count = self.frame_count.saturating_add(1);
        }
        draw_number(state.canvas_size.0 - 8.0, 8.0, self.fps as usize, Alignment::Right, state, ctx)?;
        draw_number(state.canvas_size.0 - 8.0, 16.0, self.tps as usize, Alignment::Right, state, ctx)?;
        Ok(())
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Season {
    None,
    Halloween,
    Christmas,
    PixelBirthday,
}

impl Season {
    pub fn current() -> Season {
        let now = Local::now();

        if (now.month() == 10 && now.day() > 25) || (now.month() == 11 && now.day() < 3) {
            Season::Halloween
        } else if (now.month() == 12 && now.day() > 23) || (now.month() == 0 && now.day() < 7) {
            Season::Christmas
        } else if now.month() == 4 && now.day() == 29 {
            Season::PixelBirthday
        } else {
            Season::None
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum MenuCharacter {
    Quote,
    Curly,
    Toroko,
    King,
    Sue,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ReplayState {
    None,
    Recording,
    Playback,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum TileSize {
    Tile8x8,
    Tile16x16,
}

impl TileSize {
    pub const fn as_float(self) -> f32 {
        match self {
            TileSize::Tile8x8 => 8.0,
            TileSize::Tile16x16 => 16.0,
        }
    }

    pub const fn as_int(self) -> i32 {
        match self {
            TileSize::Tile8x8 => 8,
            TileSize::Tile16x16 => 16,
        }
    }
}

pub struct SharedGameState {
    pub control_flags: ControlFlags,
    pub game_flags: BitVec,
    pub skip_flags: BitVec,
    pub map_flags: BitVec,
    pub fade_state: FadeState,
    /// RNG used by game state, using it for anything else might cause unintended side effects and break replays.
    pub game_rng: XorShift,
    /// RNG used by graphics effects that aren't dependent on game's state.
    pub effect_rng: XorShift,
    pub tile_size: TileSize,
    pub quake_counter: u16,
    pub super_quake_counter: u16,
    pub teleporter_slots: Vec<(u16, u16)>,
    pub carets: Vec<Caret>,
    pub touch_controls: TouchControls,
    pub mod_path: Option<String>,
    pub mod_list: ModList,
    pub npc_table: NPCTable,
    pub npc_super_pos: (i32, i32),
    pub npc_curly_target: (i32, i32),
    pub npc_curly_counter: u16,
    pub water_level: i32,
    pub stages: Vec<StageData>,
    pub frame_time: f64,
    pub debugger: bool,
    pub scale: f32,
    pub canvas_size: (f32, f32),
    pub screen_size: (f32, f32),
    pub preferred_viewport_size: (f32, f32),
    pub next_scene: Option<Box<dyn Scene>>,
    pub textscript_vm: TextScriptVM,
    pub creditscript_vm: CreditScriptVM,
    pub lightmap_canvas: Option<Box<dyn BackendTexture>>,
    pub season: Season,
    pub menu_character: MenuCharacter,
    pub constants: EngineConstants,
    pub font: BMFontRenderer,
    pub texture_set: TextureSet,
    #[cfg(feature = "scripting-lua")]
    pub lua: LuaScriptingState,
    pub sound_manager: SoundManager,
    pub settings: Settings,
    pub save_slot: usize,
    pub difficulty: GameDifficulty,
    pub replay_state: ReplayState,
    pub mod_requirements: ModRequirements,
    pub tutorial_counter: u16,
    pub shutdown: bool,
}

impl SharedGameState {
    pub fn new(ctx: &mut Context) -> GameResult<SharedGameState> {
        let mut constants = EngineConstants::defaults();
        let sound_manager = SoundManager::new(ctx)?;
        let settings = Settings::load(ctx)?;
        let mod_requirements = ModRequirements::load(ctx)?;

        if filesystem::exists(ctx, "/base/lighting.tbl") {
            info!("Cave Story+ (Switch) data files detected.");
            ctx.size_hint = (854, 480);
            constants.apply_csplus_patches(&sound_manager);
            constants.apply_csplus_nx_patches();
            constants.load_nx_stringtable(ctx)?;
        } else if filesystem::exists(ctx, "/base/ogph/SellScreen.bmp") {
            error!("WiiWare DEMO data files detected. !UNSUPPORTED!"); //Missing credits.tsc and crashes due to missing Stage 13 (Start)
        } else if filesystem::exists(ctx, "/base/strap_a_en.bmp") {
            info!("WiiWare data files detected."); //Missing Challenges and Remastered Soundtrack but identical to CS+ PC otherwise
            constants.apply_csplus_patches(&sound_manager);
        } else if filesystem::exists(ctx, "/root/buid_time.txt") {
            error!("DSiWare data files detected. !UNSUPPORTED!"); //Freeware 2.0, sprites are arranged VERY differently + separate drowned carets
        } else if filesystem::exists(ctx, "/darken.tex") || filesystem::exists(ctx, "/darken.png") {
            error!("EShop data files detected. !UNSUPPORTED!"); //Ditto, drowned carets finally part of mychar, the turning point towards CS+
        } else if filesystem::exists(ctx, "/data/stage3d/") {
            error!("CS3D data files detected. !UNSUPPORTED!"); //Sprites are technically all there but filenames differ, + no n3ddta support
        } else if filesystem::exists(ctx, "/base/Nicalis.bmp") || filesystem::exists(ctx, "/base/Nicalis.png") {
            info!("Cave Story+ (PC) data files detected.");
            constants.apply_csplus_patches(&sound_manager);
        } else if filesystem::exists(ctx, "/mrmap.bin") {
            info!("CSE2E data files detected.");
        } else if filesystem::exists(ctx, "/stage.dat") {
            info!("NXEngine-evo data files detected.");
        }

        for soundtrack in constants.soundtracks.iter_mut() {
            if filesystem::exists(ctx, &soundtrack.path) {
                info!("Enabling soundtrack {} from {}.", soundtrack.name, soundtrack.path);
                soundtrack.available = true;
            }
        }

        constants.load_locales(ctx)?;

        let season = Season::current();
        constants.rebuild_path_list(None, season, &settings);

        let active_locale = constants.locales.get(&settings.locale.to_string()).unwrap();

        if constants.is_cs_plus {
            constants.font_scale = active_locale.font.scale;
        }

        let font = BMFontRenderer::load(&constants.base_paths, &active_locale.font.path, ctx).or_else(|e| {
            log::warn!("Failed to load font, using built-in: {}", e);
            BMFontRenderer::load(&vec!["/".to_owned()], "/builtin/builtin_font.fnt", ctx)
        })?;

        let mod_list = ModList::load(ctx, &constants.string_table)?;

        for i in 0..0xffu8 {
            let path = format!("/pxt/fx{:02x}.pxt", i);
            if let Ok(file) = filesystem::open_find(ctx, &constants.base_paths, path) {
                sound_manager.set_sample_params_from_file(i, file)?;
                continue;
            }

            let path = format!("/PixTone/{:03}.pxt", i);
            if let Ok(file) = filesystem::open_find(ctx, &constants.base_paths, path) {
                sound_manager.set_sample_params_from_file(i, file)?;
                continue;
            }
        }

        sound_manager.set_song_volume(settings.bgm_volume);
        sound_manager.set_sfx_volume(settings.sfx_volume);

        #[cfg(feature = "hooks")]
        init_hooks();

        Ok(SharedGameState {
            control_flags: ControlFlags(0),
            game_flags: bitvec::bitvec![0; 8000],
            skip_flags: bitvec::bitvec![0; 64],
            map_flags: bitvec::bitvec![0; 64],
            fade_state: FadeState::Hidden,
            game_rng: XorShift::new(chrono::Local::now().timestamp() as i32),
            effect_rng: XorShift::new(123),
            tile_size: TileSize::Tile16x16,
            quake_counter: 0,
            super_quake_counter: 0,
            teleporter_slots: Vec::with_capacity(8),
            carets: Vec::with_capacity(32),
            touch_controls: TouchControls::new(),
            mod_path: None,
            mod_list,
            npc_table: NPCTable::new(),
            npc_super_pos: (0, 0),
            npc_curly_target: (0, 0),
            npc_curly_counter: 0,
            water_level: 0,
            stages: Vec::with_capacity(96),
            frame_time: 0.0,
            debugger: false,
            scale: 2.0,
            screen_size: (640.0, 480.0),
            canvas_size: (320.0, 240.0),
            preferred_viewport_size: (320.0, 240.0),
            next_scene: None,
            textscript_vm: TextScriptVM::new(),
            creditscript_vm: CreditScriptVM::new(),
            lightmap_canvas: None,
            season,
            menu_character: MenuCharacter::Quote,
            constants,
            font,
            texture_set: TextureSet::new(),
            #[cfg(feature = "scripting-lua")]
            lua: LuaScriptingState::new(),
            sound_manager,
            settings,
            save_slot: 1,
            difficulty: GameDifficulty::Normal,
            replay_state: ReplayState::None,
            mod_requirements,
            tutorial_counter: 0,
            shutdown: false,
        })
    }

    pub fn process_debug_keys(&mut self, key_code: ScanCode) {
        #[cfg(not(debug_assertions))]
        if !self.settings.debug_mode {
            return;
        }

        match key_code {
            ScanCode::F3 => self.settings.god_mode = !self.settings.god_mode,
            ScanCode::F4 => self.settings.infinite_booster = !self.settings.infinite_booster,
            ScanCode::F5 => self.settings.subpixel_coords = !self.settings.subpixel_coords,
            ScanCode::F6 => self.settings.motion_interpolation = !self.settings.motion_interpolation,
            ScanCode::F7 => self.set_speed(1.0),
            ScanCode::F8 => {
                if self.settings.speed > 0.2 {
                    self.set_speed(self.settings.speed - 0.1);
                }
            }
            ScanCode::F9 => {
                if self.settings.speed < 3.0 {
                    self.set_speed(self.settings.speed + 0.1);
                }
            }
            ScanCode::F10 => self.settings.debug_outlines = !self.settings.debug_outlines,
            ScanCode::F11 => self.settings.fps_counter = !self.settings.fps_counter,
            ScanCode::F12 => self.debugger = !self.debugger,
            _ => {}
        }
    }

    pub fn reload_resources(&mut self, ctx: &mut Context) -> GameResult {
        self.constants.rebuild_path_list(self.mod_path.clone(), self.season, &self.settings);
        self.constants.special_treatment_for_csplus_mods(self.mod_path.as_ref());
        self.constants.load_csplus_tables(ctx)?;
        self.constants.load_animated_faces(ctx)?;
        self.constants.load_texture_size_hints(ctx)?;
        let stages = StageData::load_stage_table(ctx, &self.constants.base_paths, self.constants.is_switch)?;
        self.stages = stages;

        let npc_tbl = filesystem::open_find(ctx, &self.constants.base_paths, "/npc.tbl")?;
        let npc_table = NPCTable::load_from(npc_tbl)?;
        self.npc_table = npc_table;

        let head_tsc = filesystem::open_find(ctx, &self.constants.base_paths, "/Head.tsc")?;
        let head_script = TextScript::load_from(head_tsc, &self.constants)?;
        self.textscript_vm.set_global_script(head_script);

        let arms_item_tsc = filesystem::open_find(ctx, &self.constants.base_paths, "/ArmsItem.tsc")?;
        let arms_item_script = TextScript::load_from(arms_item_tsc, &self.constants)?;
        self.textscript_vm.set_inventory_script(arms_item_script);

        let stage_select_tsc = filesystem::open_find(ctx, &self.constants.base_paths, "/StageSelect.tsc")?;
        let stage_select_script = TextScript::load_from(stage_select_tsc, &self.constants)?;
        self.textscript_vm.set_stage_select_script(stage_select_script);

        let credit_tsc = filesystem::open_find(ctx, &self.constants.base_paths, "/Credit.tsc")?;
        let credit_script = CreditScript::load_from(credit_tsc, &self.constants)?;
        self.creditscript_vm.set_script(credit_script);

        self.texture_set.unload_all();

        self.sound_manager.load_custom_sound_effects(ctx, &self.constants.base_paths)?;

        Ok(())
    }

    pub fn reload_graphics(&mut self) {
        self.constants.rebuild_path_list(self.mod_path.clone(), self.season, &self.settings);
        self.texture_set.unload_all();
    }

    pub fn reload_fonts(&mut self, ctx: &mut Context) {
        let active_locale = self.get_active_locale();

        let font = BMFontRenderer::load(&self.constants.base_paths, &active_locale.font.path, ctx)
            .or_else(|e| {
                log::warn!("Failed to load font, using built-in: {}", e);
                BMFontRenderer::load(&vec!["/".to_owned()], "/builtin/builtin_font.fnt", ctx)
            })
            .unwrap();

        if self.constants.is_cs_plus {
            self.constants.font_scale = active_locale.font.scale;
        }

        self.font = font;
    }

    pub fn graphics_reset(&mut self) {
        self.texture_set.unload_all();
    }

    pub fn start_new_game(&mut self, ctx: &mut Context) -> GameResult {
        self.reset();
        #[cfg(feature = "scripting-lua")]
        self.lua.reload_scripts(ctx)?;

        let mut next_scene = GameScene::new(self, ctx, self.constants.game.new_game_stage as usize)?;
        next_scene.player1.cond.set_alive(true);
        let (pos_x, pos_y) = self.constants.game.new_game_player_pos;
        next_scene.player1.x = pos_x as i32 * next_scene.stage.map.tile_size.as_int() * 0x200;
        next_scene.player1.y = pos_y as i32 * next_scene.stage.map.tile_size.as_int() * 0x200;

        self.reset_map_flags();
        self.control_flags.set_control_enabled(true);
        self.control_flags.set_tick_world(true);
        self.fade_state = FadeState::Hidden;
        self.textscript_vm.state = TextScriptExecutionState::Running(self.constants.game.new_game_event, 0);
        self.tutorial_counter = 300;

        self.next_scene = Some(Box::new(next_scene));

        Ok(())
    }

    pub fn start_intro(&mut self, ctx: &mut Context) -> GameResult {
        #[cfg(feature = "scripting-lua")]
        self.lua.reload_scripts(ctx)?;

        let start_stage_id = self.constants.game.intro_stage as usize;

        if self.stages.len() < start_stage_id {
            log::warn!("Intro scene out of bounds in stage table, skipping to title...");
            self.next_scene = Some(Box::new(TitleScene::new()));
            return Ok(());
        }

        let mut next_scene = GameScene::new(self, ctx, start_stage_id)?;
        next_scene.player1.cond.set_hidden(true);
        let (pos_x, pos_y) = self.constants.game.intro_player_pos;
        next_scene.player1.x = pos_x as i32 * next_scene.stage.map.tile_size.as_int() * 0x200;
        next_scene.player1.y = pos_y as i32 * next_scene.stage.map.tile_size.as_int() * 0x200;
        next_scene.intro_mode = true;

        self.reset_map_flags();
        self.fade_state = FadeState::Hidden;
        self.textscript_vm.state = TextScriptExecutionState::Running(self.constants.game.intro_event, 0);

        self.next_scene = Some(Box::new(next_scene));

        Ok(())
    }

    pub fn save_game(&mut self, game_scene: &mut GameScene, ctx: &mut Context) -> GameResult {
        if let Some(save_path) = self.get_save_filename(self.save_slot) {
            if let Ok(data) = filesystem::open_options(ctx, save_path, OpenOptions::new().write(true).create(true)) {
                let profile = GameProfile::dump(self, game_scene);
                profile.write_save(data)?;
            } else {
                log::warn!("Cannot open save file.");
            }
        } else {
            log::info!("Mod has saves disabled.");
        }

        Ok(())
    }

    pub fn load_or_start_game(&mut self, ctx: &mut Context) -> GameResult {
        if let Some(save_path) = self.get_save_filename(self.save_slot) {
            if let Ok(data) = filesystem::user_open(ctx, save_path) {
                match GameProfile::load_from_save(data) {
                    Ok(profile) => {
                        self.reset();
                        let mut next_scene = GameScene::new(self, ctx, profile.current_map as usize)?;

                        profile.apply(self, &mut next_scene, ctx);

                        #[cfg(feature = "scripting-lua")]
                        self.lua.reload_scripts(ctx)?;

                        self.next_scene = Some(Box::new(next_scene));
                        return Ok(());
                    }
                    Err(e) => {
                        log::warn!("Failed to load save game, starting new one: {}", e);
                    }
                }
            } else {
                log::warn!("No save game found, starting new one...");
            }
        } else {
            log::info!("Mod has saves disabled.");
        }

        self.start_new_game(ctx)
    }

    pub fn reset(&mut self) {
        self.control_flags.0 = 0;
        self.game_flags = bitvec::bitvec![0; 8000];
        self.fade_state = FadeState::Hidden;
        self.game_rng = XorShift::new(chrono::Local::now().timestamp() as i32);
        self.teleporter_slots.clear();
        self.quake_counter = 0;
        self.carets.clear();
        self.textscript_vm.set_mode(ScriptMode::Map);
        self.textscript_vm.suspend = true;
    }

    pub fn handle_resize(&mut self, ctx: &mut Context) -> GameResult {
        self.screen_size = graphics::screen_size(ctx);
        let scale_x = self.screen_size.1.div(self.preferred_viewport_size.1).floor().max(1.0);
        let scale_y = self.screen_size.0.div(self.preferred_viewport_size.0).floor().max(1.0);

        self.scale = f32::min(scale_x, scale_y);
        self.canvas_size = (self.screen_size.0 / self.scale, self.screen_size.1 / self.scale);

        let (width, height) = (self.screen_size.0 as u16, self.screen_size.1 as u16);

        // ensure no texture is bound before destroying them.
        set_render_target(ctx, None)?;
        self.lightmap_canvas = Some(create_texture_mutable(ctx, width, height)?);

        Ok(())
    }

    pub fn tick_carets(&mut self) {
        for caret in &mut self.carets {
            caret.tick(&self.effect_rng, &self.constants);
        }

        self.carets.retain(|c| !c.is_dead());
    }

    pub fn create_caret(&mut self, x: i32, y: i32, ctype: CaretType, direct: Direction) {
        self.carets.push(Caret::new(x, y, ctype, direct, &self.constants));
    }

    pub fn set_speed(&mut self, value: f64) {
        self.settings.speed = value.clamp(0.1, 3.0);
        self.frame_time = 0.0;
    }

    pub fn current_tps(&self) -> f64 {
        self.settings.timing_mode.get_tps() as f64 * self.settings.speed
    }

    pub fn shutdown(&mut self) {
        self.shutdown = true;
    }

    // Stops SFX 40/41/58 (CPS and CSS)
    pub fn stop_noise(&mut self) {
        self.sound_manager.stop_sfx(40);
        self.sound_manager.stop_sfx(41);
        self.sound_manager.stop_sfx(58);
    }

    pub fn set_flag(&mut self, id: usize, value: bool) {
        if id < self.game_flags.len() {
            self.game_flags.set(id, value);
        } else {
            log::warn!("Attempted to set an out-of-bounds flag: {} to {}.", id, value);
        }
    }

    pub fn get_flag(&self, id: usize) -> bool {
        if let Some(flag) = self.game_flags.get(id) {
            *flag
        } else {
            false
        }
    }

    pub fn reset_skip_flags(&mut self) {
        self.skip_flags = bitvec::bitvec![0; 64];
    }

    pub fn set_skip_flag(&mut self, id: usize, value: bool) {
        if id < self.skip_flags.len() {
            self.skip_flags.set(id, value);
        } else {
            log::warn!("Attempted to set an out-of-bounds skip flag {}:", id);
        }
    }

    pub fn get_skip_flag(&self, id: usize) -> bool {
        if let Some(flag) = self.skip_flags.get(id) {
            *flag
        } else {
            false
        }
    }

    pub fn reset_map_flags(&mut self) {
        self.map_flags = bitvec::bitvec![0; 128];
    }

    pub fn set_map_flag(&mut self, id: usize, value: bool) {
        if id < self.map_flags.len() {
            self.map_flags.set(id, value);
        } else {
            log::warn!("Attempted to set an out-of-bounds map flag {}:", id);
        }
    }

    pub fn get_map_flag(&self, id: usize) -> bool {
        if let Some(flag) = self.map_flags.get(id) {
            *flag
        } else {
            false
        }
    }

    pub fn get_save_filename(&mut self, slot: usize) -> Option<String> {
        if let Some(mod_path) = &self.mod_path {
            let save_slot = self.mod_list.get_save_from_path(mod_path.to_string());
            if save_slot < 0 {
                return None;
            } else if save_slot > 0 {
                return Some(format!("/Mod{}_Profile{}.dat", save_slot, slot));
            }
        }

        if slot == 1 {
            return Some("/Profile.dat".to_owned());
        } else {
            return Some(format!("/Profile{}.dat", slot));
        }
    }

    pub fn get_rec_filename(&self) -> String {
        if let Some(mod_path) = &self.mod_path {
            let name = self.mod_list.get_name_from_path(mod_path.to_string());
            return format!("/{}", name);
        } else {
            return "/290".to_string();
        }
    }

    pub fn has_replay_data(&self, ctx: &mut Context) -> bool {
        filesystem::user_exists(ctx, [self.get_rec_filename(), ".rep".to_string()].join(""))
    }

    pub fn delete_replay_data(&self, ctx: &mut Context) -> GameResult {
        if self.has_replay_data(ctx) {
            filesystem::user_delete(ctx, [self.get_rec_filename(), ".rep".to_string()].join(""))?;
        }
        Ok(())
    }

    pub fn get_damage(&self, hp: i32) -> i32 {
        match self.difficulty {
            GameDifficulty::Easy => cmp::max(hp / 2, 1),
            GameDifficulty::Normal | GameDifficulty::Hard => hp,
        }
    }

    pub fn get_skinsheet_offset(&self) -> u16 {
        if !self.constants.is_cs_plus {
            return 0;
        }

        if self.settings.seasonal_textures {
            let season = Season::current();

            if season == Season::Halloween {
                return 3; // Edgy Quote
            }

            if season == Season::Christmas {
                return 4; // Furry Quote
            }
        }

        return self.difficulty as u16;
    }

    pub fn get_active_locale(&self) -> &Locale {
        let active_locale = self.constants.locales.get(&self.settings.locale.to_string()).unwrap();
        return active_locale;
    }

    pub fn t(&self, key: &str) -> String {
        return self.get_active_locale().t(key);
    }

    pub fn tt(&self, key: &str, args: HashMap<String, String>) -> String {
        return self.get_active_locale().tt(key, args);
    }
}
