use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io;
use std::io::Cursor;
use std::io::Seek;
use std::io::SeekFrom;
use std::ops::Not;
use std::rc::Rc;

use num_traits::{clamp, FromPrimitive};

use crate::bitfield;
use crate::common::Direction::{Left, Right};
use crate::common::{Direction, FadeDirection, FadeState, Rect};
use crate::engine_constants::EngineConstants;
use crate::entity::GameEntity;
use crate::frame::UpdateTarget;
use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::input::touch_controls::TouchControlType;
use crate::npc::NPC;
use crate::player::{ControlMode, TargetPlayer};
use crate::scene::game_scene::GameScene;
use crate::scene::title_scene::TitleScene;
use crate::scripting::tsc::bytecode_utils::read_cur_varint;
use crate::scripting::tsc::encryption::decrypt_tsc;
use crate::scripting::tsc::opcodes::TSCOpCode;
use crate::shared_game_state::ReplayState;
use crate::shared_game_state::SharedGameState;
use crate::weapon::WeaponType;

bitfield! {
    pub struct TextScriptFlags(u16);
    impl Debug;
    pub render, set_render: 0;
    pub background_visible, set_background_visible: 1;
    pub fast, set_fast: 4;
    pub position_top, set_position_top: 5;
    pub perma_fast, set_perma_fast: 6;
    pub cutscene_skip, set_cutscene_skip: 7;
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum TextScriptEncoding {
    UTF8 = 0,
    ShiftJIS,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum TextScriptLine {
    Line1 = 0,
    Line2,
    Line3,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum ConfirmSelection {
    Yes,
    No,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum ScriptMode {
    Map,
    Inventory,
    StageSelect,
}

impl Not for ConfirmSelection {
    type Output = ConfirmSelection;

    fn not(self) -> ConfirmSelection {
        if self == ConfirmSelection::Yes {
            ConfirmSelection::No
        } else {
            ConfirmSelection::Yes
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TextScriptExecutionState {
    Ended,
    Running(u16, u32),
    Msg(u16, u32, u32, u8),
    MsgNewLine(u16, u32, u32, u8, u8),
    WaitTicks(u16, u32, u16),
    WaitInput(u16, u32, u16),
    WaitStanding(u16, u32),
    WaitConfirmation(u16, u32, u16, u8, ConfirmSelection),
    WaitFade(u16, u32),
    FallingIsland(u16, u32, i32, i32, u16, bool),
    MapSystem,
    SaveProfile(u16, u32),
    LoadProfile,
    Reset,
}

#[derive(PartialEq, Copy, Clone)]
pub enum IllustrationState {
    Hidden,
    Shown,
    FadeIn(f32),
    FadeOut(f32),
}

pub struct TextScriptVM {
    pub scripts: Rc<RefCell<Scripts>>,
    pub state: TextScriptExecutionState,
    pub stack: Vec<TextScriptExecutionState>,
    pub flags: TextScriptFlags,
    pub mode: ScriptMode,
    /// The player who triggered the event.
    pub executor_player: TargetPlayer,
    /// Toggle for non-strict TSC parsing because English versions of CS+ (both AG and Nicalis release)
    /// modified the events carelessly and since original Pixel's engine hasn't enforced constraints
    /// while parsing no one noticed them.
    pub strict_mode: bool,
    pub suspend: bool,
    /// Requires `constants.textscript.reset_invicibility_on_any_script`
    pub reset_invicibility: bool,
    pub numbers: [u16; 4],
    pub face: u16,
    pub item: u16,
    pub current_line: TextScriptLine,
    pub line_1: Vec<char>,
    pub line_2: Vec<char>,
    pub line_3: Vec<char>,
    pub current_illustration: Option<String>,
    pub illustration_state: IllustrationState,
    prev_char: char,
}

pub struct Scripts {
    /// Head.tsc - shared part of map scripts
    pub global_script: TextScript,
    /// <Map>.tsc - map script
    pub scene_script: TextScript,
    /// ArmsItem.tsc - used by inventory
    pub inventory_script: TextScript,
    /// StageSelect.tsc - used by teleport target selector
    pub stage_select_script: TextScript,
}

impl Scripts {
    pub fn find_script(&self, mode: ScriptMode, event_num: u16) -> Option<&Vec<u8>> {
        match mode {
            ScriptMode::Map => {
                if let Some(tsc) = self.scene_script.event_map.get(&event_num) {
                    return Some(tsc);
                } else if let Some(tsc) = self.global_script.event_map.get(&event_num) {
                    return Some(tsc);
                }
            }
            ScriptMode::Inventory => {
                if let Some(tsc) = self.inventory_script.event_map.get(&event_num) {
                    return Some(tsc);
                }
            }
            ScriptMode::StageSelect => {
                if let Some(tsc) = self.stage_select_script.event_map.get(&event_num) {
                    return Some(tsc);
                }
            }
        }

        None
    }
}

impl TextScriptVM {
    pub fn new() -> Self {
        Self {
            scripts: Rc::new(RefCell::new(Scripts {
                global_script: TextScript::new(),
                scene_script: TextScript::new(),
                inventory_script: TextScript::new(),
                stage_select_script: TextScript::new(),
            })),
            state: TextScriptExecutionState::Ended,
            stack: Vec::with_capacity(6),
            flags: TextScriptFlags(0),
            mode: ScriptMode::Map,
            executor_player: TargetPlayer::Player1,
            strict_mode: false,
            suspend: true,
            reset_invicibility: false,
            numbers: [0; 4],
            face: 0,
            item: 0,
            current_line: TextScriptLine::Line1,
            line_1: Vec::with_capacity(24),
            line_2: Vec::with_capacity(24),
            line_3: Vec::with_capacity(24),
            current_illustration: None,
            illustration_state: IllustrationState::Hidden,
            prev_char: '\x00',
        }
    }

    pub fn set_global_script(&mut self, script: TextScript) {
        {
            let mut scripts = self.scripts.borrow_mut();
            scripts.global_script = script;
        }

        if !self.suspend {
            self.reset();
        }
    }

    pub fn set_scene_script(&mut self, script: TextScript) {
        {
            let mut scripts = self.scripts.borrow_mut();
            scripts.scene_script = script;
        }

        if !self.suspend {
            self.reset();
        }
    }

    pub fn set_inventory_script(&mut self, script: TextScript) {
        let mut scripts = self.scripts.borrow_mut();
        scripts.inventory_script = script;
    }

    pub fn set_stage_select_script(&mut self, script: TextScript) {
        let mut scripts = self.scripts.borrow_mut();
        scripts.stage_select_script = script;
    }

    pub fn reset(&mut self) {
        self.state = TextScriptExecutionState::Ended;
        self.flags.0 = 0;
        self.current_illustration = None;
        self.illustration_state = IllustrationState::Hidden;
        self.face = 0;
        self.clear_text_box();
    }

    pub fn clear_text_box(&mut self) {
        self.item = 0;
        self.current_line = TextScriptLine::Line1;
        self.line_1.clear();
        self.line_2.clear();
        self.line_3.clear();
    }

    pub fn set_mode(&mut self, mode: ScriptMode) {
        self.reset();
        self.mode = mode;
    }

    pub fn start_script(&mut self, event_num: u16) {
        self.reset();
        self.reset_invicibility = true;
        self.state = TextScriptExecutionState::Running(event_num, 0);

        log::info!("Started script: #{:04}", event_num);
    }

    pub fn run(state: &mut SharedGameState, game_scene: &mut GameScene, ctx: &mut Context) -> GameResult {
        let scripts_ref = state.textscript_vm.scripts.clone();
        let scripts = scripts_ref.borrow_mut();
        let mut cached_event: Option<(u16, &Vec<u8>)> = None;

        loop {
            if state.textscript_vm.suspend {
                break;
            }

            match state.textscript_vm.state {
                TextScriptExecutionState::Ended => {
                    state.control_flags.set_interactions_disabled(false);
                    break;
                }
                TextScriptExecutionState::Running(event, ip) => {
                    state.control_flags.set_interactions_disabled(true);

                    // The `!event` case gets optimized out on None match
                    match (cached_event, !event) {
                        (None, bevent) | (Some((bevent, _)), _) if bevent != event => {
                            if let Some(bytecode) = scripts.find_script(state.textscript_vm.mode, event) {
                                cached_event = Some((event, bytecode));
                            } else {
                                cached_event = None;
                            }
                        }
                        _ => (),
                    }

                    state.textscript_vm.state = if let Some((_, bytecode)) = cached_event {
                        TextScriptVM::execute(bytecode, event, ip, state, game_scene, ctx)?
                    } else {
                        TextScriptExecutionState::Ended
                    };

                    if state.textscript_vm.state == TextScriptExecutionState::Ended {
                        state.textscript_vm.reset();
                    }
                }
                TextScriptExecutionState::Msg(event, ip, remaining, counter) => {
                    if counter > 0 {
                        state.textscript_vm.state = TextScriptExecutionState::Msg(event, ip, remaining, counter - 1);
                        break;
                    }

                    if !state.control_flags.control_enabled() {
                        state.touch_controls.control_type = TouchControlType::Dialog;
                    }

                    match (cached_event, !event) {
                        (None, bevent) | (Some((bevent, _)), _) if bevent != event => {
                            if let Some(bytecode) = scripts.find_script(state.textscript_vm.mode, event) {
                                cached_event = Some((event, bytecode));
                            } else {
                                cached_event = None;
                            }
                        }
                        _ => (),
                    }

                    if let Some((_, bytecode)) = cached_event {
                        let mut cursor: Cursor<&[u8]> = Cursor::new(bytecode);
                        let mut new_line = false;
                        cursor.seek(SeekFrom::Start(ip as u64))?;

                        let chr = std::char::from_u32(read_cur_varint(&mut cursor)? as u32).unwrap_or('\u{fffd}');

                        match chr {
                            '\n' if state.textscript_vm.current_line == TextScriptLine::Line1 => {
                                state.textscript_vm.current_line = TextScriptLine::Line2;
                            }
                            '\n' if state.textscript_vm.current_line == TextScriptLine::Line2 => {
                                state.textscript_vm.current_line = TextScriptLine::Line3;
                            }
                            '\n' => {
                                new_line = true;
                            }
                            '\r' => {}
                            _ if state.textscript_vm.current_line == TextScriptLine::Line1 => {
                                state.textscript_vm.prev_char = chr;
                                state.textscript_vm.line_1.push(chr);

                                let text_len =
                                    state.font.text_width(state.textscript_vm.line_1.iter().copied(), &state.constants);
                                if text_len >= 284.0 {
                                    state.textscript_vm.current_line = TextScriptLine::Line2;
                                }
                            }
                            _ if state.textscript_vm.current_line == TextScriptLine::Line2 => {
                                state.textscript_vm.prev_char = chr;
                                state.textscript_vm.line_2.push(chr);

                                let text_len =
                                    state.font.text_width(state.textscript_vm.line_2.iter().copied(), &state.constants);
                                if text_len >= 284.0 {
                                    state.textscript_vm.current_line = TextScriptLine::Line3;
                                }
                            }
                            _ if state.textscript_vm.current_line == TextScriptLine::Line3 => {
                                state.textscript_vm.prev_char = chr;
                                state.textscript_vm.line_3.push(chr);

                                let text_len =
                                    state.font.text_width(state.textscript_vm.line_3.iter().copied(), &state.constants);
                                if text_len >= 284.0 {
                                    new_line = true;
                                }
                            }
                            _ => {}
                        }

                        if remaining > 1 {
                            let ticks = if state.textscript_vm.flags.fast() || state.textscript_vm.flags.cutscene_skip()
                            {
                                0
                            } else if remaining != 2
                                && (game_scene.player1.controller.jump()
                                    || game_scene.player1.controller.shoot()
                                    || game_scene.player2.controller.jump()
                                    || game_scene.player2.controller.shoot())
                            {
                                state.constants.textscript.text_speed_fast
                            } else {
                                state.constants.textscript.text_speed_normal
                            };

                            if ticks > 0 {
                                state.sound_manager.play_sfx(2);
                            }

                            state.textscript_vm.state = if !new_line {
                                TextScriptExecutionState::Msg(event, cursor.position() as u32, remaining - 1, ticks)
                            } else {
                                TextScriptExecutionState::MsgNewLine(
                                    event,
                                    cursor.position() as u32,
                                    remaining - 1,
                                    ticks,
                                    4,
                                )
                            };
                        } else {
                            state.textscript_vm.state =
                                TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    } else {
                        state.textscript_vm.reset();
                    }
                }
                TextScriptExecutionState::MsgNewLine(event, ip, remaining, ticks, mut counter) => {
                    counter = if state.textscript_vm.flags.fast() || state.textscript_vm.flags.cutscene_skip() {
                        0
                    } else {
                        counter.saturating_sub(1)
                    };

                    if counter == 0 {
                        state.textscript_vm.line_1.clear();
                        state.textscript_vm.line_1.append(&mut state.textscript_vm.line_2);
                        state.textscript_vm.line_2.append(&mut state.textscript_vm.line_3);
                        state.textscript_vm.state = TextScriptExecutionState::Msg(event, ip, remaining, ticks);
                    } else {
                        state.textscript_vm.state =
                            TextScriptExecutionState::MsgNewLine(event, ip, remaining, ticks, counter);
                    }
                    break;
                }
                TextScriptExecutionState::WaitTicks(event, ip, ticks) => {
                    if ticks == 0 {
                        state.textscript_vm.state = TextScriptExecutionState::Running(event, ip);
                    } else if ticks != 9999 {
                        state.textscript_vm.state = TextScriptExecutionState::WaitTicks(event, ip, ticks - 1);
                        break;
                    } else {
                        break;
                    }
                }
                TextScriptExecutionState::WaitConfirmation(event, ip, no_event, wait, selection) => {
                    state.textscript_vm.flags.set_cutscene_skip(false);

                    if wait > 0 {
                        state.textscript_vm.state =
                            TextScriptExecutionState::WaitConfirmation(event, ip, no_event, wait - 1, selection);
                        break;
                    }

                    let mut confirm =
                        game_scene.player1.controller.trigger_jump() || game_scene.player2.controller.trigger_jump();

                    if state.settings.touch_controls && !state.control_flags.control_enabled() {
                        state.touch_controls.control_type = TouchControlType::None;

                        let (off_left, _, off_right, off_bottom) =
                            crate::framework::graphics::screen_insets_scaled(ctx, state.scale);
                        let box_x = ((state.canvas_size.0 - off_left - off_right) / 2.0) as isize + 51;
                        let box_y = (state.canvas_size.1 - off_bottom - 96.0 - 10.0) as isize;

                        if state.touch_controls.consume_click_in(Rect::new_size(box_x, box_y, 40, 40)) {
                            match selection {
                                ConfirmSelection::Yes => confirm = true,
                                ConfirmSelection::No => {
                                    state.sound_manager.play_sfx(1);
                                    state.textscript_vm.state = TextScriptExecutionState::WaitConfirmation(
                                        event,
                                        ip,
                                        no_event,
                                        0,
                                        ConfirmSelection::Yes,
                                    );
                                }
                            }
                        } else if state.touch_controls.consume_click_in(Rect::new_size(box_x + 41, box_y, 40, 40)) {
                            match selection {
                                ConfirmSelection::Yes => {
                                    state.sound_manager.play_sfx(1);
                                    state.textscript_vm.state = TextScriptExecutionState::WaitConfirmation(
                                        event,
                                        ip,
                                        no_event,
                                        0,
                                        ConfirmSelection::No,
                                    );
                                }
                                ConfirmSelection::No => confirm = true,
                            }
                        }
                    }

                    if game_scene.player1.controller.trigger_left()
                        || game_scene.player1.controller.trigger_right()
                        || game_scene.player2.controller.trigger_left()
                        || game_scene.player2.controller.trigger_right()
                    {
                        state.sound_manager.play_sfx(1);
                        state.textscript_vm.state =
                            TextScriptExecutionState::WaitConfirmation(event, ip, no_event, 0, !selection);
                        break;
                    }

                    if confirm {
                        state.sound_manager.play_sfx(18);
                        match selection {
                            ConfirmSelection::Yes => {
                                state.textscript_vm.state = TextScriptExecutionState::Running(event, ip);
                            }
                            ConfirmSelection::No => {
                                state.textscript_vm.clear_text_box();
                                state.textscript_vm.state = TextScriptExecutionState::Running(no_event, 0);
                            }
                        }
                    }

                    break;
                }
                TextScriptExecutionState::WaitStanding(event, ip) => {
                    if game_scene.player1.flags.hit_bottom_wall() {
                        state.textscript_vm.state = TextScriptExecutionState::Running(event, ip);
                    }
                    break;
                }
                TextScriptExecutionState::WaitInput(event, ip, blink) => {
                    state.textscript_vm.state = TextScriptExecutionState::WaitInput(event, ip, (blink + 1) % 20);

                    if !state.control_flags.control_enabled() {
                        state.touch_controls.control_type = TouchControlType::Dialog;
                    }

                    if state.textscript_vm.flags.cutscene_skip()
                        || game_scene.player1.controller.trigger_jump()
                        || game_scene.player1.controller.trigger_shoot()
                        || game_scene.player2.controller.trigger_jump()
                        || game_scene.player2.controller.trigger_shoot()
                    {
                        state.textscript_vm.state = TextScriptExecutionState::Running(event, ip);
                    }
                    break;
                }
                TextScriptExecutionState::WaitFade(event, ip) => {
                    if state.fade_state == FadeState::Hidden || state.fade_state == FadeState::Visible {
                        state.textscript_vm.state = TextScriptExecutionState::Running(event, ip);
                    }
                    break;
                }
                TextScriptExecutionState::FallingIsland(event, ip, pos_x, mut pos_y, mut tick, mode) => {
                    if tick == 900 {
                        state.textscript_vm.state = TextScriptExecutionState::Running(event, ip);
                        break;
                    }

                    tick += 1;

                    if mode {
                        if tick < 350 {
                            pos_y += 0x33;
                        } else if tick < 500 {
                            pos_y += 0x19;
                        } else if tick < 600 {
                            pos_y += 0xC;
                        } else if tick == 750 {
                            tick = 900;
                        }
                    } else {
                        pos_y += 0x33;
                    }

                    state.textscript_vm.state =
                        TextScriptExecutionState::FallingIsland(event, ip, pos_x, pos_y, tick, mode);
                    break;
                }
                TextScriptExecutionState::SaveProfile(event, ip) => {
                    state.save_game(game_scene, ctx)?;
                    state.textscript_vm.state = TextScriptExecutionState::Running(event, ip);
                    break;
                }
                TextScriptExecutionState::LoadProfile => {
                    state.load_or_start_game(ctx)?;
                    break;
                }
                TextScriptExecutionState::Reset => {
                    state.reset();
                    state.start_new_game(ctx)?;
                    break;
                }
                TextScriptExecutionState::MapSystem => {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn execute(
        bytecode: &[u8],
        event: u16,
        ip: u32,
        state: &mut SharedGameState,
        game_scene: &mut GameScene,
        ctx: &mut Context,
    ) -> GameResult<TextScriptExecutionState> {
        let mut exec_state = state.textscript_vm.state;
        let mut cursor = Cursor::new(bytecode);
        cursor.seek(SeekFrom::Start(ip as u64))?;

        let op: TSCOpCode = if let Some(op) =
            FromPrimitive::from_i32(read_cur_varint(&mut cursor).unwrap_or_else(|_| TSCOpCode::END as i32))
        {
            op
        } else {
            return Ok(TextScriptExecutionState::Ended);
        };

        match op {
            TSCOpCode::_NOP => {
                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::_UNI => {}
            TSCOpCode::_STR => {
                let mut len = read_cur_varint(&mut cursor)? as u32;
                if state.textscript_vm.flags.render() {
                    state.textscript_vm.prev_char = '\x00';
                    exec_state = TextScriptExecutionState::Msg(event, cursor.position() as u32, len, 4);
                } else {
                    while len > 0 {
                        len -= 1;
                        let _ = read_cur_varint(&mut cursor)?;
                    }
                    // simply skip the text if we aren't in message mode.
                    exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                }
            }
            TSCOpCode::_END => {
                state.textscript_vm.flags.set_cutscene_skip(false);
                exec_state = TextScriptExecutionState::Ended;
            }
            TSCOpCode::END => {
                state.textscript_vm.flags.set_cutscene_skip(false);
                state.control_flags.set_tick_world(true);
                state.control_flags.set_control_enabled(true);

                state.textscript_vm.flags.set_render(false);
                state.textscript_vm.flags.set_background_visible(false);
                state.textscript_vm.stack.clear();

                game_scene.player1.cond.set_interacted(false);
                game_scene.player2.cond.set_interacted(false);

                exec_state = TextScriptExecutionState::Ended;
            }
            TSCOpCode::SLP => {
                state.textscript_vm.set_mode(ScriptMode::StageSelect);

                let event_num = if let Some(slot) =
                    state.teleporter_slots.get(game_scene.stage_select.current_teleport_slot as usize)
                {
                    1000 + slot.0
                } else {
                    1000
                };

                exec_state = TextScriptExecutionState::Running(event_num, 0);
            }
            TSCOpCode::PSp => {
                let index = read_cur_varint(&mut cursor)? as u16;
                let event_num = read_cur_varint(&mut cursor)? as u16;

                if let Some(slot) = state.teleporter_slots.iter_mut().find(|s| s.0 == index) {
                    slot.1 = event_num;
                } else {
                    state.teleporter_slots.push((index, event_num));
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::PRI => {
                state.control_flags.set_tick_world(false);
                state.control_flags.set_control_enabled(false);

                game_scene.player1.shock_counter = 0;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::KEY => {
                state.control_flags.set_tick_world(true);
                state.control_flags.set_control_enabled(false);

                game_scene.player1.up = false;
                game_scene.player1.shock_counter = 0;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::FRE => {
                state.control_flags.set_tick_world(true);
                state.control_flags.set_control_enabled(true);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::MYD => {
                let new_direction = read_cur_varint(&mut cursor)? as usize;
                if let Some(direction) = Direction::from_int(new_direction) {
                    game_scene.player1.direction = direction;
                    game_scene.player2.direction = direction;
                }
                game_scene.player1.cond.set_interacted(new_direction == 3);
                game_scene.player2.cond.set_interacted(new_direction == 3);

                game_scene.player1.vel_x = 0;
                game_scene.player2.vel_x = 0;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::MYB => {
                let new_direction = read_cur_varint(&mut cursor)? as usize;

                game_scene.player1.vel_y = -0x200;
                game_scene.player2.vel_y = -0x200;

                // Reset interaction condition, needed for places like talking to Toroko in shack
                game_scene.player1.cond.set_interacted(false);
                game_scene.player2.cond.set_interacted(false);

                if let Some(direction) = Direction::from_int_facing(new_direction) {
                    match direction {
                        Direction::Left => {
                            game_scene.player1.direction = Left;
                            game_scene.player2.direction = Left;
                            game_scene.player1.vel_x = 0x200;
                            game_scene.player2.vel_x = 0x200;
                        }
                        Direction::Up => {
                            game_scene.player1.vel_y = -0x200;
                            game_scene.player2.vel_y = -0x200;
                        }
                        Direction::Right => {
                            game_scene.player1.direction = Right;
                            game_scene.player2.direction = Right;
                            game_scene.player1.vel_x = -0x200;
                            game_scene.player2.vel_x = -0x200;
                        }
                        Direction::Bottom => {
                            game_scene.player1.vel_y = 0x200;
                            game_scene.player2.vel_y = 0x200;
                        }
                        Direction::FacingPlayer => {
                            for npc in game_scene.npc_list.iter_alive() {
                                if npc.event_num == new_direction as u16 {
                                    if game_scene.player1.x >= npc.x {
                                        game_scene.player1.direction = Left;
                                        game_scene.player1.vel_x = 0x200;
                                    } else {
                                        game_scene.player1.direction = Right;
                                        game_scene.player1.vel_x = -0x200;
                                    }

                                    if game_scene.player2.x >= npc.x {
                                        game_scene.player2.direction = Left;
                                        game_scene.player2.vel_x = 0x200;
                                    } else {
                                        game_scene.player2.direction = Right;
                                        game_scene.player2.vel_x = -0x200;
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::SMC => {
                game_scene.player1.cond.set_hidden(false);
                game_scene.player2.cond.set_hidden(false);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::HMC => {
                game_scene.player1.cond.set_hidden(true);
                game_scene.player2.cond.set_hidden(true);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::HM2 => {
                let player = match state.textscript_vm.executor_player {
                    TargetPlayer::Player1 => &mut game_scene.player1,
                    TargetPlayer::Player2 => &mut game_scene.player2,
                };

                player.cond.set_hidden(true);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::WAI => {
                let ticks = read_cur_varint(&mut cursor)? as u16;

                exec_state = TextScriptExecutionState::WaitTicks(event, cursor.position() as u32, ticks);
            }
            TSCOpCode::WAS => {
                exec_state = TextScriptExecutionState::WaitStanding(event, cursor.position() as u32);
            }
            TSCOpCode::NOD => {
                exec_state = TextScriptExecutionState::WaitInput(event, cursor.position() as u32, 0);
            }
            TSCOpCode::FLp | TSCOpCode::FLm => {
                let flag_num = read_cur_varint(&mut cursor)? as u16;
                state.set_flag(flag_num as usize, op == TSCOpCode::FLp);
                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::SKp | TSCOpCode::SKm => {
                let flag_num = read_cur_varint(&mut cursor)? as u16;
                state.set_skip_flag(flag_num as usize, op == TSCOpCode::SKp);
                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::FFm => {
                let flag_from = read_cur_varint(&mut cursor)? as usize;
                let flag_to = read_cur_varint(&mut cursor)? as usize;

                if flag_to >= flag_from {
                    for flag in flag_from..=flag_to {
                        if state.get_flag(flag) {
                            state.set_flag(flag, false);
                            break;
                        }
                    }
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::FLJ => {
                let flag_num = read_cur_varint(&mut cursor)? as usize;
                let event_num = read_cur_varint(&mut cursor)? as u16;
                if state.get_flag(flag_num) {
                    state.textscript_vm.clear_text_box();
                    exec_state = TextScriptExecutionState::Running(event_num, 0);
                } else {
                    exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                }
            }
            TSCOpCode::MPJ => {
                let event_num = read_cur_varint(&mut cursor)? as u16;

                if state.get_map_flag(game_scene.stage_id) {
                    state.textscript_vm.clear_text_box();
                    exec_state = TextScriptExecutionState::Running(event_num, 0);
                } else {
                    exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                }
            }
            TSCOpCode::ITJ => {
                let item_id = read_cur_varint(&mut cursor)? as u16;
                let event_num = read_cur_varint(&mut cursor)? as u16;

                if game_scene.inventory_player1.has_item(item_id) {
                    state.textscript_vm.clear_text_box();
                    exec_state = TextScriptExecutionState::Running(event_num, 0);
                } else {
                    exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                }
            }
            TSCOpCode::INJ => {
                let item_id = read_cur_varint(&mut cursor)? as u16;
                let amount = read_cur_varint(&mut cursor)? as u16;
                let event_num = read_cur_varint(&mut cursor)? as u16;

                if game_scene.inventory_player1.has_item_amount(item_id, Ordering::Equal, amount) {
                    state.textscript_vm.clear_text_box();
                    exec_state = TextScriptExecutionState::Running(event_num, 0);
                } else {
                    exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                }
            }
            TSCOpCode::AMJ => {
                let weapon = read_cur_varint(&mut cursor)? as u8;
                let event_num = read_cur_varint(&mut cursor)? as u16;
                let weapon_type: Option<WeaponType> = FromPrimitive::from_u8(weapon);

                if weapon_type.is_some() && game_scene.inventory_player1.has_weapon(weapon_type.unwrap()) {
                    state.textscript_vm.clear_text_box();
                    exec_state = TextScriptExecutionState::Running(event_num, 0);
                } else {
                    exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                }
            }
            TSCOpCode::NCJ => {
                let npc_type = read_cur_varint(&mut cursor)? as u16;
                let event_num = read_cur_varint(&mut cursor)? as u16;

                if game_scene.npc_list.is_alive_by_type(npc_type) {
                    state.textscript_vm.clear_text_box();
                    exec_state = TextScriptExecutionState::Running(event_num, 0);
                } else {
                    exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                }
            }
            TSCOpCode::ECJ => {
                let npc_event_num = read_cur_varint(&mut cursor)? as u16;
                let event_num = read_cur_varint(&mut cursor)? as u16;

                if game_scene.npc_list.is_alive_by_event(npc_event_num) {
                    state.textscript_vm.clear_text_box();
                    exec_state = TextScriptExecutionState::Running(event_num, 0);
                } else {
                    exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                }
            }
            TSCOpCode::SKJ => {
                let flag_id = read_cur_varint(&mut cursor)? as u16;
                let event_num = read_cur_varint(&mut cursor)? as u16;

                if state.get_skip_flag(flag_id as usize) {
                    state.textscript_vm.clear_text_box();
                    exec_state = TextScriptExecutionState::Running(event_num, 0);
                } else {
                    exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                }
            }

            TSCOpCode::S2PJ => {
                let event_num = read_cur_varint(&mut cursor)? as u16;

                exec_state = if game_scene.player2.cond.alive() {
                    TextScriptExecutionState::Running(event_num, 0)
                } else {
                    TextScriptExecutionState::Running(event, cursor.position() as u32)
                }
            }
            TSCOpCode::EVE => {
                let event_num = read_cur_varint(&mut cursor)? as u16;

                state.textscript_vm.clear_text_box();
                exec_state = TextScriptExecutionState::Running(event_num, 0);
            }
            TSCOpCode::PSH => {
                let event_num = read_cur_varint(&mut cursor)? as u16;

                let saved_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                state.textscript_vm.stack.push(saved_state);

                state.textscript_vm.clear_text_box();
                exec_state = TextScriptExecutionState::Running(event_num, 0);
            }
            TSCOpCode::POP => {
                if let Some(saved_state) = state.textscript_vm.stack.pop() {
                    exec_state = saved_state;
                } else {
                    log::warn!("Tried to <POP from TSC stack without saved state!");
                    exec_state = TextScriptExecutionState::Ended;
                }
            }
            TSCOpCode::MM0 => {
                game_scene.player1.vel_x = 0;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::SMP => {
                let pos_x = read_cur_varint(&mut cursor)? as usize;
                let pos_y = read_cur_varint(&mut cursor)? as usize;

                let tile_type = game_scene.stage.tile_at(pos_x, pos_y);
                game_scene.stage.change_tile(pos_x, pos_y, tile_type.wrapping_sub(1));

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::CMP => {
                let pos_x = read_cur_varint(&mut cursor)? as usize;
                let pos_y = read_cur_varint(&mut cursor)? as usize;
                let tile_type = read_cur_varint(&mut cursor)? as u8;

                if game_scene.stage.change_tile(pos_x, pos_y, tile_type) {
                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = pos_x as i32 * 0x2000;
                    npc.y = pos_y as i32 * 0x2000;

                    let _ = game_scene.npc_list.spawn(0x100, npc.clone());
                    let _ = game_scene.npc_list.spawn(0x100, npc);
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::MLp => {
                let life = read_cur_varint(&mut cursor)? as u16;
                game_scene.player1.life += life;
                game_scene.player1.max_life += life;
                game_scene.player2.life += life;
                game_scene.player2.max_life += life;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::FAC => {
                let face = read_cur_varint(&mut cursor)? as u16;
                // Switch uses xx00 for face animation states
                if face % 100 != state.textscript_vm.face % 100 {
                    game_scene.text_boxes.slide_in = 7;
                }
                state.textscript_vm.face = face;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::CLR => {
                state.textscript_vm.current_line = TextScriptLine::Line1;
                state.textscript_vm.line_1.clear();
                state.textscript_vm.line_2.clear();
                state.textscript_vm.line_3.clear();

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::MSG | TSCOpCode::MS2 | TSCOpCode::MS3 => {
                state.textscript_vm.current_line = TextScriptLine::Line1;
                state.textscript_vm.line_1.clear();
                state.textscript_vm.line_2.clear();
                state.textscript_vm.line_3.clear();
                state.textscript_vm.flags.set_render(true);
                state.textscript_vm.flags.set_background_visible(op != TSCOpCode::MS2);
                state.textscript_vm.flags.set_fast(state.textscript_vm.flags.perma_fast());
                state.textscript_vm.flags.set_position_top(op != TSCOpCode::MSG);
                if op == TSCOpCode::MS2 {
                    state.textscript_vm.face = 0;
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::SAT | TSCOpCode::CAT => {
                state.textscript_vm.flags.set_perma_fast(true);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }

            TSCOpCode::TUR => {
                state.textscript_vm.flags.set_fast(true);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::CLO => {
                state.textscript_vm.flags.set_render(false);
                state.textscript_vm.flags.set_background_visible(false);
                state.textscript_vm.flags.set_fast(false);
                state.textscript_vm.flags.set_position_top(false);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::YNJ => {
                let event_no = read_cur_varint(&mut cursor)? as u16;

                state.sound_manager.play_sfx(5);

                exec_state = TextScriptExecutionState::WaitConfirmation(
                    event,
                    cursor.position() as u32,
                    event_no,
                    16,
                    ConfirmSelection::Yes,
                );
            }
            TSCOpCode::UNJ => {
                let mode = read_cur_varint(&mut cursor)?;
                let event_num = read_cur_varint(&mut cursor)? as u16;

                exec_state = if game_scene.player1.control_mode as i32 == mode {
                    TextScriptExecutionState::Running(event_num, cursor.position() as u32)
                } else {
                    TextScriptExecutionState::Running(event, cursor.position() as u32)
                };
            }
            TSCOpCode::NUM => {
                let index = read_cur_varint(&mut cursor)? as usize;

                if let Some(num) = state.textscript_vm.numbers.get(index) {
                    let mut str = num.to_string().chars().collect();

                    match state.textscript_vm.current_line {
                        TextScriptLine::Line1 => state.textscript_vm.line_1.append(&mut str),
                        TextScriptLine::Line2 => state.textscript_vm.line_2.append(&mut str),
                        TextScriptLine::Line3 => state.textscript_vm.line_3.append(&mut str),
                    }
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::GIT => {
                let item = read_cur_varint(&mut cursor)? as u16;
                state.textscript_vm.item = item;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::TRA => {
                let map_id = read_cur_varint(&mut cursor)? as usize;
                let event_num = read_cur_varint(&mut cursor)? as u16;

                let mut new_scene = GameScene::new(state, ctx, map_id)?;

                let block_size = new_scene.stage.map.tile_size.as_int() * 0x200;
                let pos_x = read_cur_varint(&mut cursor)? as i32 * block_size;
                let pos_y = read_cur_varint(&mut cursor)? as i32 * block_size;

                new_scene.intro_mode = game_scene.intro_mode;
                new_scene.inventory_player1 = game_scene.inventory_player1.clone();
                new_scene.inventory_player2 = game_scene.inventory_player2.clone();
                new_scene.player1 = game_scene.player1.clone();
                new_scene.player1.vel_x = 0;
                new_scene.player1.vel_y = 0;
                new_scene.player1.x = pos_x;
                new_scene.player1.y = pos_y;
                new_scene.player2 = game_scene.player2.clone();
                new_scene.player2.vel_x = 0;
                new_scene.player2.vel_y = 0;
                new_scene.player2.x = pos_x;
                new_scene.player2.y = pos_y;
                // Reset player interaction flag upon TRA
                new_scene.player1.cond.set_interacted(false);
                new_scene.player2.cond.set_interacted(false);
                // Reset ground collision for WAS / WaitStanding
                new_scene.player1.flags.set_hit_bottom_wall(false);
                new_scene.player2.flags.set_hit_bottom_wall(false);
                new_scene.frame.wait = game_scene.frame.wait;
                new_scene.nikumaru = game_scene.nikumaru;
                new_scene.replay = game_scene.replay.clone();

                let skip = state.textscript_vm.flags.cutscene_skip();
                state.control_flags.set_tick_world(true);
                state.control_flags.set_interactions_disabled(true);
                state.textscript_vm.flags.0 = 0;
                state.textscript_vm.flags.set_cutscene_skip(skip);
                state.textscript_vm.face = 0;
                state.textscript_vm.item = 0;
                state.textscript_vm.current_line = TextScriptLine::Line1;
                state.textscript_vm.line_1.clear();
                state.textscript_vm.line_2.clear();
                state.textscript_vm.line_3.clear();
                state.textscript_vm.suspend = true;
                state.next_scene = Some(Box::new(new_scene));

                log::info!("Transitioning to stage {}, with script #{:04}", map_id, event_num);
                exec_state = TextScriptExecutionState::Running(event_num, 0);
            }
            TSCOpCode::MOV => {
                let block_size = state.tile_size.as_int() * 0x200;

                let pos_x = read_cur_varint(&mut cursor)? as i32 * block_size;
                let pos_y = read_cur_varint(&mut cursor)? as i32 * block_size;

                game_scene.player1.cond.set_interacted(false);
                game_scene.player2.cond.set_interacted(false);

                for player in [&mut game_scene.player1, &mut game_scene.player2].iter_mut() {
                    player.vel_x = 0;
                    player.vel_y = 0;
                    player.x = pos_x;
                    player.y = pos_y;
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::S2MV => {
                let param = read_cur_varint(&mut cursor)? as usize;

                let (executor, partner) = match state.textscript_vm.executor_player {
                    TargetPlayer::Player1 => (&game_scene.player1, &mut game_scene.player2),
                    TargetPlayer::Player2 => (&game_scene.player2, &mut game_scene.player1),
                };

                match param {
                    0 | 1 => {
                        partner.vel_x = 0;
                        partner.vel_y = 0;
                        partner.x = executor.x + if param == 0 { -0x2000 } else { 0x2000 };
                        partner.y = executor.y;
                    }
                    2..=10 => {
                        log::warn!("<2MV unknown param");
                    }
                    // what the fuck
                    i => {
                        let distance = i as i32 / 10;

                        partner.vel_x = 0;
                        partner.vel_y = 0;
                        partner.x = executor.x + if (param % 10) == 1 { distance * 0x200 } else { -distance * 0x200 };
                        partner.y = executor.y;
                    }
                }

                if partner.cond.alive() && !partner.cond.hidden() {
                    let mut npc = NPC::create(4, &state.npc_table);
                    npc.cond.set_alive(true);
                    npc.x = partner.x;
                    npc.y = partner.y;

                    let _ = game_scene.npc_list.spawn(0x100, npc.clone());
                    let _ = game_scene.npc_list.spawn(0x100, npc.clone());
                    let _ = game_scene.npc_list.spawn(0x100, npc.clone());
                    let _ = game_scene.npc_list.spawn(0x100, npc);
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::UNI => {
                let control_mode = read_cur_varint(&mut cursor)? as u8;

                let mode: Option<ControlMode> = FromPrimitive::from_u8(control_mode);
                if let Some(mode) = mode {
                    game_scene.player1.control_mode = mode;
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::FAI => {
                let fade_type = read_cur_varint(&mut cursor)? as usize;

                if let Some(direction) = FadeDirection::from_int(fade_type) {
                    state.fade_state = FadeState::FadeIn(15, direction);
                }

                exec_state = TextScriptExecutionState::WaitFade(event, cursor.position() as u32);
            }
            TSCOpCode::FAO => {
                let fade_type = read_cur_varint(&mut cursor)? as usize;

                if let Some(direction) = FadeDirection::from_int(fade_type) {
                    state.fade_state = FadeState::FadeOut(-15, direction.opposite());
                }

                exec_state = TextScriptExecutionState::WaitFade(event, cursor.position() as u32);
            }
            TSCOpCode::QUA => {
                let count = read_cur_varint(&mut cursor)? as u16;

                state.quake_counter = count;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::MNA => {
                game_scene.display_map_name(160);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::CMU => {
                let song_id = read_cur_varint(&mut cursor)? as usize;
                state.sound_manager.play_song(song_id, &state.constants, &state.settings, ctx)?;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::FMU => {
                state.sound_manager.play_song(0, &state.constants, &state.settings, ctx)?;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::RMU => {
                state.sound_manager.restore_state()?;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::SOU => {
                let sound = read_cur_varint(&mut cursor)? as u8;

                state.sound_manager.play_sfx(sound);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::DNP => {
                let event_num = read_cur_varint(&mut cursor)? as u16;

                game_scene.npc_list.remove_by_event(event_num, state);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::DNA => {
                let npc_remove_type = read_cur_varint(&mut cursor)? as u16;

                game_scene.npc_list.remove_by_type(npc_remove_type, state);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::FOB => {
                let part_id = read_cur_varint(&mut cursor)? as u16;
                let ticks = read_cur_varint(&mut cursor)? as i32;

                game_scene.frame.wait = ticks;
                game_scene.frame.update_target = UpdateTarget::Boss(part_id);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::FOM => {
                let ticks = read_cur_varint(&mut cursor)? as i32;
                game_scene.frame.wait = ticks;
                game_scene.frame.update_target = UpdateTarget::Player;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::FON => {
                let event_num = read_cur_varint(&mut cursor)? as u16;
                let ticks = read_cur_varint(&mut cursor)? as i32;
                game_scene.frame.wait = ticks;

                for npc in game_scene.npc_list.iter() {
                    if event_num == npc.event_num {
                        game_scene.frame.update_target = UpdateTarget::NPC(npc.id);
                        break;
                    }
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::BSL => {
                let event_num = read_cur_varint(&mut cursor)? as u16;

                if event_num == 0 {
                    game_scene.boss_life_bar.set_boss_target(&game_scene.boss);
                } else {
                    for npc in game_scene.npc_list.iter_alive() {
                        if event_num == npc.event_num {
                            game_scene.boss_life_bar.set_npc_target(npc.id, &game_scene.npc_list);
                            break;
                        }
                    }
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::BOA => {
                let action_num = read_cur_varint(&mut cursor)? as u16;

                game_scene.boss.parts[0].action_num = action_num;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::ANP => {
                let event_num = read_cur_varint(&mut cursor)? as u16;
                let action_num = read_cur_varint(&mut cursor)? as u16;
                let tsc_direction = read_cur_varint(&mut cursor)? as usize;
                let direction = Direction::from_int_facing(tsc_direction).unwrap_or(Direction::Left);

                for npc in game_scene.npc_list.iter_alive() {
                    if npc.event_num == event_num {
                        npc.action_num = action_num;
                        npc.tsc_direction = tsc_direction as u16;

                        if direction == Direction::FacingPlayer {
                            npc.direction =
                                if game_scene.player1.x < npc.x { Direction::Left } else { Direction::Right };
                        } else if tsc_direction != 5 {
                            npc.direction = direction;
                        }
                    }
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::CNP | TSCOpCode::INP => {
                let event_num = read_cur_varint(&mut cursor)? as u16;
                let new_type = read_cur_varint(&mut cursor)? as u16;
                let tsc_direction = read_cur_varint(&mut cursor)? as usize;
                let direction = Direction::from_int_facing(tsc_direction).unwrap_or(Direction::Left);

                for npc in game_scene.npc_list.iter_alive() {
                    if npc.event_num == event_num {
                        npc.npc_flags.set_solid_soft(false);
                        npc.npc_flags.set_ignore_tile_44(false);
                        npc.npc_flags.set_invulnerable(false);
                        npc.npc_flags.set_ignore_solidity(false);
                        npc.npc_flags.set_bouncy(false);
                        npc.npc_flags.set_shootable(false);
                        npc.npc_flags.set_solid_hard(false);
                        npc.npc_flags.set_rear_and_top_not_hurt(false);
                        npc.npc_flags.set_show_damage(false);

                        if op == TSCOpCode::INP {
                            npc.npc_flags.set_event_when_touched(true);
                        }

                        npc.npc_type = new_type;
                        npc.display_bounds = state.npc_table.get_display_bounds(new_type);
                        npc.hit_bounds = state.npc_table.get_hit_bounds(new_type);
                        let entry = state.npc_table.get_entry(new_type).unwrap().to_owned();
                        npc.npc_flags.0 |= entry.npc_flags.0;
                        npc.life = entry.life;
                        npc.size = entry.size;
                        npc.exp = entry.experience as u16;
                        npc.damage = entry.damage as u16;
                        npc.spritesheet_id = entry.spritesheet_id as u16;

                        npc.cond.set_alive(true);
                        npc.action_num = 0;
                        npc.action_counter = 0;
                        npc.anim_num = 0;
                        npc.anim_counter = 0;
                        npc.vel_x = 0;
                        npc.vel_y = 0;
                        npc.tsc_direction = tsc_direction as u16;

                        if direction == Direction::FacingPlayer {
                            npc.direction =
                                if game_scene.player1.x < npc.x { Direction::Left } else { Direction::Right };
                        } else if tsc_direction != 5 {
                            npc.direction = direction;
                        }

                        npc.tick(
                            state,
                            (
                                [&mut game_scene.player1, &mut game_scene.player2],
                                &game_scene.npc_list,
                                &mut game_scene.stage,
                                &mut game_scene.bullet_manager,
                                &mut game_scene.flash,
                                &mut game_scene.boss,
                            ),
                        )?;
                    }
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::MNP => {
                let event_num = read_cur_varint(&mut cursor)? as u16;
                let x = read_cur_varint(&mut cursor)? as i32;
                let y = read_cur_varint(&mut cursor)? as i32;
                let tsc_direction = read_cur_varint(&mut cursor)? as usize;
                let direction = Direction::from_int_facing(tsc_direction).unwrap_or(Direction::Left);
                let block_size = state.tile_size.as_int() * 0x200;

                for npc in game_scene.npc_list.iter_alive() {
                    if npc.event_num == event_num {
                        npc.x = x * block_size;
                        npc.y = y * block_size;
                        npc.tsc_direction = tsc_direction as u16;

                        if direction == Direction::FacingPlayer {
                            npc.direction =
                                if game_scene.player1.x < npc.x { Direction::Left } else { Direction::Right };
                        } else if tsc_direction != 5 {
                            npc.direction = direction;
                        }

                        break;
                    }
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::SNP => {
                let npc_type = read_cur_varint(&mut cursor)? as u16;
                let x = read_cur_varint(&mut cursor)? as i32;
                let y = read_cur_varint(&mut cursor)? as i32;
                let tsc_direction = read_cur_varint(&mut cursor)? as usize;
                let direction = Direction::from_int_facing(tsc_direction).unwrap_or(Direction::Left);
                let block_size = state.tile_size.as_int() * 0x200;

                let mut npc = NPC::create(npc_type, &state.npc_table);
                npc.cond.set_alive(true);
                npc.x = x * block_size;
                npc.y = y * block_size;
                npc.tsc_direction = tsc_direction as u16;

                if direction == Direction::FacingPlayer {
                    npc.direction = if game_scene.player1.x < npc.x { Direction::Left } else { Direction::Right };
                } else {
                    npc.direction = direction;
                }

                let _ = game_scene.npc_list.spawn(0x100, npc);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::LIp => {
                let life = read_cur_varint(&mut cursor)? as u16;

                game_scene.player1.life = clamp(game_scene.player1.life + life, 0, game_scene.player1.max_life);
                game_scene.player2.life = clamp(game_scene.player2.life + life, 0, game_scene.player2.max_life);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::ITp => {
                let item_id = read_cur_varint(&mut cursor)? as u16;

                state.sound_manager.play_sfx(38);

                if !game_scene.inventory_player1.has_item(item_id) {
                    game_scene.inventory_player1.add_item(item_id);
                    state.mod_requirements.append_item(ctx, item_id)?;
                }

                if !game_scene.inventory_player2.has_item(item_id) {
                    game_scene.inventory_player2.add_item(item_id);
                    state.mod_requirements.append_item(ctx, item_id)?;
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::IpN => {
                let item_id = read_cur_varint(&mut cursor)? as u16;
                let amount = read_cur_varint(&mut cursor)? as u16;

                if game_scene.inventory_player1.has_item_amount(item_id, Ordering::Less, amount) {
                    game_scene.inventory_player1.add_item(item_id);
                    state.mod_requirements.append_item(ctx, item_id)?;
                }

                if game_scene.inventory_player2.has_item_amount(item_id, Ordering::Less, amount) {
                    game_scene.inventory_player2.add_item(item_id);
                    state.mod_requirements.append_item(ctx, item_id)?;
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::ITm => {
                let item_id = read_cur_varint(&mut cursor)? as u16;

                game_scene.inventory_player1.consume_item(item_id);
                game_scene.inventory_player2.consume_item(item_id);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::AMp => {
                let weapon_id = read_cur_varint(&mut cursor)? as u8;
                let max_ammo = read_cur_varint(&mut cursor)? as u16;
                let weapon_type: Option<WeaponType> = FromPrimitive::from_u8(weapon_id);

                state.textscript_vm.numbers[0] = max_ammo;

                if let Some(wtype) = weapon_type {
                    game_scene.inventory_player1.add_weapon(wtype, max_ammo);
                    game_scene.inventory_player2.add_weapon(wtype, max_ammo);
                    state.mod_requirements.append_weapon(ctx, weapon_id as u16)?;
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::AMm => {
                let weapon_id = read_cur_varint(&mut cursor)? as u8;
                let weapon_type: Option<WeaponType> = FromPrimitive::from_u8(weapon_id);

                if let Some(wtype) = weapon_type {
                    game_scene.inventory_player1.remove_weapon(wtype);
                    game_scene.inventory_player2.remove_weapon(wtype);
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::AEp => {
                game_scene.inventory_player1.refill_all_ammo();
                game_scene.inventory_player2.refill_all_ammo();

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::TAM => {
                let old_weapon_id = read_cur_varint(&mut cursor)? as u8;
                let new_weapon_id = read_cur_varint(&mut cursor)? as u8;
                let max_ammo = read_cur_varint(&mut cursor)? as u16;
                let old_weapon_type: Option<WeaponType> = FromPrimitive::from_u8(old_weapon_id);
                let new_weapon_type: Option<WeaponType> = FromPrimitive::from_u8(new_weapon_id);

                if let Some(wtype) = new_weapon_type {
                    game_scene.inventory_player1.trade_weapon(old_weapon_type, wtype, max_ammo);
                    game_scene.inventory_player2.trade_weapon(old_weapon_type, wtype, max_ammo);
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::ZAM => {
                game_scene.inventory_player1.reset_all_weapon_xp();
                game_scene.inventory_player2.reset_all_weapon_xp();

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::EQp => {
                let mask = read_cur_varint(&mut cursor)? as u16;

                game_scene.player1.equip.0 |= mask;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::EQm => {
                let mask = read_cur_varint(&mut cursor)? as u16;

                game_scene.player1.equip.0 &= !mask;

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::FLA => {
                game_scene.flash.set_blink();

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::INI => {
                game_scene.player1.flags.0 = 0;
                game_scene.player2.flags.0 = 0;

                exec_state = TextScriptExecutionState::Reset;
            }
            TSCOpCode::ESC => {
                state.next_scene = Some(Box::new(TitleScene::new()));
                state.control_flags.set_tick_world(false);
                state.control_flags.set_control_enabled(false);
                state.control_flags.set_interactions_disabled(true);
                state.textscript_vm.flags.set_cutscene_skip(false);

                exec_state = TextScriptExecutionState::Ended;
            }
            TSCOpCode::SVP => {
                exec_state = TextScriptExecutionState::SaveProfile(event, cursor.position() as u32);
            }
            TSCOpCode::LDP => {
                game_scene.player1.flags.0 = 0;
                game_scene.player2.flags.0 = 0;

                state.control_flags.set_tick_world(false);
                state.control_flags.set_control_enabled(false);
                state.control_flags.set_interactions_disabled(true);

                exec_state = TextScriptExecutionState::LoadProfile;
            }
            TSCOpCode::MPp => {
                let stage_id = read_cur_varint(&mut cursor)? as u16;

                state.set_map_flag(stage_id as usize, true);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::CRE => {
                state.textscript_vm.flags.set_cutscene_skip(false);
                state.control_flags.set_credits_running(true);
                state.creditscript_vm.start();

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::SIL => {
                let number = read_cur_varint(&mut cursor)? as u16;
                log::warn!("<SIL{:04}", number);

                state.textscript_vm.current_illustration = None;
                state.textscript_vm.illustration_state = IllustrationState::FadeIn(-160.0);

                for path in &state.constants.credit_illustration_paths {
                    let path = format!("{}Credit{:02}", path, number);
                    if state.texture_set.find_texture(ctx, &state.constants.base_paths, &path).is_some() {
                        state.textscript_vm.current_illustration = Some(path);
                        break;
                    }
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::CIL => {
                log::warn!("<CIL");
                state.textscript_vm.illustration_state = if state.textscript_vm.current_illustration.is_some() {
                    IllustrationState::FadeOut(0.0)
                } else {
                    IllustrationState::Hidden
                };

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::CPS => {
                state.sound_manager.stop_sfx(58);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::SPS => {
                state.sound_manager.loop_sfx(58);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::CSS => {
                state.sound_manager.stop_sfx(40);
                state.sound_manager.stop_sfx(41);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::SSS => {
                let freq = read_cur_varint(&mut cursor)? as f32 / 2205.0;

                state.sound_manager.loop_sfx_freq(40, freq);
                state.sound_manager.loop_sfx_freq(41, freq);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::XX1 => {
                let mode = read_cur_varint(&mut cursor)?;

                if mode != 0 && !state.mod_requirements.beat_hell {
                    state.mod_requirements.beat_hell = true;
                    state.mod_requirements.save(ctx)?;
                }

                exec_state = TextScriptExecutionState::FallingIsland(
                    event,
                    cursor.position() as u32,
                    0x15000,
                    0x8000,
                    0,
                    mode != 0,
                );
            }
            TSCOpCode::STC => {
                let new_record = game_scene.nikumaru.save_counter(state, ctx)?;

                if new_record && state.replay_state == ReplayState::Recording {
                    game_scene.replay.stop_recording(state, ctx)?;
                }

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::MLP => {
                exec_state = TextScriptExecutionState::MapSystem;
            }
            TSCOpCode::KE2 => {
                state.control_flags.set_tick_world(true);
                state.control_flags.set_ok_button_disabled(true);
                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::FR2 => {
                state.control_flags.set_tick_world(true);
                state.control_flags.set_ok_button_disabled(false);
                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
            TSCOpCode::ACH => {
                // todo: any idea for any practical purpose of that opcode?
                let idx = read_cur_varint(&mut cursor)?;

                log::info!("achievement get: {}", idx);

                exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
            }
        }

        Ok(exec_state)
    }
}

pub struct TextScript {
    pub(in crate::scripting::tsc) event_map: HashMap<u16, Vec<u8>>,
}

impl Clone for TextScript {
    fn clone(&self) -> Self {
        Self { event_map: self.event_map.clone() }
    }
}

impl Default for TextScript {
    fn default() -> Self {
        TextScript::new()
    }
}

impl TextScript {
    pub fn new() -> TextScript {
        Self { event_map: HashMap::new() }
    }

    /// Loads, decrypts and compiles a text script from specified stream.
    pub fn load_from<R: io::Read>(mut data: R, constants: &EngineConstants) -> GameResult<TextScript> {
        let mut buf = Vec::new();
        data.read_to_end(&mut buf)?;

        if constants.textscript.encrypted {
            decrypt_tsc(&mut buf);
        }

        TextScript::compile(&buf, false, constants.textscript.encoding)
    }

    pub fn get_event_ids(&self) -> Vec<u16> {
        let mut vec: Vec<u16> = self.event_map.keys().copied().collect();
        vec.sort();
        vec
    }

    pub fn has_event(&self, id: u16) -> bool {
        self.event_map.contains_key(&id)
    }
}
