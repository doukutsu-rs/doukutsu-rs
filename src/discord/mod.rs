use std::sync::Mutex;

use discord_rich_presence::{
    activity::{Activity, Assets, Button},
    DiscordIpc, DiscordIpcClient,
};

use crate::framework::error::{GameError, GameResult};
use crate::game::{player::Player, shared_game_state::GameDifficulty, stage::StageData};

pub enum DiscordRPCState {
    Initializing,
    Idling,
    InGame,
    Jukebox,
}

pub struct DiscordRPC {
    pub enabled: bool,
    pub ready: bool,

    client: DiscordIpcClient,
    state: DiscordRPCState,
    life: u16,
    max_life: u16,
    stage_name: String,
    difficulty: Option<GameDifficulty>,

    can_update: Mutex<bool>,
}

impl DiscordRPC {
    pub fn new(app_id: &str) -> Self {
        Self {
            enabled: false,
            ready: false,

            client: DiscordIpcClient::new(app_id).unwrap(),
            state: DiscordRPCState::Idling,
            life: 0,
            max_life: 0,
            stage_name: String::new(),
            difficulty: None,

            can_update: Mutex::new(true),
        }
    }

    pub fn start(&mut self) -> GameResult {
        log::info!("Starting Discord RPC client...");

        let mut can_update = self.can_update.lock().unwrap();
        *can_update = false;

        match self.client.connect() {
            Ok(_) => {
                self.ready = true;
                *can_update = true;

                Ok(())
            }
            Err(e) => Err(GameError::DiscordRPCError(e.to_string())),
        }
    }

    fn update(&mut self) -> GameResult {
        if !self.enabled || !self.ready {
            return Ok(());
        }

        let mut can_update = self.can_update.lock().unwrap();

        if !*can_update {
            return Ok(());
        }

        *can_update = false;

        let (state, details) = match self.state {
            DiscordRPCState::Initializing => ("Initializing...".to_owned(), "Just started playing".to_owned()),
            DiscordRPCState::Idling => ("In the menus".to_owned(), "Idling".to_owned()),
            DiscordRPCState::InGame => {
                (format!("Currently in: {}", self.stage_name), format!("HP: {} / {}", self.life, self.max_life))
            }
            DiscordRPCState::Jukebox => ("In the menus".to_owned(), "Listening to the soundtrack".to_owned()),
        };

        log::debug!("Updating Discord RPC state: {} - {}", state, details);

        let mut activity_assets = Assets::new().large_image("drs");

        if self.difficulty.is_some() {
            let difficulty = self.difficulty.unwrap();

            let asset_name = match difficulty {
                GameDifficulty::Easy => "deasy",
                GameDifficulty::Normal => "dnormal",
                GameDifficulty::Hard => "dhard",
            };

            let asset_label = match difficulty {
                GameDifficulty::Easy => "Easy",
                GameDifficulty::Normal => "Normal",
                GameDifficulty::Hard => "Hard",
            };

            activity_assets = activity_assets.small_image(asset_name).small_text(asset_label);
        }

        let activity = Activity::new()
            .state(state.as_str())
            .details(details.as_str())
            .assets(activity_assets)
            .buttons(vec![Button::new("doukutsu-rs on GitHub", "https://github.com/doukutsu-rs/doukutsu-rs")]);

        match self.client.set_activity(activity) {
            Ok(()) => {
                *can_update = true;
                log::debug!("Discord RPC state updated successfully");
            }
            Err(e) => log::error!("Failed to update Discord RPC state: {}", e),
        };

        Ok(()) // whatever
    }

    pub fn update_stage(&mut self, stage: &StageData) -> GameResult {
        self.stage_name = stage.name.clone();
        self.update()
    }

    pub fn update_hp(&mut self, player: &Player) -> GameResult {
        self.life = player.life;
        self.max_life = player.max_life;
        self.update()
    }

    pub fn update_difficulty(&mut self, difficulty: GameDifficulty) -> GameResult {
        self.difficulty = Some(difficulty);
        self.update()
    }

    pub fn set_initializing(&mut self) -> GameResult {
        self.set_state(DiscordRPCState::Initializing)
    }

    pub fn set_idling(&mut self) -> GameResult {
        self.difficulty = None;
        self.set_state(DiscordRPCState::Idling)
    }

    pub fn set_in_game(&mut self) -> GameResult {
        self.set_state(DiscordRPCState::InGame)
    }

    pub fn set_in_jukebox(&mut self) -> GameResult {
        self.set_state(DiscordRPCState::Jukebox)
    }

    pub fn set_state(&mut self, state: DiscordRPCState) -> GameResult {
        self.state = state;
        self.update()
    }

    pub fn clear(&mut self) -> GameResult {
        let _ = self.client.clear_activity();
        Ok(())
    }

    pub fn dispose(&mut self) {
        let _ = self.client.close();
    }
}
