use crate::common::{Condition, ControlFlags, Direction, FadeState};
use crate::player::{ControlMode, TargetPlayer};
use crate::scripting::tsc::text_script::{
    IllustrationState, Scripts, TextScriptExecutionState, TextScriptFlags, TextScriptLine,
};
use crate::stage::Stage;
use crate::{GameResult, ScriptMode};
use crate::components::number_popup::NumberPopup;
use crate::inventory::Inventory;
use crate::npc::NPC;

#[derive(Clone, bincode::Decode, bincode::Encode)]
pub struct ServerInfo {
    pub motd: String,
    // online/max
    pub players: (u16, u16),
}

#[derive(Clone, bincode::Decode, bincode::Encode)]
pub struct PlayerInfo {
    pub name: String,
    pub public_key: [u8; 32],
    pub challenge_signature: [u8; 64],
}

#[derive(Clone, bincode::Decode, bincode::Encode)]
pub struct HelloData {
    pub challenge: [u8; 32],
}

#[derive(Clone, bincode::Decode, bincode::Encode)]
pub struct StageData {
    pub stage_id: u32,
    pub stage: Stage,
    pub player_pos: (i32, i32),
}

#[derive(Clone, bincode::Decode, bincode::Encode)]
pub struct PlayerMove {
    pub target: TargetPlayer,
    pub x: i32,
    pub y: i32,
    pub vel_x: i32,
    pub vel_y: i32,
    pub state: u16,
    pub old_state: u16,
    pub trigger: u16,
    pub direction: Direction,
    pub cond: Condition,
}

#[derive(Clone, bincode::Decode, bincode::Encode)]
pub struct TextScriptData {
    pub state: TextScriptExecutionState,
    pub stack: Vec<TextScriptExecutionState>,
    pub flags: TextScriptFlags,
    pub mode: ScriptMode,
    pub executor_player: TargetPlayer,
    pub strict_mode: bool,
    pub suspend: bool,
    pub reset_invincibility: bool,
    pub numbers: [u16; 4],
    pub face: u16,
    pub item: u16,
    pub current_line: TextScriptLine,
    pub line_1: Vec<char>,
    pub line_2: Vec<char>,
    pub line_3: Vec<char>,
    pub current_illustration: Option<String>,
    pub illustration_state: IllustrationState,
    pub prev_char: char,
    pub fade_state: FadeState,
    pub current_song: u32,
    pub prev_song: u32,
}


#[derive(Clone, bincode::Decode, bincode::Encode)]
pub struct PlayerData {
    pub target: TargetPlayer,
    pub life: u16,
    pub max_life: u16,
    pub control_mode: ControlMode,
    pub question: bool,
    pub popup: NumberPopup,
    pub shock_counter: u8,
    pub xp_counter: u8,
    pub current_weapon: u8,
    pub stars: u8,
    pub damage: u16,
    pub air_counter: u16,
    pub air: u16,
}

#[derive(Clone, bincode::Decode, bincode::Encode)]
pub enum DRSPacket {
    KeepAlive,
    Hello(HelloData),
    Kicked(String),
    ChatMessage(String),
    ServerInfoRequest,
    ServerInfoResponse(ServerInfo),
    Connect(PlayerInfo),
    ConnectResponse(TargetPlayer),
    Move(PlayerMove),
    SyncControlFlags(ControlFlags),
    SyncFlags([u8; 1000]),
    SyncStageData(StageData),
    SyncTSCScripts(Scripts),
    SyncTSC(TextScriptData),
    SyncPlayer(PlayerData),
    SyncInventory(TargetPlayer, Inventory),
    SyncNPC(NPC),
}

impl DRSPacket {
    pub fn decode(data: &[u8]) -> GameResult<DRSPacket> {
        let (packet, _) = bincode::decode_from_slice(data, bincode::config::standard())?;
        Ok(packet)
    }

    pub fn encode(&self, data: &mut [u8]) -> GameResult {
        bincode::encode_into_slice(self, data, bincode::config::standard())?;
        Ok(())
    }

    pub fn encode_to_vec(&self) -> Vec<u8> {
        bincode::encode_to_vec(self, bincode::config::standard()).unwrap()
    }
}
