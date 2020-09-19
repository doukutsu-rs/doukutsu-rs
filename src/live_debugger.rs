use imgui::{CollapsingHeader, Condition, im_str, ImStr, ImString, Window};
use itertools::Itertools;

use crate::ggez::{Context, GameResult};
use crate::scene::game_scene::GameScene;
use crate::SharedGameState;

pub struct LiveDebugger {
    map_selector_visible: bool,
    events_visible: bool,
    hacks_visible: bool,
    flags_visible: bool,
    last_stage_id: usize,
    stages: Vec<ImString>,
    selected_stage: i32,
    events: Vec<ImString>,
    event_ids: Vec<u16>,
    selected_event: i32,
    error: Option<ImString>,
}

impl LiveDebugger {
    pub fn new() -> Self {
        Self {
            map_selector_visible: false,
            events_visible: false,
            hacks_visible: false,
            flags_visible: false,
            last_stage_id: usize::MAX,
            stages: Vec::new(),
            selected_stage: -1,
            events: Vec::new(),
            event_ids: Vec::new(),
            selected_event: -1,
            error: None,
        }
    }

    pub fn run_ingame(&mut self, game_scene: &mut GameScene, state: &mut SharedGameState, ctx: &mut Context, ui: &mut imgui::Ui) -> GameResult {
        if self.last_stage_id != game_scene.stage_id {
            self.last_stage_id = game_scene.stage_id;
            self.events.clear();
            self.selected_event = -1;
        }

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
                    self.map_selector_visible = !self.map_selector_visible;
                }

                ui.same_line(0.0);
                if ui.button(im_str!("Events"), [0.0, 0.0]) {
                    self.events_visible = !self.events_visible;
                }

                ui.same_line(0.0);
                if ui.button(im_str!("Hacks"), [0.0, 0.0]) {
                    self.hacks_visible = !self.hacks_visible;
                }

                ui.same_line(0.0);
                if ui.button(im_str!("Flags"), [0.0, 0.0]) {
                    self.flags_visible = !self.flags_visible;
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
                    ui.text_wrapped(self.error.as_ref().unwrap());

                    if ui.button(im_str!("OK"), [0.0, 0.0]) {
                        self.error = None;
                    }
                });
        }

        if self.map_selector_visible {
            Window::new(im_str!("Map selector"))
                .resizable(false)
                .position([80.0, 80.0], Condition::FirstUseEver)
                .size([240.0, 280.0], Condition::FirstUseEver)
                .build(ui, || {
                    if self.stages.is_empty() {
                        for s in state.stages.iter() {
                            self.stages.push(ImString::new(s.name.to_owned()));
                        }

                        self.selected_stage = match state.stages.iter().find_position(|s| s.name == game_scene.stage.data.name) {
                            Some((pos, _)) => { pos as i32 }
                            _ => { -1 }
                        };
                    }
                    let stages: Vec<&ImStr> = self.stages.iter().map(|e| e.as_ref()).collect();

                    ui.push_item_width(-1.0);
                    ui.list_box(im_str!(""), &mut self.selected_stage, &stages, 10);

                    if ui.button(im_str!("Load"), [0.0, 0.0]) {
                        match GameScene::new(state, ctx, self.selected_stage as usize) {
                            Ok(mut scene) => {
                                scene.inventory = game_scene.inventory.clone();
                                scene.player = game_scene.player.clone();
                                scene.player.x = (scene.stage.map.width / 2 * 16 * 0x200) as isize;
                                scene.player.y = (scene.stage.map.height / 2 * 16 * 0x200) as isize;

                                if scene.player.life == 0 {
                                    scene.player.life = scene.player.max_life;
                                }

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

        if self.events_visible {
            Window::new(im_str!("Events"))
                .resizable(false)
                .position([80.0, 80.0], Condition::FirstUseEver)
                .size([280.0, 300.0], Condition::FirstUseEver)
                .build(ui, || {
                    if self.events.is_empty() {
                        self.event_ids.clear();

                        let vm = &state.textscript_vm;
                        for event in vm.scripts.global_script.get_event_ids() {
                            self.events.push(ImString::new(format!("Global: #{:04}", event)));
                            self.event_ids.push(event);
                        }

                        for event in vm.scripts.scene_script.get_event_ids() {
                            self.events.push(ImString::new(format!("Scene: #{:04}", event)));
                            self.event_ids.push(event);
                        }
                    }
                    let events: Vec<&ImStr> = self.events.iter().map(|e| e.as_ref()).collect();

                    ui.text_wrapped(&ImString::new(format!("Execution state: {:?}", state.textscript_vm.state)));

                    ui.push_item_width(-1.0);
                    ui.list_box(im_str!(""), &mut self.selected_event, &events, 10);

                    if ui.button(im_str!("Execute"), [0.0, 0.0]) {
                        assert_eq!(self.event_ids.len(), self.events.len());

                        if let Some(&event_num) = self.event_ids.get(self.selected_event as usize) {
                            state.textscript_vm.start_script(event_num);
                        }
                    }
                });
        }


        if self.flags_visible {
            Window::new(im_str!("Flags"))
                .position([80.0, 80.0], Condition::FirstUseEver)
                .size([280.0, 300.0], Condition::FirstUseEver)
                .build(ui, || {
                    if CollapsingHeader::new(im_str!("Control flags")).default_open(true).build(&ui)
                    {
                        ui.checkbox_flags(im_str!("Flag 0x01"), &mut state.control_flags.0, 1);
                        ui.checkbox_flags(im_str!("Control enabled"), &mut state.control_flags.0, 2);
                        ui.checkbox_flags(im_str!("Interactions disabled"), &mut state.control_flags.0, 4);
                    }
                });
        }

        Ok(())
    }
}
