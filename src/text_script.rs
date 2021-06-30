use std::cmp::Ordering;
use std::collections::HashMap;
use std::io;
use std::io::Cursor;
use std::io::Seek;
use std::io::SeekFrom;
use std::iter::Peekable;
use std::ops::Not;
use std::str::FromStr;

use byteorder::ReadBytesExt;
use itertools::Itertools;
use num_derive::FromPrimitive;
use num_traits::{clamp, FromPrimitive};

use crate::bitfield;
use crate::common::{Direction, FadeDirection, FadeState, Rect};
use crate::encoding::{read_cur_shift_jis, read_cur_wtf8};
use crate::engine_constants::EngineConstants;
use crate::entity::GameEntity;
use crate::frame::UpdateTarget;
use crate::framework::context::Context;
use crate::framework::error::GameError::{InvalidValue, ParseError};
use crate::framework::error::GameResult;
use crate::input::touch_controls::TouchControlType;
use crate::npc::NPC;
use crate::player::{ControlMode, TargetPlayer};
use crate::scene::game_scene::GameScene;
use crate::scene::title_scene::TitleScene;
use crate::shared_game_state::SharedGameState;
use crate::str;
use crate::weapon::WeaponType;

/// Engine's text script VM operation codes.
#[derive(EnumString, Debug, FromPrimitive, PartialEq)]
#[repr(i32)]
pub enum OpCode {
    // ---- Internal opcodes (used by bytecode, no TSC representation)
    /// internal: no operation
    _NOP = 0,
    /// internal: unimplemented
    _UNI,
    /// internal: string marker
    _STR,
    /// internal: implicit END marker
    _END,

    // ---- Vanilla opcodes ----
    /// <BOAxxxx, Starts boss animation
    BOA,
    /// <BSLxxxx, Starts boss fight
    BSL,

    /// <FOBxxxx:yyyy, Focuses on boss part xxxx and sets speed to yyyy ticks
    FOB,
    /// <FOMxxxx, Focuses on player and sets speed to xxxx
    FOM,
    /// <FONxxxx:yyyy, Focuses on NPC tagged with event xxxx and sets speed to yyyy
    FON,
    /// <FLA, Flashes screen
    FLA,
    /// <QUAxxxx, Starts quake for xxxx ticks
    QUA,

    /// <UNIxxxx, Sets player movement mode (0 = normal, 1 = main artery)
    UNI,
    /// <HMC, Hides the player
    HMC,
    /// <SMC, Shows the player
    SMC,
    /// <MM0, Halts horizontal movement
    MM0,
    /// <MOVxxxx:yyyy, Moves the player to tile (xxxx,yyyy)
    MOV,
    /// <MYBxxxx, Bumps the player from direction xxxx
    MYB,
    /// <MYDxxxx, Makes the player face direction xxxx
    MYD,
    /// <TRAxxxx:yyyy:zzzz:wwww, Travels to map xxxx, starts event yyyy, places the player at tile (zzzz,wwww)
    TRA,

    /// <END, Ends the current event
    END,
    /// <FRE, Starts world ticking and unlocks player controls.
    FRE,
    /// <FAIxxxx, Fades in with direction xxxx
    FAI,
    /// <FAOxxxx, Fades out with direction xxxx
    FAO,
    /// <WAIxxxx, Waits for xxxx frames
    WAI,
    /// <WASs, Waits until the player is standing
    WAS,
    /// <KEY, Locks out the player controls.
    KEY,
    /// <PRI, Stops world ticking and locks out player controls.
    PRI,
    /// <NOD, Waits for input
    NOD,
    /// <CAT, Instantly displays the text, works for entire event
    CAT,
    /// <SAT, Same as <CAT
    SAT,
    /// <TUR, Instantly displays the text, works until <MSG/2/3 or <END
    TUR,
    /// <CLO, Closes the text box
    CLO,
    /// <CLR, Clears the text box
    CLR,
    /// <FACxxxx, Shows the face xxxx in text box, 0 to hide,
    /// CS+ Switch extensions:
    /// - add 0100 to display talking animation (requires faceanm.dat)
    /// - add 1000 to the number to display the face in opposite direction. (works on any CS, including freeware mods)
    FAC,
    /// <GITxxxx, Shows the item xxxx above text box, 0 to hide
    GIT,
    /// <MS2, Displays text on top of the screen without background.
    MS2,
    /// <MS3, Displays text on top of the screen with background.
    MS3,
    /// <MSG, Displays text on bottom of the screen with background.
    MSG,
    /// <NUMxxxx, Displays a value from AM+, buggy in vanilla.
    NUM,

    /// <ANPxxxx:yyyy:zzzz, Changes the animation state of NPC tagged with
    /// event xxxx to yyyy and set the direction to zzzz
    ANP,
    /// <CNPxxxx:yyyy:zzzz, Changes the NPC tagged with event xxxx to type yyyy
    /// and makes it face direction zzzz
    CNP,
    /// <INPxxxx:yyyy:zzzz, Same as <CNP, but also sets NPC flag event_when_touched (0x100)
    INP,
    /// <MNPxxxx:yyyy:zzzz:wwww, Moves NPC tagged with event xxxx to tile position (xxxx,yyyy)
    /// and makes it face direction zzzz
    MNP,
    /// <DNAxxxx, Deletes all NPCs of type xxxx
    DNA,
    /// <DNPxxxx, Deletes all NPCs of type xxxx
    DNP,
    SNP,

    /// <FL-xxxx, Sets the flag xxxx to false
    #[strum(serialize = "FL-")]
    FLm,
    /// <FL+xxxx, Sets the flag xxxx to true
    #[strum(serialize = "FL+")]
    FLp,
    /// <MP-xxxx, Sets the map xxxx to true
    #[strum(serialize = "MP+")]
    MPp,
    /// <SK-xxxx, Sets the skip flag xxx to false
    #[strum(serialize = "SK-")]
    SKm,
    /// <SK+xxxx, Sets the skip flag xxx to true
    #[strum(serialize = "SK+")]
    SKp,

    /// <EQ+xxxx, Sets specified bits in equip bitfield
    #[strum(serialize = "EQ+")]
    EQp,
    /// <EQ-xxxx, Unsets specified bits in equip bitfield
    #[strum(serialize = "EQ-")]
    EQm,
    /// <ML+xxxx, Adds xxxx to maximum health.
    #[strum(serialize = "ML+")]
    MLp,
    /// <IT+xxxx, Adds item xxxx to players inventory.
    #[strum(serialize = "IT+")]
    ITp,
    /// <IT-xxxx, Removes item xxxx to players inventory.
    #[strum(serialize = "IT-")]
    ITm,
    /// <AM+xxxx:yyyy, Adds weapon xxxx with yyyy ammo (0 = infinite) to players inventory.
    #[strum(serialize = "AM+")]
    AMp,
    /// <AM-xxxx, Removes weapon xxxx from players inventory.
    #[strum(serialize = "AM-")]
    AMm,
    /// <TAMxxxx:yyyy:zzzz, Trades weapon xxxx for weapon yyyy with zzzz ammo
    TAM,

    /// <UNJxxxx, Jumps to event xxxx if no damage has been taken
    UNJ,
    /// <NCJxxxx:yyyy, Jumps to event xxxx if NPC of type yyyy is alive
    NCJ,
    /// <ECJxxxx:yyyy, Jumps to event xxxx if NPC tagged with event yyyy is alive
    ECJ,
    /// <FLJxxxx:yyyy, Jumps to event yyyy if flag xxxx is set
    FLJ,
    /// <FLJxxxx:yyyy, Jumps to event xxxx if player has item yyyy
    ITJ,
    /// <MPJxxxx, Jumps to event xxxx if map flag for current stage is set
    MPJ,
    /// <YNJxxxx, Jumps to event xxxx if prompt response is No, otherwise continues event execution
    YNJ,
    /// <MPJxxxx, Jumps to event xxxx if skip flag for is set
    SKJ,
    /// <EVExxxx, Jumps to event xxxx
    EVE,
    /// <AMJyyyy, Jumps to event xxxx player has weapon yyyy
    AMJ,

    /// <MLP, Displays the map of current stage
    MLP,
    /// <MLP, Displays the name of current stage
    MNA,
    /// <CMPxxxx:yyyy:zzzz, Sets the tile at (xxxx,yyyy) to type zzzz
    CMP,
    /// <SMPxxxx:yyyy:zzzz, Subtracts 1 from tile type at (xxxx,yyyy)
    SMP,

    /// <CRE, Shows credits
    CRE,
    /// <XX1xxxx, Shows falling island
    XX1,
    /// <CIL, Hides credits illustration
    CIL,
    /// <SILxxxx, Shows credits illustration xxxx
    SIL,
    /// <ESC, Exits to title screen
    ESC,
    /// <ESC, Exits to credits
    INI,
    /// <LDP, Loads a saved game
    LDP,
    /// <PS+xxxx:yyyy, Sets teleporter slot xxxx to event number yyyy
    #[strum(serialize = "PS+")]
    PSp,
    /// <SLP, Shows the teleporter menu
    SLP,
    /// <ZAM, Resets the experience and level of all weapons
    ZAM,

    /// <AE+, Refills ammunition
    #[strum(serialize = "AE+")]
    AEp,
    /// <LI+xxxx, Recovers xxxx health
    #[strum(serialize = "LI+")]
    LIp,

    /// <SVP, Saves the current game
    SVP,
    /// <STC, Saves the state of Nikumaru counter
    STC,

    /// <SOUxxxx, Plays sound effect xxxx
    SOU,
    /// <CMUxxxx, Changes BGM to xxxx
    CMU,
    /// <FMU, Fades the BGM
    FMU,
    /// <RMU, Restores the music state of BGM played before current one
    RMU,
    /// <CPS, Stops the propeller sound
    CPS,
    /// <SPS, Starts the propeller sound
    SPS,
    /// <CSS, Stops the stream sound
    CSS,
    /// <SSSxxxx, Starts the stream sound at volume xxxx
    SSS,

    // ---- Cave Story+ specific opcodes ----
    /// <ACHxxxx, triggers a Steam achievement. No-op in EGS/Humble Bundle version.
    ACH,

    // ---- Cave Story+ (Switch) specific opcodes ----
    /// <HM2, HMC only for executor player.
    HM2,
    /// <2MVxxxx, Put another player near the player who executed the event.
    /// 0000 - puts player on left side of executor player
    /// 0001 - puts player on right side of executor player
    /// 0002-0010 - unused
    /// 0011.. - the first 3 digits are distance in pixels, the last digit is a flag
    ///        - if it's 1 put the player on right side of the player, otherwise put it on left
    #[strum(serialize = "2MV")]
    S2MV,
    /// <INJxxxx:yyyy:zzzz, Jumps to event zzzz if amount of item xxxx equals yyyy
    INJ,
    /// <I+Nxxxx:yyyy, Adds item xxxx with maximum amount of yyyy
    #[strum(serialize = "I+N")]
    IpN,
    /// <FF-xxxx:yyyy, Sets first flag in range xxxx-yyyy to false
    #[strum(serialize = "FF-")]
    FFm,
    /// <PSHxxxx, Pushes text script state to stack and starts event xxxx
    PSH,
    /// <POP, Restores text script state from stack and resumes previous event.
    POP,
    /// <KEY related to player 2?
    KE2,
    /// <FRE related to player 2?
    FR2,
    // ---- Custom opcodes, for use by modders ----
}

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
#[repr(u8)]
pub enum TextScriptExecutionState {
    Ended,
    Running(u16, u32),
    Msg(u16, u32, u32, u8),
    WaitTicks(u16, u32, u16),
    WaitInput(u16, u32, u16),
    WaitStanding(u16, u32),
    WaitConfirmation(u16, u32, u16, u8, ConfirmSelection),
    WaitFade(u16, u32),
    SaveProfile(u16, u32),
    LoadProfile,
    Reset,
}

pub struct TextScriptVM {
    pub scripts: Scripts,
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
    pub numbers: [u16; 4],
    pub face: u16,
    pub item: u16,
    pub current_line: TextScriptLine,
    pub line_1: Vec<char>,
    pub line_2: Vec<char>,
    pub line_3: Vec<char>,
    prev_char: char,
}

impl Default for TextScriptVM {
    fn default() -> Self {
        TextScriptVM::new()
    }
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

fn read_cur_varint(cursor: &mut Cursor<&Vec<u8>>) -> GameResult<i32> {
    let mut result = 0u32;

    for o in 0..5 {
        let n = cursor.read_u8()?;
        result |= (n as u32 & 0x7f) << (o * 7);

        if n & 0x80 == 0 {
            break;
        }
    }

    Ok(((result << 31) ^ (result >> 1)) as i32)
}

impl TextScriptVM {
    pub fn new() -> Self {
        Self {
            scripts: Scripts {
                global_script: TextScript::new(),
                scene_script: TextScript::new(),
                inventory_script: TextScript::new(),
                stage_select_script: TextScript::new(),
            },
            state: TextScriptExecutionState::Ended,
            stack: Vec::with_capacity(6),
            flags: TextScriptFlags(0),
            mode: ScriptMode::Map,
            executor_player: TargetPlayer::Player1,
            strict_mode: false,
            suspend: true,
            numbers: [0; 4],
            face: 0,
            item: 0,
            current_line: TextScriptLine::Line1,
            line_1: Vec::with_capacity(24),
            line_2: Vec::with_capacity(24),
            line_3: Vec::with_capacity(24),
            prev_char: '\x00',
        }
    }

    pub fn set_global_script(&mut self, script: TextScript) {
        self.scripts.global_script = script;
        if !self.suspend {
            self.reset();
        }
    }

    pub fn set_scene_script(&mut self, script: TextScript) {
        self.scripts.scene_script = script;
        if !self.suspend {
            self.reset();
        }
    }

    pub fn set_inventory_script(&mut self, script: TextScript) {
        self.scripts.inventory_script = script;
    }

    pub fn set_stage_select_script(&mut self, script: TextScript) {
        self.scripts.stage_select_script = script;
    }

    pub fn reset(&mut self) {
        self.state = TextScriptExecutionState::Ended;
        self.flags.0 = 0;
        self.clear_text_box();
    }

    pub fn clear_text_box(&mut self) {
        self.face = 0;
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
        self.state = TextScriptExecutionState::Running(event_num, 0);

        log::info!("Started script: #{:04}", event_num);
    }

    pub fn run(state: &mut SharedGameState, game_scene: &mut GameScene, ctx: &mut Context) -> GameResult {
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
                    state.textscript_vm.state = TextScriptVM::execute(event, ip, state, game_scene, ctx)?;

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

                    if let Some(bytecode) = state.textscript_vm.scripts.find_script(state.textscript_vm.mode, event) {
                        let mut cursor = Cursor::new(bytecode);
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
                                state.textscript_vm.line_1.clear();
                                state.textscript_vm.line_1.append(&mut state.textscript_vm.line_2);
                                state.textscript_vm.line_2.append(&mut state.textscript_vm.line_3);
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
                                    state.textscript_vm.line_1.clear();
                                    state.textscript_vm.line_1.append(&mut state.textscript_vm.line_2);
                                    state.textscript_vm.line_2.append(&mut state.textscript_vm.line_3);
                                }
                            }
                            _ => {}
                        }

                        if remaining > 1 {
                            let ticks = if state.textscript_vm.flags.fast() || state.textscript_vm.flags.cutscene_skip() {
                                0
                            } else if remaining != 2 && (game_scene.player1.controller.jump()
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

                            state.textscript_vm.state =
                                TextScriptExecutionState::Msg(event, cursor.position() as u32, remaining - 1, ticks);
                        } else {
                            state.textscript_vm.state =
                                TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    } else {
                        state.textscript_vm.reset();
                    }
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
            }
        }

        Ok(())
    }

    pub fn execute(
        event: u16,
        ip: u32,
        state: &mut SharedGameState,
        game_scene: &mut GameScene,
        ctx: &mut Context,
    ) -> GameResult<TextScriptExecutionState> {
        let mut exec_state = state.textscript_vm.state;

        let state_ref = state as *mut SharedGameState;
        let scripts = unsafe { &(*state_ref).textscript_vm.scripts };

        if let Some(bytecode) = scripts.find_script(state.textscript_vm.mode, event) {
            let mut cursor = Cursor::new(bytecode);
            cursor.seek(SeekFrom::Start(ip as u64))?;

            let op_maybe: Option<OpCode> =
                FromPrimitive::from_i32(read_cur_varint(&mut cursor).unwrap_or_else(|_| OpCode::END as i32));

            if let Some(op) = op_maybe {
                match op {
                    OpCode::_NOP => {
                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::_UNI => {}
                    OpCode::_STR => {
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
                    OpCode::_END => {
                        state.textscript_vm.flags.set_cutscene_skip(false);
                        exec_state = TextScriptExecutionState::Ended;
                    }
                    OpCode::END => {
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
                    OpCode::SLP => {
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
                    OpCode::PSp => {
                        let index = read_cur_varint(&mut cursor)? as u16;
                        let event_num = read_cur_varint(&mut cursor)? as u16;

                        if let Some(slot) = state.teleporter_slots.iter_mut().find(|s| s.0 == index) {
                            slot.1 = event_num;
                        } else {
                            state.teleporter_slots.push((index, event_num));
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::PRI => {
                        state.control_flags.set_tick_world(false);
                        state.control_flags.set_control_enabled(false);

                        game_scene.player1.shock_counter = 0;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::KEY => {
                        state.control_flags.set_tick_world(true);
                        state.control_flags.set_control_enabled(false);

                        game_scene.player1.up = false;
                        game_scene.player1.shock_counter = 0;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FRE => {
                        state.control_flags.set_tick_world(true);
                        state.control_flags.set_control_enabled(true);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::MYD => {
                        let new_direction = read_cur_varint(&mut cursor)? as usize;
                        if let Some(direction) = Direction::from_int(new_direction) {
                            game_scene.player1.direction = direction;
                            game_scene.player2.direction = direction;
                        }
                        game_scene.player1.cond.set_interacted(false);
                        game_scene.player2.cond.set_interacted(false);

                        game_scene.player1.vel_x = 0;
                        game_scene.player2.vel_x = 0;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::MYB => {
                        let new_direction = read_cur_varint(&mut cursor)? as usize;

                        game_scene.player1.vel_y = -0x200;
                        game_scene.player2.vel_y = -0x200;

                        // Reset interaction condition, needed for places like talking to Toroko in shack
                        game_scene.player1.cond.set_interacted(false);
                        game_scene.player2.cond.set_interacted(false);

                        if let Some(direction) = Direction::from_int_facing(new_direction) {
                            match direction {
                                Direction::Left => {
                                    game_scene.player1.vel_x = 0x200;
                                    game_scene.player2.vel_x = 0x200;
                                }
                                Direction::Up => {
                                    game_scene.player1.vel_y = -0x200;
                                    game_scene.player2.vel_y = -0x200;
                                }
                                Direction::Right => {
                                    game_scene.player1.vel_x = -0x200;
                                    game_scene.player2.vel_x = -0x200;
                                }
                                Direction::Bottom => {
                                    game_scene.player1.vel_y = 0x200;
                                    game_scene.player2.vel_y = 0x200;
                                }
                                Direction::FacingPlayer => {
                                    // todo npc direction dependent bump
                                }
                            }
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::SMC => {
                        game_scene.player1.cond.set_hidden(false);
                        game_scene.player2.cond.set_hidden(false);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::HMC => {
                        game_scene.player1.cond.set_hidden(true);
                        game_scene.player2.cond.set_hidden(true);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::WAI => {
                        let ticks = read_cur_varint(&mut cursor)? as u16;

                        exec_state = TextScriptExecutionState::WaitTicks(event, cursor.position() as u32, ticks);
                    }
                    OpCode::WAS => {
                        exec_state = TextScriptExecutionState::WaitStanding(event, cursor.position() as u32);
                    }
                    OpCode::NOD => {
                        exec_state = TextScriptExecutionState::WaitInput(event, cursor.position() as u32, 0);
                    }
                    OpCode::FLp | OpCode::FLm => {
                        let flag_num = read_cur_varint(&mut cursor)? as u16;
                        state.set_flag(flag_num as usize, op == OpCode::FLp);
                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::SKp | OpCode::SKm => {
                        let flag_num = read_cur_varint(&mut cursor)? as u16;
                        state.set_skip_flag(flag_num as usize, op == OpCode::SKp);
                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FFm => {
                        let flag_from = read_cur_varint(&mut cursor)? as usize;
                        let flag_to = read_cur_varint(&mut cursor)? as usize;

                        if flag_to >= flag_from {
                            for flag in flag_from..=flag_to {
                                if state.get_flag(flag) {
                                    state.game_flags.set(flag, false);
                                    break;
                                }
                            }
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FLJ => {
                        let flag_num = read_cur_varint(&mut cursor)? as usize;
                        let event_num = read_cur_varint(&mut cursor)? as u16;
                        if state.get_flag(flag_num) {
                            state.textscript_vm.clear_text_box();
                            exec_state = TextScriptExecutionState::Running(event_num, 0);
                        } else {
                            exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    }
                    OpCode::MPJ => {
                        let event_num = read_cur_varint(&mut cursor)? as u16;

                        if state.get_map_flag(game_scene.stage_id) {
                            state.textscript_vm.clear_text_box();
                            exec_state = TextScriptExecutionState::Running(event_num, 0);
                        } else {
                            exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    }
                    OpCode::ITJ => {
                        let item_id = read_cur_varint(&mut cursor)? as u16;
                        let event_num = read_cur_varint(&mut cursor)? as u16;

                        if game_scene.inventory_player1.has_item(item_id) {
                            state.textscript_vm.clear_text_box();
                            exec_state = TextScriptExecutionState::Running(event_num, 0);
                        } else {
                            exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    }
                    OpCode::INJ => {
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
                    OpCode::AMJ => {
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
                    OpCode::NCJ => {
                        let npc_type = read_cur_varint(&mut cursor)? as u16;
                        let event_num = read_cur_varint(&mut cursor)? as u16;

                        if game_scene.npc_list.is_alive_by_type(npc_type) {
                            state.textscript_vm.clear_text_box();
                            exec_state = TextScriptExecutionState::Running(event_num, 0);
                        } else {
                            exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    }
                    OpCode::ECJ => {
                        let npc_event_num = read_cur_varint(&mut cursor)? as u16;
                        let event_num = read_cur_varint(&mut cursor)? as u16;

                        if game_scene.npc_list.is_alive_by_event(npc_event_num) {
                            state.textscript_vm.clear_text_box();
                            exec_state = TextScriptExecutionState::Running(event_num, 0);
                        } else {
                            exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    }
                    OpCode::SKJ => {
                        let flag_id = read_cur_varint(&mut cursor)? as u16;
                        let event_num = read_cur_varint(&mut cursor)? as u16;

                        if state.get_skip_flag(flag_id as usize) {
                            state.textscript_vm.clear_text_box();
                            exec_state = TextScriptExecutionState::Running(event_num, 0);
                        } else {
                            exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                        }
                    }
                    OpCode::EVE => {
                        let event_num = read_cur_varint(&mut cursor)? as u16;

                        state.textscript_vm.clear_text_box();
                        exec_state = TextScriptExecutionState::Running(event_num, 0);
                    }
                    OpCode::PSH => {
                        let event_num = read_cur_varint(&mut cursor)? as u16;

                        let saved_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                        state.textscript_vm.stack.push(saved_state);

                        state.textscript_vm.clear_text_box();
                        exec_state = TextScriptExecutionState::Running(event_num, 0);
                    }
                    OpCode::POP => {
                        if let Some(saved_state) = state.textscript_vm.stack.pop() {
                            exec_state = saved_state;
                        } else {
                            log::warn!("Tried to <POP from TSC stack without saved state!");
                            exec_state = TextScriptExecutionState::Ended;
                        }
                    }
                    OpCode::MM0 => {
                        game_scene.player1.vel_x = 0;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::SMP => {
                        let pos_x = read_cur_varint(&mut cursor)? as usize;
                        let pos_y = read_cur_varint(&mut cursor)? as usize;

                        let tile_type = game_scene.stage.tile_at(pos_x, pos_y);
                        game_scene.stage.change_tile(pos_x, pos_y, tile_type.wrapping_sub(1));

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::CMP => {
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
                    OpCode::MLp => {
                        let life = read_cur_varint(&mut cursor)? as u16;
                        game_scene.player1.life += life;
                        game_scene.player1.max_life += life;
                        game_scene.player2.life += life;
                        game_scene.player2.max_life += life;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FAC => {
                        let face = read_cur_varint(&mut cursor)? as u16;
                        state.textscript_vm.face = face;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::CLR => {
                        state.textscript_vm.current_line = TextScriptLine::Line1;
                        state.textscript_vm.line_1.clear();
                        state.textscript_vm.line_2.clear();
                        state.textscript_vm.line_3.clear();

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::MSG | OpCode::MS2 | OpCode::MS3 => {
                        state.textscript_vm.current_line = TextScriptLine::Line1;
                        state.textscript_vm.line_1.clear();
                        state.textscript_vm.line_2.clear();
                        state.textscript_vm.line_3.clear();
                        state.textscript_vm.flags.set_render(true);
                        state.textscript_vm.flags.set_background_visible(op != OpCode::MS2);
                        state.textscript_vm.flags.set_fast(state.textscript_vm.flags.perma_fast());
                        state.textscript_vm.flags.set_position_top(op != OpCode::MSG);
                        if op == OpCode::MS2 {
                            state.textscript_vm.face = 0;
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::SAT | OpCode::CAT => {
                        state.textscript_vm.flags.set_perma_fast(true);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }

                    OpCode::TUR => {
                        state.textscript_vm.flags.set_fast(true);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::CLO => {
                        state.textscript_vm.flags.set_render(false);
                        state.textscript_vm.flags.set_background_visible(false);
                        state.textscript_vm.flags.set_fast(false);
                        state.textscript_vm.flags.set_position_top(false);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::YNJ => {
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
                    OpCode::NUM => {
                        let index = read_cur_varint(&mut cursor)? as usize;

                        if let Some(num) = state.textscript_vm.numbers.get(index) {
                            let mut str = num.to_string().chars().collect_vec();

                            match state.textscript_vm.current_line {
                                TextScriptLine::Line1 => state.textscript_vm.line_1.append(&mut str),
                                TextScriptLine::Line2 => state.textscript_vm.line_2.append(&mut str),
                                TextScriptLine::Line3 => state.textscript_vm.line_3.append(&mut str),
                            }
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::GIT => {
                        let item = read_cur_varint(&mut cursor)? as u16;
                        state.textscript_vm.item = item;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::TRA => {
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
                        new_scene.frame.wait = game_scene.frame.wait;

                        let skip = state.textscript_vm.flags.cutscene_skip();
                        state.control_flags.set_tick_world(true);
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
                    OpCode::MOV => {
                        let block_size = state.tile_size.as_int() * 0x200;

                        let pos_x = read_cur_varint(&mut cursor)? as i32 * block_size;
                        let pos_y = read_cur_varint(&mut cursor)? as i32 * block_size;

                        for player in [&mut game_scene.player1, &mut game_scene.player2].iter_mut() {
                            player.vel_x = 0;
                            player.vel_y = 0;
                            player.x = pos_x;
                            player.y = pos_y;
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::S2MV => {
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
                                partner.x =
                                    executor.x + if (param % 10) == 1 { distance * 0x200 } else { -distance * 0x200 };
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
                    OpCode::UNI => {
                        let control_mode = read_cur_varint(&mut cursor)? as u8;

                        let mode: Option<ControlMode> = FromPrimitive::from_u8(control_mode);
                        if let Some(mode) = mode {
                            game_scene.player1.control_mode = mode;
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FAI => {
                        let fade_type = read_cur_varint(&mut cursor)? as usize;

                        if let Some(direction) = FadeDirection::from_int(fade_type) {
                            state.fade_state = FadeState::FadeIn(15, direction);
                        }

                        exec_state = TextScriptExecutionState::WaitFade(event, cursor.position() as u32);
                    }
                    OpCode::FAO => {
                        let fade_type = read_cur_varint(&mut cursor)? as usize;

                        if let Some(direction) = FadeDirection::from_int(fade_type) {
                            state.fade_state = FadeState::FadeOut(-15, direction.opposite());
                        }

                        exec_state = TextScriptExecutionState::WaitFade(event, cursor.position() as u32);
                    }
                    OpCode::QUA => {
                        let count = read_cur_varint(&mut cursor)? as u16;

                        state.quake_counter = count;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::MNA => {
                        game_scene.display_map_name(160);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::CMU => {
                        let song_id = read_cur_varint(&mut cursor)? as usize;
                        state.sound_manager.play_song(song_id, &state.constants, &state.settings, ctx)?;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FMU => {
                        state.sound_manager.play_song(0, &state.constants, &state.settings, ctx)?;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::RMU => {
                        state.sound_manager.restore_state()?;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::SOU => {
                        let sound = read_cur_varint(&mut cursor)? as u8;

                        state.sound_manager.play_sfx(sound);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::DNP => {
                        let event_num = read_cur_varint(&mut cursor)? as u16;

                        game_scene.npc_list.remove_by_event(event_num, state);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::DNA => {
                        let npc_remove_type = read_cur_varint(&mut cursor)? as u16;

                        game_scene.npc_list.remove_by_type(npc_remove_type, state);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FOB => {
                        let part_id = read_cur_varint(&mut cursor)? as u16;
                        let ticks = read_cur_varint(&mut cursor)? as i32;

                        game_scene.frame.wait = ticks;
                        game_scene.frame.update_target = UpdateTarget::Boss(part_id);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FOM => {
                        let ticks = read_cur_varint(&mut cursor)? as i32;
                        game_scene.frame.wait = ticks;
                        game_scene.frame.update_target = UpdateTarget::Player;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FON => {
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
                    OpCode::BSL => {
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
                    OpCode::BOA => {
                        let action_num = read_cur_varint(&mut cursor)? as u16;

                        game_scene.boss.parts[0].action_num = action_num;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::ANP => {
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
                                        if game_scene.player1.x < npc.x { Direction::Right } else { Direction::Left };
                                } else {
                                    npc.direction = direction;
                                }
                            }
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::CNP | OpCode::INP => {
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

                                if op == OpCode::INP {
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
                                        if game_scene.player1.x < npc.x { Direction::Right } else { Direction::Left };
                                } else {
                                    npc.direction = direction;
                                }

                                npc.tick(
                                    state,
                                    (
                                        [&mut game_scene.player1, &mut game_scene.player2],
                                        &game_scene.npc_list,
                                        &mut game_scene.stage,
                                        &game_scene.bullet_manager,
                                        &mut game_scene.flash,
                                    ),
                                )?;
                            }
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::MNP => {
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
                                        if game_scene.player1.x < npc.x { Direction::Right } else { Direction::Left };
                                } else {
                                    npc.direction = direction;
                                }

                                break;
                            }
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::SNP => {
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
                            npc.direction =
                                if game_scene.player1.x < npc.x { Direction::Right } else { Direction::Left };
                        } else {
                            npc.direction = direction;
                        }

                        let _ = game_scene.npc_list.spawn(0x100, npc);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::LIp => {
                        let life = read_cur_varint(&mut cursor)? as u16;

                        game_scene.player1.life = clamp(game_scene.player1.life + life, 0, game_scene.player1.max_life);
                        game_scene.player2.life = clamp(game_scene.player2.life + life, 0, game_scene.player2.max_life);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::ITp => {
                        let item_id = read_cur_varint(&mut cursor)? as u16;

                        state.sound_manager.play_sfx(38);

                        if !game_scene.inventory_player1.has_item(item_id) {
                            game_scene.inventory_player1.add_item(item_id);
                        }

                        if !game_scene.inventory_player2.has_item(item_id) {
                            game_scene.inventory_player2.add_item(item_id);
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::IpN => {
                        let item_id = read_cur_varint(&mut cursor)? as u16;
                        let amount = read_cur_varint(&mut cursor)? as u16;

                        if game_scene.inventory_player1.has_item_amount(item_id, Ordering::Less, amount) {
                            game_scene.inventory_player1.add_item(item_id);
                        }

                        if game_scene.inventory_player2.has_item_amount(item_id, Ordering::Less, amount) {
                            game_scene.inventory_player2.add_item(item_id);
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::ITm => {
                        let item_id = read_cur_varint(&mut cursor)? as u16;

                        game_scene.inventory_player1.consume_item(item_id);
                        game_scene.inventory_player2.consume_item(item_id);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::AMp => {
                        let weapon_id = read_cur_varint(&mut cursor)? as u8;
                        let max_ammo = read_cur_varint(&mut cursor)? as u16;
                        let weapon_type: Option<WeaponType> = FromPrimitive::from_u8(weapon_id);

                        state.textscript_vm.numbers[0] = max_ammo;

                        if let Some(wtype) = weapon_type {
                            game_scene.inventory_player1.add_weapon(wtype, max_ammo);
                            game_scene.inventory_player2.add_weapon(wtype, max_ammo);
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::AMm => {
                        let weapon_id = read_cur_varint(&mut cursor)? as u8;
                        let weapon_type: Option<WeaponType> = FromPrimitive::from_u8(weapon_id);

                        if let Some(wtype) = weapon_type {
                            game_scene.inventory_player1.remove_weapon(wtype);
                            game_scene.inventory_player2.remove_weapon(wtype);
                        }

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::AEp => {
                        game_scene.inventory_player1.refill_all_ammo();
                        game_scene.inventory_player2.refill_all_ammo();

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::TAM => {
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
                    OpCode::ZAM => {
                        game_scene.inventory_player1.reset_all_weapon_xp();
                        game_scene.inventory_player2.reset_all_weapon_xp();

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::EQp => {
                        let mask = read_cur_varint(&mut cursor)? as u16;

                        game_scene.player1.equip.0 |= mask;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::EQm => {
                        let mask = read_cur_varint(&mut cursor)? as u16;

                        game_scene.player1.equip.0 &= !mask;

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::FLA => {
                        game_scene.flash.set_blink();

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::INI => {
                        exec_state = TextScriptExecutionState::Reset;
                    }
                    OpCode::ESC => {
                        state.next_scene = Some(Box::new(TitleScene::new()));
                        state.control_flags.set_tick_world(false);
                        state.control_flags.set_control_enabled(false);
                        state.control_flags.set_interactions_disabled(true);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    OpCode::SVP => {
                        exec_state = TextScriptExecutionState::SaveProfile(event, cursor.position() as u32);
                    }
                    OpCode::LDP => {
                        state.control_flags.set_tick_world(false);
                        state.control_flags.set_control_enabled(false);
                        state.control_flags.set_interactions_disabled(true);

                        exec_state = TextScriptExecutionState::LoadProfile;
                    }
                    OpCode::MPp => {
                        let stage_id = read_cur_varint(&mut cursor)? as u16;

                        state.set_map_flag(stage_id as usize, true);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    // unimplemented opcodes
                    // Zero operands
                    OpCode::CIL
                    | OpCode::CPS
                    | OpCode::KE2
                    | OpCode::CRE
                    | OpCode::CSS
                    | OpCode::MLP
                    | OpCode::SPS
                    | OpCode::FR2
                    | OpCode::STC
                    | OpCode::HM2 => {
                        log::warn!("unimplemented opcode: {:?}", op);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                    // One operand codes
                    OpCode::UNJ | OpCode::XX1 | OpCode::SIL | OpCode::SSS | OpCode::ACH => {
                        let par_a = read_cur_varint(&mut cursor)?;

                        log::warn!("unimplemented opcode: {:?} {}", op, par_a);

                        exec_state = TextScriptExecutionState::Running(event, cursor.position() as u32);
                    }
                }
            } else {
                exec_state = TextScriptExecutionState::Ended;
            }
        } else {
            return Ok(TextScriptExecutionState::Ended);
        }

        Ok(exec_state)
    }
}

pub struct TextScript {
    event_map: HashMap<u16, Vec<u8>>,
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
            let half = buf.len() / 2;
            let key = if let Some(0) = buf.get(half) { 0xf9 } else { (-(*buf.get(half).unwrap() as isize)) as u8 };
            log::info!("Decrypting TSC using key {:#x}", key);

            for (idx, byte) in buf.iter_mut().enumerate() {
                if idx == half {
                    continue;
                }

                *byte = byte.wrapping_add(key);
            }
        }

        TextScript::compile(&buf, false, constants.textscript.encoding)
    }

    pub fn get_event_ids(&self) -> Vec<u16> {
        self.event_map.keys().copied().sorted().collect_vec()
    }

    /// Compiles a decrypted text script data into internal bytecode.
    pub fn compile(data: &[u8], strict: bool, encoding: TextScriptEncoding) -> GameResult<TextScript> {
        log::info!("data: {}", String::from_utf8_lossy(data));

        let mut event_map = HashMap::new();
        let mut iter = data.iter().copied().peekable();
        let mut last_event = 0;

        while let Some(&chr) = iter.peek() {
            match chr {
                b'#' => {
                    iter.next();
                    let event_num = TextScript::read_number(&mut iter)? as u16;
                    if iter.peek().is_some() {
                        TextScript::skip_until(b'\n', &mut iter)?;
                        iter.next();
                    }
                    last_event = event_num;

                    if event_map.contains_key(&event_num) {
                        if strict {
                            return Err(ParseError(format!("Event {} has been defined twice.", event_num)));
                        }

                        match TextScript::skip_until(b'#', &mut iter).ok() {
                            Some(_) => {
                                continue;
                            }
                            None => {
                                break;
                            }
                        }
                    }

                    let bytecode = TextScript::compile_event(&mut iter, strict, encoding)?;
                    log::info!("Successfully compiled event #{} ({} bytes generated).", event_num, bytecode.len());
                    event_map.insert(event_num, bytecode);
                }
                b'\r' | b'\n' | b' ' | b'\t' => {
                    iter.next();
                }
                n => {
                    // CS+ boss rush is the buggiest shit ever.
                    if !strict && last_event == 0 {
                        iter.next();
                        continue;
                    }

                    return Err(ParseError(format!("Unexpected token in event {}: {}", last_event, n as char)));
                }
            }
        }

        Ok(TextScript { event_map })
    }

    fn compile_event<I: Iterator<Item = u8>>(
        iter: &mut Peekable<I>,
        strict: bool,
        encoding: TextScriptEncoding,
    ) -> GameResult<Vec<u8>> {
        let mut bytecode = Vec::new();
        let mut char_buf = Vec::with_capacity(16);
        let mut allow_next_event = true;

        while let Some(&chr) = iter.peek() {
            match chr {
                b'#' if allow_next_event => {
                    if !char_buf.is_empty() {
                        TextScript::put_string(&mut char_buf, &mut bytecode, encoding);
                    }

                    // some events end without <END marker.
                    TextScript::put_varint(OpCode::_END as i32, &mut bytecode);
                    break;
                }
                b'<' => {
                    allow_next_event = false;
                    if char_buf.len() > 2 {
                        if let Some(&c) = char_buf.last() {
                            if c == b'\n' {
                                let _ = char_buf.pop();
                            }
                        }
                    }

                    if !char_buf.is_empty() {
                        TextScript::put_string(&mut char_buf, &mut bytecode, encoding);
                    }

                    iter.next();
                    let n = iter
                        .next_tuple::<(u8, u8, u8)>()
                        .map(|t| [t.0, t.1, t.2])
                        .ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?;

                    let code = String::from_utf8_lossy(&n);

                    TextScript::compile_code(code.as_ref(), strict, iter, &mut bytecode)?;
                }
                b'\r' => {
                    iter.next();
                }
                b'\n' => {
                    allow_next_event = true;
                    char_buf.push(chr);

                    iter.next();
                }
                _ => {
                    allow_next_event = false;
                    char_buf.push(chr);

                    iter.next();
                }
            }
        }

        Ok(bytecode)
    }

    fn put_string(buffer: &mut Vec<u8>, out: &mut Vec<u8>, encoding: TextScriptEncoding) {
        if buffer.len() == 0 {
            return;
        }

        let mut cursor: Cursor<&Vec<u8>> = Cursor::new(buffer);
        let mut tmp_buf = Vec::new();
        let mut remaining = buffer.len() as u32;
        let mut chars = 0;

        while remaining > 0 {
            let (consumed, chr) = match encoding {
                TextScriptEncoding::UTF8 => read_cur_wtf8(&mut cursor, remaining),
                TextScriptEncoding::ShiftJIS => read_cur_shift_jis(&mut cursor, remaining),
            };

            remaining -= consumed;
            chars += 1;

            TextScript::put_varint(chr as i32, &mut tmp_buf);
        }

        buffer.clear();

        TextScript::put_varint(OpCode::_STR as i32, out);
        TextScript::put_varint(chars, out);
        out.append(&mut tmp_buf);
    }

    fn put_varint(val: i32, out: &mut Vec<u8>) {
        let mut x = ((val as u32) >> 31) ^ ((val as u32) << 1);

        loop {
            let mut n = (x & 0x7f) as u8;
            x >>= 7;

            if x != 0 {
                n |= 0x80;
            }

            out.push(n);

            if x == 0 {
                break;
            }
        }
    }

    #[allow(unused)]
    fn read_varint<I: Iterator<Item = u8>>(iter: &mut I) -> GameResult<i32> {
        let mut result = 0u32;

        for o in 0..5 {
            let n = iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?;
            result |= (n as u32 & 0x7f) << (o * 7);

            if n & 0x80 == 0 {
                break;
            }
        }

        Ok(((result << 31) ^ (result >> 1)) as i32)
    }

    fn compile_code<I: Iterator<Item = u8>>(
        code: &str,
        strict: bool,
        iter: &mut Peekable<I>,
        out: &mut Vec<u8>,
    ) -> GameResult {
        let instr = OpCode::from_str(code).map_err(|_| ParseError(format!("Unknown opcode: {}", code)))?;

        match instr {
            // Zero operand codes
            OpCode::AEp
            | OpCode::CAT
            | OpCode::CIL
            | OpCode::CLO
            | OpCode::CLR
            | OpCode::CPS
            | OpCode::CRE
            | OpCode::CSS
            | OpCode::END
            | OpCode::ESC
            | OpCode::FLA
            | OpCode::FMU
            | OpCode::FRE
            | OpCode::HMC
            | OpCode::INI
            | OpCode::KEY
            | OpCode::LDP
            | OpCode::MLP
            | OpCode::MM0
            | OpCode::MNA
            | OpCode::MS2
            | OpCode::MS3
            | OpCode::MSG
            | OpCode::NOD
            | OpCode::PRI
            | OpCode::RMU
            | OpCode::SAT
            | OpCode::SLP
            | OpCode::SMC
            | OpCode::SPS
            | OpCode::STC
            | OpCode::SVP
            | OpCode::TUR
            | OpCode::WAS
            | OpCode::ZAM
            | OpCode::HM2
            | OpCode::POP
            | OpCode::KE2
            | OpCode::FR2 => {
                TextScript::put_varint(instr as i32, out);
            }
            // One operand codes
            OpCode::BOA
            | OpCode::BSL
            | OpCode::FOB
            | OpCode::FOM
            | OpCode::QUA
            | OpCode::UNI
            | OpCode::MYB
            | OpCode::MYD
            | OpCode::FAI
            | OpCode::FAO
            | OpCode::WAI
            | OpCode::FAC
            | OpCode::GIT
            | OpCode::NUM
            | OpCode::DNA
            | OpCode::DNP
            | OpCode::FLm
            | OpCode::FLp
            | OpCode::MPp
            | OpCode::SKm
            | OpCode::SKp
            | OpCode::EQp
            | OpCode::EQm
            | OpCode::MLp
            | OpCode::ITp
            | OpCode::ITm
            | OpCode::AMm
            | OpCode::UNJ
            | OpCode::MPJ
            | OpCode::YNJ
            | OpCode::EVE
            | OpCode::XX1
            | OpCode::SIL
            | OpCode::LIp
            | OpCode::SOU
            | OpCode::CMU
            | OpCode::SSS
            | OpCode::ACH
            | OpCode::S2MV
            | OpCode::PSH => {
                let operand = TextScript::read_number(iter)?;
                TextScript::put_varint(instr as i32, out);
                TextScript::put_varint(operand as i32, out);
            }
            // Two operand codes
            OpCode::FON
            | OpCode::MOV
            | OpCode::AMp
            | OpCode::NCJ
            | OpCode::ECJ
            | OpCode::FLJ
            | OpCode::ITJ
            | OpCode::SKJ
            | OpCode::AMJ
            | OpCode::SMP
            | OpCode::PSp
            | OpCode::IpN
            | OpCode::FFm => {
                let operand_a = TextScript::read_number(iter)?;
                if strict {
                    TextScript::expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?;
                }
                let operand_b = TextScript::read_number(iter)?;

                TextScript::put_varint(instr as i32, out);
                TextScript::put_varint(operand_a as i32, out);
                TextScript::put_varint(operand_b as i32, out);
            }
            // Three operand codes
            OpCode::ANP | OpCode::CNP | OpCode::INP | OpCode::TAM | OpCode::CMP | OpCode::INJ => {
                let operand_a = TextScript::read_number(iter)?;
                if strict {
                    TextScript::expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?;
                }
                let operand_b = TextScript::read_number(iter)?;
                if strict {
                    TextScript::expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?;
                }
                let operand_c = TextScript::read_number(iter)?;

                TextScript::put_varint(instr as i32, out);
                TextScript::put_varint(operand_a as i32, out);
                TextScript::put_varint(operand_b as i32, out);
                TextScript::put_varint(operand_c as i32, out);
            }
            // Four operand codes
            OpCode::TRA | OpCode::MNP | OpCode::SNP => {
                let operand_a = TextScript::read_number(iter)?;
                if strict {
                    TextScript::expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?;
                }
                let operand_b = TextScript::read_number(iter)?;
                if strict {
                    TextScript::expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?;
                }
                let operand_c = TextScript::read_number(iter)?;
                if strict {
                    TextScript::expect_char(b':', iter)?;
                } else {
                    iter.next().ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))?;
                }
                let operand_d = TextScript::read_number(iter)?;

                TextScript::put_varint(instr as i32, out);
                TextScript::put_varint(operand_a as i32, out);
                TextScript::put_varint(operand_b as i32, out);
                TextScript::put_varint(operand_c as i32, out);
                TextScript::put_varint(operand_d as i32, out);
            }
            OpCode::_NOP | OpCode::_UNI | OpCode::_STR | OpCode::_END => {
                unreachable!()
            }
        }

        Ok(())
    }

    pub fn decompile_event(&self, id: u16) -> GameResult<String> {
        if let Some(bytecode) = self.event_map.get(&id) {
            let mut result = String::new();
            let mut cursor = Cursor::new(bytecode);

            while let Ok(op_num) = read_cur_varint(&mut cursor) {
                let op_maybe: Option<OpCode> = FromPrimitive::from_i32(op_num);

                if let Some(op) = op_maybe {
                    match op {
                        // Zero operand codes
                        OpCode::AEp
                        | OpCode::CAT
                        | OpCode::CIL
                        | OpCode::CLO
                        | OpCode::CLR
                        | OpCode::CPS
                        | OpCode::CRE
                        | OpCode::CSS
                        | OpCode::END
                        | OpCode::ESC
                        | OpCode::FLA
                        | OpCode::FMU
                        | OpCode::FRE
                        | OpCode::HMC
                        | OpCode::INI
                        | OpCode::KEY
                        | OpCode::LDP
                        | OpCode::MLP
                        | OpCode::MM0
                        | OpCode::MNA
                        | OpCode::MS2
                        | OpCode::MS3
                        | OpCode::MSG
                        | OpCode::NOD
                        | OpCode::PRI
                        | OpCode::RMU
                        | OpCode::SAT
                        | OpCode::SLP
                        | OpCode::SMC
                        | OpCode::SPS
                        | OpCode::STC
                        | OpCode::SVP
                        | OpCode::TUR
                        | OpCode::WAS
                        | OpCode::ZAM
                        | OpCode::HM2
                        | OpCode::POP
                        | OpCode::KE2
                        | OpCode::FR2 => {
                            result.push_str(format!("{:?}()\n", op).as_str());
                        }
                        // One operand codes
                        OpCode::BOA
                        | OpCode::BSL
                        | OpCode::FOB
                        | OpCode::FOM
                        | OpCode::QUA
                        | OpCode::UNI
                        | OpCode::MYB
                        | OpCode::MYD
                        | OpCode::FAI
                        | OpCode::FAO
                        | OpCode::WAI
                        | OpCode::FAC
                        | OpCode::GIT
                        | OpCode::NUM
                        | OpCode::DNA
                        | OpCode::DNP
                        | OpCode::FLm
                        | OpCode::FLp
                        | OpCode::MPp
                        | OpCode::SKm
                        | OpCode::SKp
                        | OpCode::EQp
                        | OpCode::EQm
                        | OpCode::MLp
                        | OpCode::ITp
                        | OpCode::ITm
                        | OpCode::AMm
                        | OpCode::UNJ
                        | OpCode::MPJ
                        | OpCode::YNJ
                        | OpCode::EVE
                        | OpCode::XX1
                        | OpCode::SIL
                        | OpCode::LIp
                        | OpCode::SOU
                        | OpCode::CMU
                        | OpCode::SSS
                        | OpCode::ACH
                        | OpCode::S2MV
                        | OpCode::PSH => {
                            let par_a = read_cur_varint(&mut cursor)?;

                            result.push_str(format!("{:?}({})\n", op, par_a).as_str());
                        }
                        // Two operand codes
                        OpCode::FON
                        | OpCode::MOV
                        | OpCode::AMp
                        | OpCode::NCJ
                        | OpCode::ECJ
                        | OpCode::FLJ
                        | OpCode::ITJ
                        | OpCode::SKJ
                        | OpCode::AMJ
                        | OpCode::SMP
                        | OpCode::PSp
                        | OpCode::IpN
                        | OpCode::FFm => {
                            let par_a = read_cur_varint(&mut cursor)?;
                            let par_b = read_cur_varint(&mut cursor)?;

                            result.push_str(format!("{:?}({}, {})\n", op, par_a, par_b).as_str());
                        }
                        // Three operand codes
                        OpCode::ANP | OpCode::CNP | OpCode::INP | OpCode::TAM | OpCode::CMP | OpCode::INJ => {
                            let par_a = read_cur_varint(&mut cursor)?;
                            let par_b = read_cur_varint(&mut cursor)?;
                            let par_c = read_cur_varint(&mut cursor)?;

                            result.push_str(format!("{:?}({}, {}, {})\n", op, par_a, par_b, par_c).as_str());
                        }
                        // Four operand codes
                        OpCode::TRA | OpCode::MNP | OpCode::SNP => {
                            let par_a = read_cur_varint(&mut cursor)?;
                            let par_b = read_cur_varint(&mut cursor)?;
                            let par_c = read_cur_varint(&mut cursor)?;
                            let par_d = read_cur_varint(&mut cursor)?;

                            result.push_str(format!("{:?}({}, {}, {}, {})\n", op, par_a, par_b, par_c, par_d).as_str());
                        }
                        OpCode::_STR => {
                            let len = read_cur_varint(&mut cursor)?;

                            result.push_str(format!("%string(len = {}, value = \"", len).as_str());
                            for _ in 0..len {
                                let chr = std::char::from_u32(read_cur_varint(&mut cursor)? as u32).unwrap_or('?');
                                match chr {
                                    '\n' => {
                                        result.push_str("\\n");
                                    }
                                    '\r' => {
                                        result.push_str("\\r");
                                    }
                                    '\t' => {
                                        result.push_str("\\t");
                                    }
                                    '\u{0000}'..='\u{001f}' | '\u{0080}'..='\u{ffff}' => {
                                        result.push_str(chr.escape_unicode().to_string().as_str());
                                    }
                                    _ => {
                                        result.push(chr);
                                    }
                                }
                            }
                            result.push_str("\")\n");
                        }
                        OpCode::_NOP => result.push_str("%no_op()\n"),
                        OpCode::_UNI => result.push_str("%unimplemented()\n"),
                        OpCode::_END => result.push_str("%end_marker()\n"),
                    }
                } else {
                    break;
                }
            }

            Ok(result)
        } else {
            Err(InvalidValue("Unknown script.".to_string()))
        }
    }

    fn expect_char<I: Iterator<Item = u8>>(expect: u8, iter: &mut I) -> GameResult {
        let res = iter.next();

        match res {
            Some(n) if n == expect => Ok(()),
            Some(n) => Err(ParseError(format!("Expected {}, found {}", expect as char, n as char))),
            None => Err(ParseError(str!("Script unexpectedly ended."))),
        }
    }

    fn skip_until<I: Iterator<Item = u8>>(expect: u8, iter: &mut Peekable<I>) -> GameResult {
        while let Some(&chr) = iter.peek() {
            if chr == expect {
                return Ok(());
            } else {
                iter.next();
            }
        }

        Err(ParseError(str!("Script unexpectedly ended.")))
    }

    /// Reads a 4 digit TSC formatted number from iterator.
    /// Intentionally does no '0'..'9' range checking, since it was often exploited by modders.
    fn read_number<I: Iterator<Item = u8>>(iter: &mut Peekable<I>) -> GameResult<i32> {
        Some(0)
            .and_then(|result| iter.next().map(|v| result + 1000 * v.wrapping_sub(b'0') as i32))
            .and_then(|result| iter.next().map(|v| result + 100 * v.wrapping_sub(b'0') as i32))
            .and_then(|result| iter.next().map(|v| result + 10 * v.wrapping_sub(b'0') as i32))
            .and_then(|result| iter.next().map(|v| result + v.wrapping_sub(b'0') as i32))
            .ok_or_else(|| ParseError(str!("Script unexpectedly ended.")))
    }

    pub fn has_event(&self, id: u16) -> bool {
        self.event_map.contains_key(&id)
    }
}

#[test]
fn test_varint() {
    for n in -4000..=4000 {
        let mut out = Vec::new();
        TextScript::put_varint(n, &mut out);

        let result = TextScript::read_varint(&mut out.iter().copied()).unwrap();
        assert_eq!(result, n);
        let mut cur = Cursor::new(&out);
        let result = read_cur_varint(&mut cur).unwrap();
        assert_eq!(result, n);
    }
}
