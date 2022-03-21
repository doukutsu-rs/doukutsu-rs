use imgui::{InputInt, InputText, InputTextFlags, Ui, WindowFlags};

use crate::common::Color;
use crate::components::background::Background;
use crate::frame::Frame;
use crate::framework::context::Context;
use crate::framework::error::{GameError, GameResult};
use crate::framework::ui::Components;
use crate::map::Map;
use crate::netplay::client::{Client, ServerInfoFuture};
use crate::netplay::future::RSFuture;
use crate::netplay::protocol::ServerInfo;
use crate::scene::title_scene::TitleScene;
use crate::shared_game_state::{SharedGameState, TileSize};
use crate::stage::{BackgroundType, NpcType, Stage, StageData, StageTexturePaths, Tileset};
use crate::Scene;

pub struct NetplayScene {
    background: Background,
    stage: Stage,
    frame: Frame,
    textures: StageTexturePaths,
    player_name: String,
    ip: String,
    client: Option<Client>,
    info_future: Option<ServerInfoFuture>,
}

impl NetplayScene {
    pub fn new() -> NetplayScene {
        let fake_stage = Stage {
            map: Map { width: 0, height: 0, tiles: vec![], attrib: [0; 0x100], tile_size: TileSize::Tile16x16 },
            data: StageData {
                name: "".to_string(),
                map: "".to_string(),
                boss_no: 0,
                tileset: Tileset { name: "0".to_string() },
                pxpack_data: None,
                background: crate::stage::Background::new("bkMoon"),
                background_type: BackgroundType::Outside,
                background_color: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
                npc1: NpcType::new("0"),
                npc2: NpcType::new("0"),
            },
        };
        let mut textures = StageTexturePaths::new();
        textures.update(&fake_stage);

        NetplayScene {
            background: Background::new(),
            stage: fake_stage,
            frame: Frame::new(),
            textures,
            player_name: "Quote".to_owned(),
            ip: "127.0.0.1:21075".to_owned(),
            client: None,
            info_future: None,
        }
    }
}

impl Scene for NetplayScene {
    fn init(&mut self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn tick(&mut self, _state: &mut SharedGameState, _ctx: &mut Context) -> GameResult {
        self.background.tick()?;

        let mut clear = false;
        if let Some(future) = &mut self.info_future {
            if let Some(info) = future.poll() {
                match info.as_ref() {
                    Ok(info) => {
                        log::info!("Got server info: {} {}/{}", info.motd, info.players.0, info.players.1);
                    }
                    Err(e) => {
                        log::error!("Failed to get server info: {}", e);
                    }
                }
                clear = true;
            }
        }

        if clear {
            self.info_future = None;
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        self.background.draw(state, ctx, &self.frame, &self.textures, &self.stage)?;

        Ok(())
    }

    fn imgui_draw(
        &mut self,
        _game_ui: &mut Components,
        state: &mut SharedGameState,
        _ctx: &mut Context,
        frame: &mut Ui,
    ) -> GameResult {
        imgui::Window::new("Netplay")
            .flags(WindowFlags::NO_TITLE_BAR | WindowFlags::NO_MOVE | WindowFlags::NO_RESIZE)
            .position_pivot([0.5, 0.5])
            .always_auto_resize(true)
            .build(frame, || {
                InputText::new(frame, "Player name", &mut self.player_name).build();
                InputText::new(frame, "Address", &mut self.ip).build();

                {
                    let _t = frame.begin_disabled(self.client.is_some());
                    if frame.button("Connect") {
                        match Client::new(&self.ip) {
                            Ok(mut c) => {
                                self.info_future = Some(c.get_server_info());
                                self.client = Some(c);
                            }
                            Err(e) => {
                                log::error!("Failed to create client: {}", e);
                            }
                        }
                    }
                }

                frame.same_line();

                if frame.button("Back to title") {
                    state.next_scene = Some(Box::new(TitleScene::new()));
                }
            });

        Ok(())
    }
}
