use std::ops::Div;

use bitvec::vec::BitVec;
use chrono::{Datelike, Local};

use crate::bmfont_renderer::BMFontRenderer;
use crate::caret::{Caret, CaretType};
use crate::common::{ControlFlags, Direction, FadeState};
use crate::engine_constants::EngineConstants;
use crate::framework::{filesystem, graphics};
use crate::framework::backend::BackendTexture;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::graphics::{create_texture_mutable, set_render_target};
use crate::framework::keyboard::ScanCode;
use crate::framework::vfs::OpenOptions;
#[cfg(feature = "hooks")]
use crate::hooks::init_hooks;
use crate::input::touch_controls::TouchControls;
use crate::npc::NPCTable;
use crate::profile::GameProfile;
use crate::rng::XorShift;
use crate::scene::game_scene::GameScene;
use crate::scene::Scene;
#[cfg(feature = "scripting")]
use crate::scripting::LuaScriptingState;
use crate::settings::Settings;
use crate::sound::SoundManager;
use crate::stage::StageData;
use crate::str;
use crate::text_script::{ScriptMode, TextScriptExecutionState, TextScriptVM};
use crate::texture_set::TextureSet;
use bitvec::array::BitArray;
use crate::scene::title_scene::TitleScene;

#[derive(PartialEq, Eq, Copy, Clone)]
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

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Season {
    None,
    Halloween,
    Christmas,
}

impl Season {
    pub fn current() -> Season {
        let now = Local::now();

        if (now.month() == 10 && now.day() > 25) || (now.month() == 11 && now.day() < 3) {
            Season::Halloween
        } else if (now.month() == 12 && now.day() > 23) || (now.month() == 0 && now.day() < 7) {
            Season::Christmas
        } else {
            Season::None
        }
    }
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
    pub timing_mode: TimingMode,
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
    pub teleporter_slots: Vec<(u16, u16)>,
    pub carets: Vec<Caret>,
    pub touch_controls: TouchControls,
    pub base_path: String,
    pub npc_table: NPCTable,
    pub npc_super_pos: (i32, i32),
    pub npc_curly_target: (i32, i32),
    pub npc_curly_counter: u16,
    pub stages: Vec<StageData>,
    pub frame_time: f64,
    pub debugger: bool,
    pub scale: f32,
    pub canvas_size: (f32, f32),
    pub screen_size: (f32, f32),
    pub next_scene: Option<Box<dyn Scene>>,
    pub textscript_vm: TextScriptVM,
    pub lightmap_canvas: Option<Box<dyn BackendTexture>>,
    pub season: Season,
    pub constants: EngineConstants,
    pub font: BMFontRenderer,
    pub texture_set: TextureSet,
    #[cfg(feature = "scripting")]
    pub lua: LuaScriptingState,
    pub sound_manager: SoundManager,
    pub settings: Settings,
    pub shutdown: bool,
}

impl SharedGameState {
    pub fn new(ctx: &mut Context) -> GameResult<SharedGameState> {
        let mut constants = EngineConstants::defaults();
        let sound_manager = SoundManager::new(ctx)?;
        let mut base_path = "/";
        let settings = Settings::load(ctx)?;

        if filesystem::exists(ctx, "/base/Nicalis.bmp") {
            info!("Cave Story+ (PC) data files detected.");
            constants.apply_csplus_patches(&sound_manager);
            base_path = "/base/";
        } else if filesystem::exists(ctx, "/base/lighting.tbl") {
            info!("Cave Story+ (Switch) data files detected.");
            constants.apply_csplus_patches(&sound_manager);
            constants.apply_csplus_nx_patches();
            base_path = "/base/";
        } else if filesystem::exists(ctx, "/mrmap.bin") {
            info!("CSE2E data files detected.");
        } else if filesystem::exists(ctx, "/stage.dat") {
            info!("NXEngine-evo data files detected.");
        }

        let font = BMFontRenderer::load(base_path, &constants.font_path, ctx)
            .or_else(|_| BMFontRenderer::load("/", "builtin/builtin_font.fnt", ctx))?;
        let season = Season::current();
        let mut texture_set = TextureSet::new(base_path);

        if constants.is_cs_plus {
            texture_set.apply_seasonal_content(season, &settings);
        }

        for i in 0..0xffu8 {
            let path = format!("{}/pxt/fx{:02x}.pxt", base_path, i);
            if let Ok(file) = filesystem::open(ctx, path) {
                sound_manager.set_sample_params_from_file(i, file)?;
                continue;
            }

            let path = format!("/pxt/fx{:02x}.pxt", i);
            if let Ok(file) = filesystem::open(ctx, path) {
                sound_manager.set_sample_params_from_file(i, file)?;
                continue;
            }

            let path = format!("{}/PixTone/{:03}.pxt", base_path, i);
            if let Ok(file) = filesystem::open(ctx, path) {
                sound_manager.set_sample_params_from_file(i, file)?;
                continue;
            }

            let path = format!("/PixTone/{:03}.pxt", i);
            if let Ok(file) = filesystem::open(ctx, path) {
                sound_manager.set_sample_params_from_file(i, file)?;
                continue;
            }
        }

        println!("lookup path: {:#?}", texture_set.paths);

        #[cfg(feature = "hooks")]
        init_hooks();

        Ok(SharedGameState {
            timing_mode: TimingMode::_50Hz,
            control_flags: ControlFlags(0),
            game_flags: bitvec::bitvec![0; 8000],
            skip_flags: bitvec::bitvec![0; 64],
            map_flags: bitvec::bitvec![0; 64],
            fade_state: FadeState::Hidden,
            game_rng: XorShift::new(0),
            effect_rng: XorShift::new(123),
            tile_size: TileSize::Tile16x16,
            quake_counter: 0,
            teleporter_slots: Vec::with_capacity(8),
            carets: Vec::with_capacity(32),
            touch_controls: TouchControls::new(),
            base_path: str!(base_path),
            npc_table: NPCTable::new(),
            npc_super_pos: (0, 0),
            npc_curly_target: (0, 0),
            npc_curly_counter: 0,
            stages: Vec::with_capacity(96),
            frame_time: 0.0,
            debugger: false,
            scale: 2.0,
            screen_size: (640.0, 480.0),
            canvas_size: (320.0, 240.0),
            next_scene: None,
            textscript_vm: TextScriptVM::new(),
            lightmap_canvas: None,
            season,
            constants,
            font,
            texture_set,
            #[cfg(feature = "scripting")]
            lua: LuaScriptingState::new(),
            sound_manager,
            settings,
            shutdown: false,
        })
    }

    pub fn process_debug_keys(&mut self, key_code: ScanCode) {
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
            ScanCode::F12 => self.debugger = !self.debugger,
            _ => {}
        }
    }

    pub fn reload_textures(&mut self) {
        let mut texture_set = TextureSet::new(self.base_path.as_str());

        if self.constants.is_cs_plus {
            texture_set.apply_seasonal_content(self.season, &self.settings);
        }

        self.texture_set = texture_set;
    }

    pub fn graphics_reset(&mut self) {
        self.reload_textures();
    }

    pub fn start_new_game(&mut self, ctx: &mut Context) -> GameResult {
        #[cfg(feature = "scripting")]
            self.lua.reload_scripts(ctx)?;

        let mut next_scene = GameScene::new(self, ctx, self.constants.game.new_game_stage as usize)?;
        next_scene.player1.cond.set_alive(true);
        let (pos_x, pos_y)= self.constants.game.new_game_player_pos;
        next_scene.player1.x = pos_x as i32 * next_scene.stage.map.tile_size.as_int() * 0x200;
        next_scene.player1.y = pos_y as i32 * next_scene.stage.map.tile_size.as_int() * 0x200;

        self.reset_map_flags();
        self.fade_state = FadeState::Hidden;
        self.textscript_vm.state = TextScriptExecutionState::Running(self.constants.game.new_game_event, 0);

        self.next_scene = Some(Box::new(next_scene));

        Ok(())
    }

    pub fn start_intro(&mut self, ctx: &mut Context) -> GameResult {
        #[cfg(feature = "scripting")]
            self.lua.reload_scripts(ctx)?;

        let start_stage_id = self.constants.game.intro_stage as usize;

        if self.stages.len() < start_stage_id {
            log::warn!("Intro scene out of bounds in stage table, skipping to title...");
            self.next_scene = Some(Box::new(TitleScene::new()));
            return Ok(());
        }

        let mut next_scene = GameScene::new(self, ctx, start_stage_id)?;
        next_scene.player1.cond.set_hidden(true);
        let (pos_x, pos_y)= self.constants.game.intro_player_pos;
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
        if let Ok(data) = filesystem::open_options(ctx, "/Profile.dat", OpenOptions::new().write(true).create(true)) {
            let profile = GameProfile::dump(self, game_scene);
            profile.write_save(data)?;
        } else {
            log::warn!("Cannot open save file.");
        }

        Ok(())
    }

    pub fn load_or_start_game(&mut self, ctx: &mut Context) -> GameResult {
        if let Ok(data) = filesystem::user_open(ctx, "/Profile.dat") {
            match GameProfile::load_from_save(data) {
                Ok(profile) => {
                    self.reset();
                    let mut next_scene = GameScene::new(self, ctx, profile.current_map as usize)?;

                    profile.apply(self, &mut next_scene, ctx);

                    #[cfg(feature = "scripting")]
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

        self.start_new_game(ctx)
    }

    pub fn reset(&mut self) {
        self.control_flags.0 = 0;
        self.game_flags = bitvec::bitvec![0; 8000];
        self.fade_state = FadeState::Hidden;
        self.game_rng = XorShift::new(0);
        self.teleporter_slots.clear();
        self.quake_counter = 0;
        self.carets.clear();
        self.textscript_vm.set_mode(ScriptMode::Map);
        self.textscript_vm.suspend = true;
    }

    pub fn handle_resize(&mut self, ctx: &mut Context) -> GameResult {
        self.screen_size = graphics::screen_size(ctx);
        self.scale = self.screen_size.1.div(230.0).floor().max(1.0);
        self.canvas_size = (self.screen_size.0 / self.scale, self.screen_size.1 / self.scale);

        let (width, height) = (self.screen_size.0 as u16, self.screen_size.1 as u16);

        // ensure no texture is bound before destroying them.
        set_render_target(ctx, None)?;
        self.lightmap_canvas = Some(create_texture_mutable(ctx, width, height)?);

        Ok(())
    }

    pub fn tick_carets(&mut self) {
        for caret in self.carets.iter_mut() {
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
        self.timing_mode.get_tps() as f64 * self.settings.speed
    }

    pub fn shutdown(&mut self) {
        self.shutdown = true;
    }

    pub fn set_flag(&mut self, id: usize, value: bool) {
        if id < self.game_flags.len() {
            self.game_flags.set(id, value);
        } else {
            log::warn!("Attempted to set an out-of-bounds flag {}:", id);
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
}
