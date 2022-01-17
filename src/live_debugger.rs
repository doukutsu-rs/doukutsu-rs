use imgui::{ CollapsingHeader, Condition, ImStr, ImString, Slider, Window};
use itertools::Itertools;

use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::scene::game_scene::GameScene;
use crate::scripting::tsc::text_script::TextScriptExecutionState;
use crate::shared_game_state::SharedGameState;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum ScriptType {
    Scene,
    Global,
    Inventory,
    StageSelect,
}

pub struct LiveDebugger {
    map_selector_visible: bool,
    events_visible: bool,
    flags_visible: bool,
    npc_inspector_visible: bool,
    last_stage_id: usize,
    stages: Vec<ImString>,
    selected_stage: i32,
    events: Vec<ImString>,
    event_ids: Vec<(ScriptType, u16)>,
    selected_event: i32,
    text_windows: Vec<(u32, ImString, ImString)>,
    error: Option<ImString>,
}

impl LiveDebugger {
    pub fn new() -> Self {
        Self {
            map_selector_visible: false,
            events_visible: false,
            flags_visible: false,
            npc_inspector_visible: false,
            last_stage_id: usize::MAX,
            stages: Vec::new(),
            selected_stage: -1,
            events: Vec::new(),
            event_ids: Vec::new(),
            selected_event: -1,
            text_windows: Vec::new(),
            error: None,
        }
    }

    pub fn run_ingame(
        &mut self,
        game_scene: &mut GameScene,
        state: &mut SharedGameState,
        ctx: &mut Context,
        ui: &mut imgui::Ui,
    ) -> GameResult {
        if self.last_stage_id != game_scene.stage_id {
            self.last_stage_id = game_scene.stage_id;
            self.events.clear();
            self.selected_event = -1;
        }

        if !state.debugger {
            return Ok(());
        }

        Window::new("Debugger")
            .resizable(false)
            .collapsed(true, Condition::FirstUseEver)
            .position([5.0, 5.0], Condition::FirstUseEver)
            .size([400.0, 190.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text(format!(
                    "Player position: ({:.1},{:.1}), velocity: ({:.1},{:.1})",
                    game_scene.player1.x as f32 / 512.0,
                    game_scene.player1.y as f32 / 512.0,
                    game_scene.player1.vel_x as f32 / 512.0,
                    game_scene.player1.vel_y as f32 / 512.0,
                ));

                ui.text(format!(
                    "frame: ({:.1},{:.1} -> {:.1},{:.1} / {})",
                    game_scene.frame.x as f32 / 512.0,
                    game_scene.frame.y as f32 / 512.0,
                    game_scene.frame.target_x as f32 / 512.0,
                    game_scene.frame.target_y as f32 / 512.0,
                    game_scene.frame.wait
                ));

                ui.text(format!(
                    "NPC Count: {}/{}/{} Booster fuel: {}",
                    game_scene.npc_list.iter_alive().count(),
                    game_scene.npc_list.current_capacity(),
                    game_scene.npc_list.max_capacity(),
                    game_scene.player1.booster_fuel
                ));

                ui.text(format!("Game speed ({:.1} TPS):", state.current_tps()));
                let mut speed = state.settings.speed;
                Slider::new("", 0.1, 3.0).build(ui, &mut speed);
                ui.same_line();
                if ui.button("Reset") {
                    speed = 1.0
                }

                #[allow(clippy::float_cmp)]
                if state.settings.speed != speed {
                    state.set_speed(speed);
                }

                if ui.button("Maps") {
                    self.map_selector_visible = !self.map_selector_visible;
                }

                ui.same_line();
                if ui.button("TSC Scripts") {
                    self.events_visible = !self.events_visible;
                }

                ui.same_line();
                if ui.button("Flags") {
                    self.flags_visible = !self.flags_visible;
                }

                #[cfg(feature = "scripting-lua")]
                {
                    ui.same_line();
                    if ui.button("Reload Lua Scripts") {
                        if let Err(err) = state.lua.reload_scripts(ctx) {
                            log::error!("Error reloading scripts: {:?}", err);
                            self.error = Some(ImString::new(err.to_string()));
                        }
                    }
                }

                if game_scene.player2.cond.alive() {
                    if ui.button("Drop Player 2") {
                        game_scene.drop_player2();
                    }
                } else if ui.button("Add Player 2") {
                    game_scene.add_player2();
                }
                ui.same_line();

                if ui.button("NPC Inspector") {
                    self.npc_inspector_visible = !self.npc_inspector_visible;
                }
            });

        if self.map_selector_visible {
            Window::new("Map selector")
                .resizable(false)
                .position([80.0, 80.0], Condition::Appearing)
                .size([240.0, 280.0], Condition::Appearing)
                .build(ui, || {
                    if self.stages.is_empty() {
                        for s in &state.stages {
                            self.stages.push(ImString::new(s.name.to_owned()));
                        }

                        self.selected_stage =
                            match state.stages.iter().find_position(|s| s.name == game_scene.stage.data.name) {
                                Some((pos, _)) => pos as i32,
                                _ => -1,
                            };
                    }
                    let stages: Vec<&ImStr> = self.stages.iter().map(|e| e.as_ref()).collect();

                    ui.push_item_width(-1.0);
                    ui.list_box("", &mut self.selected_stage, &stages, 10);

                    if ui.button("Load") {
                        match GameScene::new(state, ctx, self.selected_stage as usize) {
                            Ok(mut scene) => {
                                let tile_size = scene.stage.map.tile_size.as_int() * 0x200;
                                scene.inventory_player1 = game_scene.inventory_player1.clone();
                                scene.inventory_player2 = game_scene.inventory_player2.clone();

                                scene.player1 = game_scene.player1.clone();
                                scene.player1.x = scene.stage.map.width as i32 / 2 * tile_size;
                                scene.player1.y = scene.stage.map.height as i32 / 2 * tile_size;

                                if scene.player1.life == 0 {
                                    scene.player1.life = scene.player1.max_life;
                                }

                                scene.player2 = game_scene.player2.clone();
                                scene.player2.x = scene.stage.map.width as i32 / 2 * tile_size;
                                scene.player2.y = scene.stage.map.height as i32 / 2 * tile_size;

                                if scene.player2.life == 0 {
                                    scene.player2.life = scene.player1.max_life;
                                }

                                state.textscript_vm.suspend = true;
                                state.textscript_vm.state = TextScriptExecutionState::Running(94, 0);
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
            Window::new("TSC Scripts")
                .resizable(false)
                .position([80.0, 80.0], Condition::Appearing)
                .size([300.0, 320.0], Condition::Appearing)
                .build(ui, || {
                    if self.events.is_empty() {
                        self.event_ids.clear();

                        let scripts = state.textscript_vm.scripts.borrow();

                        for event in scripts.scene_script.get_event_ids() {
                            self.events.push(ImString::new(format!("Scene: #{:04}", event)));
                            self.event_ids.push((ScriptType::Scene, event));
                        }

                        for event in scripts.global_script.get_event_ids() {
                            self.events.push(ImString::new(format!("Global: #{:04}", event)));
                            self.event_ids.push((ScriptType::Global, event));
                        }

                        for event in scripts.inventory_script.get_event_ids() {
                            self.events.push(ImString::new(format!("Inventory: #{:04}", event)));
                            self.event_ids.push((ScriptType::Inventory, event));
                        }

                        for event in scripts.stage_select_script.get_event_ids() {
                            self.events.push(ImString::new(format!("Stage Select: #{:04}", event)));
                            self.event_ids.push((ScriptType::StageSelect, event));
                        }
                    }
                    let events: Vec<&ImStr> = self.events.iter().map(|e| e.as_ref()).collect();

                    ui.text_wrapped(&ImString::new(format!(
                        "TextScript execution state: {:?}",
                        state.textscript_vm.state
                    )));
                    ui.text_wrapped(&ImString::new(format!(
                        "CreditScript execution state: {:?}",
                        state.creditscript_vm.state
                    )));

                    ui.push_item_width(-1.0);
                    ui.list_box("", &mut self.selected_event, &events, 10);

                    if ui.button("Execute") {
                        assert_eq!(self.event_ids.len(), self.events.len());

                        if let Some((_, event_num)) = self.event_ids.get(self.selected_event as usize) {
                            state.control_flags.set_tick_world(true);
                            state.control_flags.set_interactions_disabled(true);
                            state.textscript_vm.start_script(*event_num);
                        }
                    }

                    ui.same_line();
                    if ui.button("Decompile") {
                        if let Some((stype, event_num)) = self.event_ids.get(self.selected_event as usize) {
                            let id = ((*stype as u32) << 16) | (*event_num as u32);
                            if !self.text_windows.iter().any(|(e, _, _)| *e == id) {
                                let scripts = state.textscript_vm.scripts.borrow();
                                let script = match stype {
                                    ScriptType::Scene => &scripts.scene_script,
                                    ScriptType::Global => &scripts.global_script,
                                    ScriptType::Inventory => &scripts.inventory_script,
                                    ScriptType::StageSelect => &scripts.stage_select_script,
                                };

                                match script.decompile_event(*event_num) {
                                    Ok(code) => {
                                        self.text_windows.push((
                                            id,
                                            ImString::new(format!("Decompiled event: #{:04}", *event_num)),
                                            ImString::new(code),
                                        ));
                                    }
                                    Err(e) => {
                                        self.error = Some(ImString::new(format!(
                                            "Error decompiling TextScript #{:04}: {}",
                                            *event_num, e
                                        )));
                                    }
                                }
                            }
                        }
                    }
                });
        }

        if self.flags_visible {
            Window::new("Flags")
                .position([80.0, 80.0], Condition::FirstUseEver)
                .size([280.0, 300.0], Condition::FirstUseEver)
                .build(ui, || {
                    if CollapsingHeader::new("Control flags").default_open(false).build(ui) {
                        ui.checkbox_flags("Tick world", &mut state.control_flags.0, 1);
                        ui.checkbox_flags("Control enabled", &mut state.control_flags.0, 2);
                        ui.checkbox_flags("Interactions disabled", &mut state.control_flags.0, 4);
                        ui.checkbox_flags("Credits running", &mut state.control_flags.0, 8);
                        ui.separator();
                        ui.checkbox_flags("[Internal] Windy level", &mut state.control_flags.0, 15);
                    }

                    if CollapsingHeader::new("Player condition flags").default_open(false).build(ui) {
                        cond_flags(ui, &mut game_scene.player1.cond);
                    }

                    if CollapsingHeader::new("Player equipment").default_open(false).build(ui) {
                        ui.checkbox_flags("Booster 0.8", &mut game_scene.player1.equip.0, 1);
                        ui.checkbox_flags("Map System", &mut game_scene.player1.equip.0, 2);
                        ui.checkbox_flags("Arms Barrier", &mut game_scene.player1.equip.0, 4);
                        ui.checkbox_flags("Turbocharge", &mut game_scene.player1.equip.0, 8);
                        ui.checkbox_flags("Air Tank", &mut game_scene.player1.equip.0, 16);
                        ui.checkbox_flags("Booster 2.0", &mut game_scene.player1.equip.0, 32);
                        ui.checkbox_flags("Mimiga Mask", &mut game_scene.player1.equip.0, 64);
                        ui.checkbox_flags("Whimsical Star", &mut game_scene.player1.equip.0, 128);
                        ui.checkbox_flags("Nikumaru Counter", &mut game_scene.player1.equip.0, 256);
                    }
                });
        }

        if self.npc_inspector_visible {
            Window::new("NPC Inspector")
                .position([80.0, 80.0], Condition::FirstUseEver)
                .size([280.0, 300.0], Condition::FirstUseEver)
                .scrollable(true)
                .always_vertical_scrollbar(true)
                .build(ui, || {
                    for npc in game_scene.npc_list.iter_alive() {
                        if CollapsingHeader::new(&ImString::from(format!("id={} type={}", npc.id, npc.npc_type)))
                            .default_open(false)
                            .build(ui)
                        {
                            let mut position = [npc.x as f32 / 512.0, npc.y as f32 / 512.0];
                            ui.input_float2("Position:", &mut position).build();

                            npc.x = (position[0] * 512.0) as i32;
                            npc.y = (position[1] * 512.0) as i32;

                            let content = &ImString::from(format!(
                                "\
                                    Velocity: ({:.1},{:.1})\n\
                                    Vel2/State2: ({:.1},{:.1} / {} {})\n\
                                    Animation: frame={}, counter={}\n\
                                    Action: num={}, counter={}, counter2={}\n\
                                    Health: {}, Experience drop: {}\n\
                                    Event ID: {}, Flag ID: {}\n\
                                    Parent: {}, Shock: {}, Size: {}",
                                npc.vel_x as f32 / 512.0,
                                npc.vel_y as f32 / 512.0,
                                npc.vel_x2 as f32 / 512.0,
                                npc.vel_y2 as f32 / 512.0,
                                npc.vel_x2,
                                npc.vel_y2,
                                npc.anim_num,
                                npc.anim_counter,
                                npc.action_num,
                                npc.action_counter,
                                npc.action_counter2,
                                npc.life,
                                npc.exp,
                                npc.event_num,
                                npc.flag_num,
                                npc.parent_id,
                                npc.shock,
                                npc.size
                            ));
                            ui.text_wrapped(content);

                            cond_flags(ui, &mut npc.cond);
                        }
                    }
                });
        }

        let mut remove = -1;
        for (idx, (_, title, contents)) in self.text_windows.iter().enumerate() {
            let mut opened = true;

            Window::new(title)
                .position([100.0, 100.0], Condition::FirstUseEver)
                .size([400.0, 300.0], Condition::FirstUseEver)
                .opened(&mut opened)
                .build(ui, || {
                    ui.text_wrapped(contents);
                });

            if !opened {
                remove = idx as i32;
            }
        }

        if remove >= 0 {
            self.text_windows.remove(remove as usize);
        }

        if self.error.is_some() {
            Window::new("Error!")
                .resizable(false)
                .collapsible(false)
                .position(
                    [((state.screen_size.0 - 300.0) / 2.0).floor(), ((state.screen_size.1 - 100.0) / 2.0).floor()],
                    Condition::Appearing,
                )
                .size([300.0, 100.0], Condition::Appearing)
                .build(ui, || {
                    ui.push_item_width(-1.0);
                    ui.text_wrapped(self.error.as_ref().unwrap());

                    if ui.button("OK") {
                        self.error = None;
                    }
                });
        }

        Ok(())
    }
}

fn cond_flags(ui: &imgui::Ui, cond: &mut crate::common::Condition) {
    ui.checkbox_flags("Interacted", &mut cond.0, 1);
    ui.checkbox_flags("Hidden", &mut cond.0, 2);
    ui.checkbox_flags("Fallen", &mut cond.0, 4);
    ui.checkbox_flags("Built-in NPC destroy handler", &mut cond.0, 8);
    ui.checkbox_flags("Damage first boss NPC", &mut cond.0, 16);
    ui.checkbox_flags("Increased acceleration", &mut cond.0, 32);
    ui.checkbox_flags("Unknown (0x40)", &mut cond.0, 64);
    ui.checkbox_flags("Alive", &mut cond.0, 128);
}
