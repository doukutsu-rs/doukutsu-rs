use std::io::Read;
use std::ptr::null_mut;


use crate::framework::context::Context;
use crate::framework::error::{GameResult, GameError};


use lua_ffi::{c_int, State, ThreadStatus};

use crate::scene::game_scene::GameScene;
use crate::scripting::doukutsu::Doukutsu;
use crate::shared_game_state::SharedGameState;
use crate::framework::filesystem::File;
use crate::framework::filesystem;

mod doukutsu;
mod player;
mod scene;

pub struct LuaScriptingState {
    state: Option<State>,
    state_ptr: *mut SharedGameState,
    ctx_ptr: *mut Context,
    game_scene: *mut GameScene,
}

pub static REF_ERROR: &str = "Reference went out of scope. DO NOT store/use references to game objects outside the event.";

static BOOT_SCRIPT: &str = include_str!("boot.lua");

fn check_status(status: ThreadStatus, state: &mut State) -> GameResult {
    match status {
        ThreadStatus::Ok | ThreadStatus::Yield => { return Ok(()); }
        _ => {}
    }

    let error = state.to_str(-1).unwrap_or("???");
    match status {
        ThreadStatus::RuntimeError => Err(GameError::EventLoopError(format!("Lua Runtime Error: {}", error))),
        ThreadStatus::SyntaxError => Err(GameError::EventLoopError(format!("Lua Syntax Error: {}", error))),
        ThreadStatus::MemoryError => Err(GameError::EventLoopError(format!("Lua Memory Error: {}", error))),
        ThreadStatus::MsgHandlerError => Err(GameError::EventLoopError(format!("Lua Message Handler Error: {}", error))),
        ThreadStatus::FileError => Err(GameError::EventLoopError(format!("Lua File Error: {}", error))),
        _ => Ok(())
    }
}

fn print(state: &mut State) -> c_int {
    if let Some(msg) = state.to_str(1) {
        log::info!("[Lua] {}", msg);
    }

    0
}

impl LuaScriptingState {
    pub fn new() -> LuaScriptingState {
        LuaScriptingState {
            state: None,
            state_ptr: null_mut(),
            ctx_ptr: null_mut(),
            game_scene: null_mut(),
        }
    }

    pub fn update_refs(&mut self, state: *mut SharedGameState, ctx: *mut Context) {
        self.state_ptr = state;
        self.ctx_ptr = ctx;
    }

    fn load_script(mut state: &mut State, path: &str, mut script: File) -> bool {
        let mut buf = Vec::with_capacity(1024);
        let res = script.read_to_end(&mut buf);

        if let Err(err) = res {
            log::warn!("Error reading script {}: {}", path, err);
            return false;
        }

        let name = format!("@{}", path);
        let res = state.load_buffer(&buf, &name);
        let res = check_status(res, &mut state);
        if let Err(err) = res {
            log::warn!("Error loading script {}: {}", path, err);
            return false;
        }

        state.get_global("doukutsu");
        state.get_field(-1, "_initializeScript");
        state.push_value(-3);

        let res = state.pcall(1, 0, 0);
        if let Err((_, err)) = res {
            log::warn!("Error evaluating script {}: {}", path, err);
            return false;
        }

        log::info!("Successfully loaded Lua script: {}", path);

        true
    }

    pub fn reload_scripts(&mut self, ctx: &mut Context) -> GameResult {
        let mut state = State::new();
        state.open_libs();

        state.push(lua_fn!(print));
        state.set_global("print");

        state.push(Doukutsu { ptr: self as *mut LuaScriptingState });
        state.set_global("__doukutsu");

        let res = state.do_string(BOOT_SCRIPT);
        check_status(res, &mut state)?;

        if filesystem::exists(ctx, "/scripts/") {
            let mut script_count = 0;
            let files = filesystem::read_dir(ctx, "/scripts/")?
                .filter(|f| f.to_string_lossy().to_lowercase().ends_with(".lua"));

            for file in files {
                let path = file.clone();

                match filesystem::open(ctx, file) {
                    Ok(script) => {
                        if LuaScriptingState::load_script(&mut state, path.to_string_lossy().as_ref(), script) {
                            script_count += 1;
                        }
                    }
                    Err(err) => {
                        log::warn!("Error opening script {:?}: {}", path, err);
                    }
                }
            }

            if script_count > 0 {
                log::info!("{} Lua scripts have been loaded.", script_count);
            }
        }

        self.state = Some(state);

        Ok(())
    }
}
