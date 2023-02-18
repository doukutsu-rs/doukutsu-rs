use discord_rich_presence::{
    activity::{Activity, Assets, Button},
    DiscordIpc, DiscordIpcClient,
};

use crate::framework::error::{GameError, GameResult};
use crate::game::{player::Player, stage::StageData};

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
        }
    }

    pub fn start(&mut self) -> GameResult {
        log::info!("Starting Discord RPC client...");

        match self.client.connect() {
            Ok(_) => {
                self.ready = true;
                Ok(())
            }
            Err(e) => Err(GameError::DiscordRPCError(e.to_string())),
        }
    }

    fn update(&mut self) -> GameResult {
        if !self.enabled {
            return Ok(());
        }

        let (state, details) = match self.state {
            DiscordRPCState::Initializing => ("Initializing...".to_owned(), "Just started playing".to_owned()),
            DiscordRPCState::Idling => ("In the menus".to_owned(), "Idling".to_owned()),
            DiscordRPCState::InGame => {
                (format!("Currently in: {}", self.stage_name), format!("HP: {} / {}", self.life, self.max_life))
            }
            DiscordRPCState::Jukebox => ("In the menus".to_owned(), "Listening to the soundtrack".to_owned()),
        };

        log::debug!("Updating Discord RPC state: {} - {}", state, details);

        let activity = Activity::new()
            .state(state.as_str())
            .details(details.as_str())
            .assets(Assets::new().large_image("drs"))
            .buttons(vec![Button::new("doukutsu-rs on GitHub", "https://github.com/doukutsu-rs/doukutsu-rs")]);

        match self.client.set_activity(activity) {
            Ok(_) => Ok(()),
            Err(e) => Err(GameError::DiscordRPCError(e.to_string())),
        }
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

    pub fn set_initializing(&mut self) -> GameResult {
        self.set_state(DiscordRPCState::Initializing)
    }

    pub fn set_idling(&mut self) -> GameResult {
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
