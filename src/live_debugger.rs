use imgui::{Condition, im_str, ImStr, ImString, Window};
use itertools::Itertools;

use crate::ggez::{Context, GameResult};
use crate::scene::game_scene::GameScene;
use crate::SharedGameState;

pub struct LiveDebugger {
    selected_item: i32,
    map_selector_visible: bool,
    hacks_visible: bool,
    stages: Vec<ImString>,
    error: Option<ImString>,
}

impl LiveDebugger {
    pub fn new() -> Self {
        Self {
            selected_item: -1,
            map_selector_visible: false,
            hacks_visible: false,
            stages: vec![],
            error: None,
        }
    }

    pub fn run_ingame(&mut self, game_scene: &mut GameScene, state: &mut SharedGameState, ctx: &mut Context, ui: &mut imgui::Ui) -> GameResult {
        Window::new(im_str!("Debugger"))
            .position([5.0, 5.0], Condition::FirstUseEver)
            .size([300.0, 120.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text(format!(
                    "Player position: ({:.1},{:.1})",
                    game_scene.player.x as f32 / 512.0,
                    game_scene.player.y as f32 / 512.0,
                ));

                ui.text(format!(
                    "Player velocity: ({:.1},{:.1})",
                    game_scene.player.vel_x as f32 / 512.0,
                    game_scene.player.vel_y as f32 / 512.0,
                ));

                ui.text(format!(
                    "Booster fuel: ({})", game_scene.player.booster_fuel
                ));

                if ui.button(im_str!("Map Selector"), [0.0, 0.0]) {
                    self.map_selector_visible = true;
                }

                if ui.button(im_str!("Hacks"), [0.0, 0.0]) {
                    self.hacks_visible = true;
                }
            });

        if self.error.is_some() {
            Window::new(im_str!("Error!"))
                .resizable(false)
                .collapsible(false)
                .position([((state.screen_size.0 - 300.0) / 2.0).floor(), ((state.screen_size.1 - 100.0) / 2.0).floor()], Condition::Appearing)
                .size([300.0, 100.0], Condition::Appearing)
                .build(ui, || {
                    ui.push_item_width(-1.0);
                    ui.text(self.error.as_ref().unwrap());

                    if ui.button(im_str!("OK"), [0.0, 0.0]) {
                        self.error = None;
                    }
                });
        }

        if self.map_selector_visible {
            Window::new(im_str!("Map selector"))
                .resizable(false)
                .position([5.0, 35.0], Condition::FirstUseEver)
                .size([240.0, 280.0], Condition::FirstUseEver)
                .build(ui, || {
                    if self.stages.is_empty() {
                        for s in state.stages.iter() {
                            self.stages.push(ImString::new(s.name.to_owned()));
                        }

                        self.selected_item = match state.stages.iter().find_position(|s| s.name == game_scene.stage.data.name) {
                            Some((pos, _)) => { pos as i32 }
                            _ => { -1 }
                        };
                    }
                    let stages: Vec<&ImStr> = self.stages.iter().map(|e| e.as_ref()).collect();

                    ui.push_item_width(-1.0);
                    ui.list_box(im_str!(""), &mut self.selected_item, &stages, 10);

                    if ui.button(im_str!("Load"), [0.0, 0.0]) {
                        match GameScene::new(state, ctx, self.selected_item as usize) {
                            Ok(mut scene) => {
                                scene.player.x = (scene.stage.map.width / 2 * 16 * 0x200) as isize;
                                scene.player.y = (scene.stage.map.height / 2 * 16 * 0x200) as isize;
                                state.next_scene = Some(Box::new(scene));
                            }
                            Err(e) => {
                                log::error!("Error loading map: {:?}", e);
                                self.error = Some(ImString::new(e.to_string()));
                            }
                        }
                    }
                });
        }

        Ok(())
    }
}
