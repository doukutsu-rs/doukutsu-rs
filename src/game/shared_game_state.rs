use std::{cmp, ops::Div};

use chrono::{Datelike, Local};

use crate::common::{ControlFlags, Direction, FadeState};
use crate::components::draw_common::{draw_number, Alignment};
use crate::data::vanilla::VanillaExtractor;
#[cfg(feature = "discord-rpc")]
use crate::discord::DiscordRPC;
use crate::engine_constants::EngineConstants;
use crate::framework::backend::BackendTexture;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::{create_texture_mutable, set_render_target};
use crate::framework::vfs::OpenOptions;
use crate::framework::{filesystem, graphics};
use crate::game::caret::{Caret, CaretType};
use crate::game::npc::NPCTable;
use crate::game::player::TargetPlayer;
use crate::game::profile::{GameProfile, SaveContainer, SaveFormat, SaveSlot};
use crate::game::scripting::tsc::credit_script::{CreditScript, CreditScriptVM};
use crate::game::scripting::tsc::text_script::{
    ScriptMode, TextScript, TextScriptEncoding, TextScriptExecutionState, TextScriptVM,
};
use crate::game::settings::Settings;
use crate::game::stage::StageData;
use crate::graphics::bmfont::BMFont;
use crate::graphics::texture_set::TextureSet;
use crate::i18n::Locale;
use crate::input::touch_controls::TouchControls;
use crate::mod_list::ModList;
use crate::mod_requirements::ModRequirements;
use crate::scene::game_scene::GameScene;
use crate::scene::title_scene::TitleScene;
use crate::scene::Scene;
use crate::sound::SoundManager;
use crate::util::bitvec::BitVec;
use crate::util::rng::XorShift;

use super::filesystem_container::FilesystemContainer;

#[derive(PartialEq, Eq, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum TimingMode {
    _50Hz,
    _60Hz,
    FrameSynchronized,
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

    #[cfg(feature = "backend-glutin")]
    pub fn get_glutin_fullscreen_type(&self) -> Option<glutin::window::Fullscreen> {
        match self {
            WindowMode::Windowed => None,
            WindowMode::Fullscreen => Some(glutin::window::Fullscreen::Borderless(None)),
        }
    }

    pub fn should_display_mouse_cursor(&self) -> bool {
        match self {
            WindowMode::Windowed => true,
            WindowMode::Fullscreen => false,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug, num_derive::FromPrimitive)]
pub enum GameDifficulty {
    Normal = 0,
    Easy = 2,
    Hard = 4,
}

#[derive(PartialEq, Eq, Copy, Clone, num_derive::FromPrimitive)]
pub enum PlayerCount {
    One,
    Two,
}

#[derive(PartialEq, Eq, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum CutsceneSkipMode {
    Hold,
    FastForward,
    Auto,
}

impl GameDifficulty {
    pub fn from_primitive(val: u8) -> GameDifficulty {
        return num_traits::FromPrimitive::from_u8(val).unwrap_or(GameDifficulty::Normal);
    }
}

#[derive(PartialEq, Eq, Copy, Clone, num_derive::FromPrimitive, serde::Serialize, serde::Deserialize)]
pub enum ScreenShakeIntensity {
    Full,
    Half,
    Off,
}

impl ScreenShakeIntensity {
    pub fn to_val(self) -> f64 {
        match self {
            ScreenShakeIntensity::Full => 1.0,
            ScreenShakeIntensity::Half => 0.5,
            ScreenShakeIntensity::Off => 0.0,
        }
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
    should_draw: bool,
}

impl Fps {
    pub fn new() -> Fps {
        Fps { frame_count: 0, fps: 0, tick_count: 0, tps: 0, last_capture: 0, should_draw: true }
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

        if self.should_draw {
            let first = draw_number(state.canvas_size.0 - 8.0, 8.0, self.fps as usize, Alignment::Right, state, ctx);
            let second = draw_number(state.canvas_size.0 - 8.0, 16.0, self.tps as usize, Alignment::Right, state, ctx);
            self.should_draw = first.is_ok() && second.is_ok();
        }

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

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ReplayKind {
    Best,
    Last,
}

impl ReplayKind {
    pub fn get_suffix(&self) -> String {
        match self {
            ReplayKind::Best => ".rep".to_string(),
            ReplayKind::Last => ".last.rep".to_string(),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ReplayState {
    None,
    Recording,
    Playback(ReplayKind),
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

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct PlayerSkinLocation {
    pub texture_index: u16,
    pub offset: u16,
}

impl PlayerSkinLocation {
    pub const fn new(texture_index: u16, offset: u16) -> PlayerSkinLocation {
        PlayerSkinLocation { texture_index, offset }
    }
}

impl Default for PlayerSkinLocation {
    fn default() -> PlayerSkinLocation {
        PlayerSkinLocation::new(0, 0)
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
    pub quake_rumble_counter: u32,
    pub super_quake_rumble_counter: u32,
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
    pub command_line: bool,
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
    pub fs_container: Option<FilesystemContainer>,
    pub constants: EngineConstants,
    pub font: BMFont,
    pub texture_set: TextureSet,
    pub sound_manager: SoundManager,
    pub settings: Settings,
    pub save_slot: usize,
    pub difficulty: GameDifficulty,
    pub player_count: PlayerCount,
    pub player_count_modified_in_game: bool,
    pub player2_skin_location: PlayerSkinLocation,
    pub replay_state: ReplayState,
    pub mod_requirements: ModRequirements,
    pub loc: Locale,
    pub tutorial_counter: u16,
    pub more_rust: bool,
    #[cfg(feature = "discord-rpc")]
    pub discord_rpc: DiscordRPC,
    pub shutdown: bool,
}

impl SharedGameState {
    pub fn new(ctx: &mut Context) -> GameResult<SharedGameState> {
        let mut constants = EngineConstants::defaults();
        let mut sound_manager = SoundManager::new(ctx)?;
        let settings = Settings::load(ctx)?;
        let mod_requirements = ModRequirements::load(ctx)?;

        let vanilla_ext_exe = match option_env!("VANILLA_EXT_EXE") {
            Some(exe_name) => exe_name,
            None => "Doukutsu.exe",
        };

        let vanilla_ext_outdir = match option_env!("VANILLA_EXT_OUTDIR") {
            Some(outdir) => outdir,
            None => "data",
        };

        #[cfg(not(target_os = "horizon"))]
        if let Some(vanilla_extractor) =
            VanillaExtractor::from(ctx, vanilla_ext_exe.to_string(), vanilla_ext_outdir.to_string())
        {
            let result = vanilla_extractor.extract_data();
            if let Err(e) = result {
                log::error!("Failed to extract vanilla data: {}", e);
            }
        }

        if filesystem::exists(ctx, "/base/lighting.tbl") {
            log::info!("Cave Story+ (Switch) data files detected.");
            ctx.size_hint = (854, 480);
            constants.apply_csplus_patches(&mut sound_manager);
            constants.apply_csplus_nx_patches();
            constants.load_nx_stringtable(ctx)?;
        } else if filesystem::exists(ctx, "/base/ogph/SellScreen.bmp") {
            log::info!("WiiWare DEMO data files detected.");
            constants.apply_csplus_patches(&mut sound_manager);
            constants.apply_csdemo_patches();
        } else if filesystem::exists(ctx, "/base/strap_a_en.bmp") {
            log::info!("WiiWare data files detected."); //Missing Challenges and Remastered Soundtrack but identical to CS+ PC otherwise
            constants.apply_csplus_patches(&mut sound_manager);
        } else if filesystem::exists(ctx, "/root/buid_time.txt") {
            log::error!("DSiWare data files detected. !UNSUPPORTED!"); //Freeware 2.0, sprites are arranged VERY differently + separate drowned carets
        } else if filesystem::exists(ctx, "/darken.tex") || filesystem::exists(ctx, "/darken.png") {
            log::error!("EShop data files detected. !UNSUPPORTED!"); //Ditto, drowned carets finally part of mychar, the turning point towards CS+
        } else if filesystem::exists(ctx, "/data/stage3d/") {
            log::error!("CS3D data files detected. !UNSUPPORTED!"); //Sprites are technically all there but filenames differ, + no n3ddta support
        } else if filesystem::exists(ctx, "/base/Nicalis.bmp") || filesystem::exists(ctx, "/base/Nicalis.png") {
            log::info!("Cave Story+ (PC) data files detected.");
            constants.apply_csplus_patches(&mut sound_manager);
        } else if filesystem::exists(ctx, "/mrmap.bin") {
            log::info!("CSE2E data files detected.");
        } else if filesystem::exists(ctx, "/stage.dat") {
            log::info!("NXEngine-evo data files detected.");
        }

        for soundtrack in constants.soundtracks.iter_mut() {
            if filesystem::exists(ctx, &soundtrack.path) {
                log::info!("Enabling soundtrack {} from {}.", soundtrack.id, soundtrack.path);
                soundtrack.available = true;
            }
        }

        let season = Season::current();
        constants.rebuild_path_list(None, season, &settings);

        constants.load_locales(ctx)?;

        let locale = SharedGameState::get_locale(&constants, &settings.locale).unwrap_or_default();
        let font = Self::try_update_locale(&mut constants, &locale, ctx).unwrap();

        let mod_list = ModList::load(ctx, &constants.string_table)?;

        for i in 0..0xffu8 {
            let path = format!("pxt/fx{:02x}.pxt", i);
            if let Ok(file) = filesystem::open_find(ctx, &constants.base_paths, path) {
                sound_manager.set_sample_params_from_file(i, file)?;
                continue;
            }

            let path = format!("PixTone/{:03}.pxt", i);
            if let Ok(file) = filesystem::open_find(ctx, &constants.base_paths, path) {
                sound_manager.set_sample_params_from_file(i, file)?;
                continue;
            }
        }

        sound_manager.set_song_volume(settings.bgm_volume);
        sound_manager.set_sfx_volume(settings.sfx_volume);

        let current_time = Local::now();
        let more_rust = (current_time.month() == 7 && current_time.day() == 7) || settings.more_rust;
        let seed = chrono::Local::now().timestamp() as i32;

        #[cfg(feature = "discord-rpc")]
        let discord_rpc_app_id = match option_env!("DISCORD_RPC_APP_ID") {
            Some(app_id) => app_id,
            None => "1076523467337367622",
        };

        Ok(SharedGameState {
            control_flags: ControlFlags(0),
            game_flags: BitVec::with_size(8000),
            skip_flags: BitVec::with_size(64),
            map_flags: BitVec::with_size(128),
            fade_state: FadeState::Hidden,
            game_rng: XorShift::new(seed),
            effect_rng: XorShift::new(123),
            tile_size: TileSize::Tile16x16,
            quake_counter: 0,
            super_quake_counter: 0,
            quake_rumble_counter: 0,
            super_quake_rumble_counter: 0,
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
            command_line: false,
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
            fs_container: None,
            constants,
            font,
            texture_set: TextureSet::new(),
            sound_manager,
            settings,
            save_slot: 1,
            difficulty: GameDifficulty::Normal,
            player_count: PlayerCount::One,
            player_count_modified_in_game: false,
            player2_skin_location: PlayerSkinLocation::default(),
            replay_state: ReplayState::None,
            mod_requirements,
            loc: locale,
            tutorial_counter: 0,
            more_rust,
            #[cfg(feature = "discord-rpc")]
            discord_rpc: DiscordRPC::new(discord_rpc_app_id),
            shutdown: false,
        })
    }

    pub fn reload_stage_table(&mut self, ctx: &mut Context) -> GameResult {
        let stages = StageData::load_stage_table(
            ctx,
            &self.constants.base_paths,
            self.constants.is_switch,
            self.constants.stage_encoding,
        )?;
        self.stages = stages;
        Ok(())
    }

    pub fn reload_resources(&mut self, ctx: &mut Context) -> GameResult {
        self.constants.rebuild_path_list(self.mod_path.clone(), self.season, &self.settings);
        if !self.constants.is_demo {
            //TODO find a more elegant way to handle this
            self.constants.special_treatment_for_csplus_mods(self.mod_path.as_ref());
        }
        self.constants.load_csplus_tables(ctx)?;
        self.constants.load_animated_faces(ctx)?;
        self.constants.load_texture_size_hints(ctx)?;
        self.reload_stage_table(ctx)?;

        let npc_tbl = filesystem::open_find(ctx, &self.constants.base_paths, "npc.tbl")?;
        let npc_table = NPCTable::load_from(npc_tbl)?;
        self.npc_table = npc_table;

        let head_tsc = filesystem::open_find(ctx, &self.constants.base_paths, "Head.tsc")?;
        let head_script = TextScript::load_from(head_tsc, &self.constants)?;
        self.textscript_vm.set_global_script(head_script);

        let arms_item_tsc = filesystem::open_find(ctx, &self.constants.base_paths, "ArmsItem.tsc")?;
        let arms_item_script = TextScript::load_from(arms_item_tsc, &self.constants)?;
        self.textscript_vm.set_inventory_script(arms_item_script);

        let stage_select_tsc = filesystem::open_find(ctx, &self.constants.base_paths, "StageSelect.tsc")?;
        let stage_select_script = TextScript::load_from(stage_select_tsc, &self.constants)?;
        self.textscript_vm.set_stage_select_script(stage_select_script);

        let substitution_rect_map = [('=', self.constants.textscript.textbox_item_marker_rect)];
        self.textscript_vm.set_substitution_rect_map(substitution_rect_map);

        if filesystem::exists_find(ctx, &self.constants.base_paths, "Credit.tsc") {
            let credit_tsc = filesystem::open_find(ctx, &self.constants.base_paths, "Credit.tsc")?;
            let credit_script = CreditScript::load_from(credit_tsc, &self.constants)?;
            self.creditscript_vm.set_script(credit_script);
        }

        self.texture_set.unload_all();

        self.sound_manager.load_custom_sound_effects(ctx, &self.constants.base_paths)?;

        Ok(())
    }

    pub fn reload_graphics(&mut self) {
        self.constants.rebuild_path_list(self.mod_path.clone(), self.season, &self.settings);
        self.texture_set.unload_all();
    }

    pub fn try_update_locale(
        constants: &mut EngineConstants,
        locale: &Locale,
        ctx: &mut Context,
    ) -> GameResult<BMFont> {
        constants.textscript.encoding = if let Some(encoding) = locale.encoding {
            encoding
        } else {
            // In freeware, Japanese and English text scripts use ShiftJIS.
            // In Cave Story+, Japanese scripts use ShiftJIS and English scripts use UTF-8.
            // The Switch version uses UTF-8 for both English and Japanese fonts.
            match locale.code.as_str() {
                "jp" => {
                    if constants.is_switch {
                        TextScriptEncoding::UTF8
                    } else {
                        TextScriptEncoding::ShiftJIS
                    }
                }
                "en" => {
                    if constants.is_base() {
                        TextScriptEncoding::ShiftJIS
                    } else {
                        TextScriptEncoding::UTF8
                    }
                }
                _ => TextScriptEncoding::UTF8,
            }
        };

        constants.stage_encoding = locale.stage_encoding;

        let font = BMFont::load(&constants.base_paths, &locale.font.path, ctx, locale.font.scale).or_else(|e| {
            log::warn!("Failed to load font, using built-in: {}", e);
            BMFont::load(&vec!["/".to_owned()], "builtin/builtin_font.fnt", ctx, 1.0)
        })?;

        Ok(font)
    }

    pub fn update_locale(&mut self, ctx: &mut Context) {
        let Some(locale) = SharedGameState::get_locale(&self.constants, &self.settings.locale) else {
            return;
        };
        let font = Self::try_update_locale(&mut self.constants, &locale, ctx).unwrap();
        self.loc = locale;
        self.font = font;
        let _ = self.reload_stage_table(ctx);
    }

    pub fn graphics_reset(&mut self) {
        self.texture_set.unload_all();
    }

    pub fn start_new_game(&mut self, ctx: &mut Context) -> GameResult {
        self.reset();

        #[cfg(feature = "discord-rpc")]
        self.discord_rpc.update_difficulty(self.difficulty)?;

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

    pub fn save_game(
        &mut self,
        game_scene: &mut GameScene,
        ctx: &mut Context,
        target_player: Option<TargetPlayer>,
    ) -> GameResult {

/*
        if let Some(save_path) = self.get_save_filename(self.save_slot) {
            if let Ok(data) = filesystem::open_options(ctx, save_path, OpenOptions::new().write(true).create(true)) {
                let profile = GameProfile::dump(self, game_scene, target_player);
                //profile.write_save(data)?;

                let mut save_container = SaveContainer::load(ctx)?;
                save_container.set_profile(None, self.save_slot, profile);
                save_container.write_save(ctx, SaveFormat::Generic, None, None);
            } else {
                log::warn!("Cannot open save file.");
            }
        } else {
            log::info!("Mod has saves disabled.");
        }
*/
        if let Some(slot) = self.get_save_slot(self.save_slot) {
            let profile = GameProfile::dump(self, game_scene, target_player);

            let mut save_container = SaveContainer::load(ctx, self)?;
            save_container.set_profile(slot, profile);
            //save_container.write_save(ctx, state, SaveFormat::Generic, None, None);
            //save_container.write_save(ctx, state, self.settings.save_format, None, None);
            save_container.save(ctx, self)?;
        } else {
            log::info!("Mod has saves disabled.");
        }

        Ok(())
    }

    pub fn load_or_start_game(&mut self, ctx: &mut Context) -> GameResult {
        if let Some(slot) = self.get_save_slot(self.save_slot) {
            if let Ok(save) = SaveContainer::load(ctx, self) {
                if let Some(profile) = save.get_profile(slot) {
                    self.reset();
                    let mut next_scene = GameScene::new(self, ctx, profile.current_map as usize)?;

                    profile.apply(self, &mut next_scene, ctx);

                    #[cfg(feature = "discord-rpc")]
                    self.discord_rpc.update_difficulty(self.difficulty)?;

                    self.next_scene = Some(Box::new(next_scene));
                    return Ok(());
                }
            } else {
                log::warn!("No save game found, starting new one...");
            }
        } else {
            log::info!("Mod has saves disabled, starting new game...");
        }

        self.start_new_game(ctx)?;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.control_flags.0 = 0;
        self.game_flags = BitVec::with_size(8000);
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

        #[cfg(feature = "discord-rpc")]
        self.discord_rpc.dispose();
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
            flag
        } else {
            false
        }
    }

    pub fn reset_skip_flags(&mut self) {
        self.skip_flags = BitVec::with_size(64);
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
            flag
        } else {
            false
        }
    }

    pub fn reset_map_flags(&mut self) {
        self.map_flags = BitVec::with_size(128);
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
            flag
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

    pub fn get_save_slot(&mut self, slot: usize) -> Option<SaveSlot> {
        if let Some(mod_path) = &self.mod_path {
            if let Some(mod_info) = self.mod_list.get_mod_info_from_path(mod_path.clone()) {
                log::debug!("Mod info get save slot: {:?}", mod_info);
                if mod_info.id.starts_with(&"csmod_".to_string()) {
                    if mod_info.save_slot > 0 {
                        return Some(SaveSlot::CSPMod(mod_info.save_slot.try_into().unwrap(), slot));
                    } else if mod_info.save_slot < 0 {
                        // Mods with a negative save set(slot) has saves disabled.
                        return None;
                    }

                    // If mod uses save set 0, saves are stored in the main game set
                } else {
                    return Some(SaveSlot::Mod(mod_info.id.clone(), slot));
                }
            } else {
                return None;
            }
        }

        Some(SaveSlot::MainGame(slot))
    }

    pub fn get_rec_filename(&self) -> String {
        if let Some(mod_path) = &self.mod_path {
            let name = self.mod_list.get_name_from_path(mod_path.to_string());
            return format!("/{}", name);
        } else {
            return "/290".to_string();
        }
    }

    pub fn has_replay_data(&self, ctx: &mut Context, replay_kind: ReplayKind) -> bool {
        filesystem::user_exists(ctx, [self.get_rec_filename(), replay_kind.get_suffix()].join(""))
    }

    pub fn delete_replay_data(&self, ctx: &mut Context, replay_kind: ReplayKind) -> GameResult {
        if self.has_replay_data(ctx, replay_kind) {
            filesystem::user_delete(ctx, [self.get_rec_filename(), replay_kind.get_suffix()].join(""))?;
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
                return 6; // Edgy Quote
            }

            if season == Season::Christmas {
                return 8; // Furry Quote
            }
        }

        return self.difficulty as u16;
    }

    fn get_locale(constants: &EngineConstants, user_locale: &str) -> Option<Locale> {
        let mut out_locale = None;

        for locale in &constants.locales {
            if locale.code == "en" {
                out_locale = Some(locale.clone());
            }

            if locale.code == user_locale {
                out_locale = Some(locale.clone());
                break;
            }
        }

        out_locale
    }

    pub fn get_localized_soundtrack_name(&self, id: &str) -> String {
        if id == "organya" {
            return self.loc.t("soundtrack.organya").to_owned();
        }

        self.constants
            .soundtracks
            .iter()
            .find(|s| s.id == id)
            .map_or_else(|| id.to_owned(), |s| self.loc.t(format!("soundtrack.{}", s.id).as_str()).to_owned())
    }

    pub fn tt(&self, key: &str, args: &[(&str, &str)]) -> String {
        return self.loc.tt(key, args);
    }
}
