use std::cell::RefCell;
use std::rc::Rc;

use downcast::Downcast;
use imgui::{Condition, MenuItem, TabItem, TabItemFlags, Window};

use crate::editor::{CurrentTool, EditorInstance};
use crate::framework::keyboard;
use crate::framework::keyboard::ScanCode;
use crate::framework::ui::Components;
use crate::scene::game_scene::GameScene;
use crate::scene::title_scene::TitleScene;
use crate::stage::Stage;
use crate::{Context, GameResult, Scene, SharedGameState};

struct ErrorList {
    errors: Vec<String>,
}

impl ErrorList {
    fn new() -> ErrorList {
        Self { errors: Vec::new() }
    }

    fn try_or_push_error(&mut self, func: impl FnOnce() -> GameResult<()>) {
        if let Err(err) = func() {
            self.errors.push(err.to_string());
        }
    }
}

fn catch(error_list: Rc<RefCell<ErrorList>>, func: impl FnOnce() -> GameResult<()>) {
    error_list.borrow_mut().try_or_push_error(func);
}

pub struct EditorScene {
    stage_list: StageListWindow,
    error_list: Rc<RefCell<ErrorList>>,
    instances: Vec<EditorInstance>,
    subscene: Option<Box<GameScene>>,
    current_tool: CurrentTool,
    selected_instance: usize,
    switch_tab: bool,
}

impl EditorScene {
    pub fn new() -> Self {
        EditorScene {
            stage_list: StageListWindow::new(),
            error_list: Rc::new(RefCell::new(ErrorList::new())),
            instances: Vec::new(),
            subscene: None,
            current_tool: CurrentTool::Move,
            selected_instance: 0,
            switch_tab: false,
        }
    }

    fn exit_editor(&mut self, state: &mut SharedGameState) {
        state.next_scene = Some(Box::new(TitleScene::new()));
    }

    fn open_stage(&mut self, state: &mut SharedGameState, ctx: &mut Context, stage_id: usize) {
        catch(self.error_list.clone(), || {
            for (idx, instance) in self.instances.iter().enumerate() {
                if instance.stage_id == stage_id {
                    self.selected_instance = idx;
                    self.switch_tab = true;
                    return Ok(());
                }
            }

            if let Some(stage) = state.stages.get(stage_id) {
                let stage = Stage::load(&state.base_path, stage, ctx)?;

                let new_instance = EditorInstance::new(stage_id, stage);
                self.instances.push(new_instance);
                self.selected_instance = self.instances.len() - 1;
                self.switch_tab = true;
            }

            Ok(())
        });
    }

    fn test_stage(&mut self, state: &mut SharedGameState, ctx: &mut Context) {
        catch(self.error_list.clone(), || {
            if let Some(instance) = self.instances.get(self.selected_instance) {
                state.reset();
                state.textscript_vm.start_script(94);
                let mut game_scene = GameScene::from_stage(state, ctx, instance.stage.clone(), instance.stage_id)?;
                game_scene.init(state, ctx)?;
                game_scene.player1.cond.set_alive(true);
                game_scene.player1.x = instance.frame.x + (state.canvas_size.0 * 256.0) as i32;
                game_scene.player1.y = instance.frame.y + (state.canvas_size.1 * 256.0) as i32;
                state.control_flags.set_control_enabled(true);
                state.control_flags.set_tick_world(true);
                state.textscript_vm.suspend = false;
                self.subscene = Some(Box::new(game_scene));
            }

            Ok(())
        });
    }

    fn perform_actions(&mut self, state: &mut SharedGameState, ctx: &mut Context) {
        let actions = std::mem::take(&mut self.stage_list.actions);
        for action in actions.iter() {
            match action {
                StageListAction::OpenStage(idx) => self.open_stage(state, ctx, *idx),
            }
        }
    }
}

trait ExtraWidgetsExt {
    fn tool_button(&self, label: impl AsRef<str>, active: bool) -> bool;
}

impl ExtraWidgetsExt for imgui::Ui<'_> {
    fn tool_button(&self, label: impl AsRef<str>, active: bool) -> bool {
        if active {
            let color = self.style_color(imgui::StyleColor::ButtonActive);
            let _token1 = self.push_style_color(imgui::StyleColor::Button, color);
            let _token2 = self.push_style_color(imgui::StyleColor::ButtonHovered, color);
            let ret = self.button(label);
            ret
        } else {
            self.button(label)
        }
    }
}

impl Scene for EditorScene {
    fn init(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        state.sound_manager.play_song(0, &state.constants, &state.settings, ctx)?;

        Ok(())
    }

    fn tick(&mut self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        let subscene_ref = &mut self.subscene;
        if subscene_ref.is_some() {
            subscene_ref.as_mut().unwrap().tick(state, ctx)?;

            if keyboard::is_key_pressed(ctx, ScanCode::Escape) {
                *subscene_ref = None;
            }

            // hijack scene switches
            let next_scene = std::mem::take(&mut state.next_scene);
            if let Some(next_scene) = next_scene {
                *subscene_ref = if let Ok(game_scene) = next_scene.downcast() {
                    let mut game_scene: Box<GameScene> = game_scene;
                    game_scene.init(state, ctx)?;
                    Some(game_scene)
                } else {
                    None
                };
            }

            if subscene_ref.is_none() {
                state.sound_manager.play_song(0, &state.constants, &state.settings, ctx)?;
            }

            return Ok(());
        }

        Ok(())
    }

    fn draw_tick(&mut self, state: &mut SharedGameState) -> GameResult {
        if let Some(scene) = &mut self.subscene {
            scene.draw_tick(state)?;
            return Ok(());
        }

        Ok(())
    }

    fn draw(&self, state: &mut SharedGameState, ctx: &mut Context) -> GameResult {
        if let Some(scene) = &self.subscene {
            scene.draw(state, ctx)?;
            state.font.draw_text(
                "Press [ESC] to return.".chars(),
                4.0,
                4.0,
                &state.constants,
                &mut state.texture_set,
                ctx,
            )?;
            return Ok(());
        }

        if let Some(instance) = self.instances.get(self.selected_instance) {
            instance.draw(state, ctx, self.current_tool)?;
        }

        Ok(())
    }

    fn imgui_draw(
        &mut self,
        _game_ui: &mut Components,
        state: &mut SharedGameState,
        ctx: &mut Context,
        ui: &mut imgui::Ui,
    ) -> GameResult {
        self.perform_actions(state, ctx);

        if let Some(_) = self.subscene {
            return Ok(());
        }

        let mut menu_bar_size = (0.0, 0.0);
        if let Some(menu_bar) = ui.begin_main_menu_bar() {
            let [menu_bar_w, menu_bar_h] = ui.window_size();
            menu_bar_size = (menu_bar_w, menu_bar_h);

            if let Some(menu) = ui.begin_menu("File") {
                if MenuItem::new("Open stage").shortcut("Ctrl+O").build(ui) {
                    self.stage_list.show();
                }

                ui.separator();

                if MenuItem::new("Exit editor").build(ui) {
                    self.exit_editor(state);
                }

                menu.end();
            }
            menu_bar.end();
        }

        Window::new("Toolbar")
            .title_bar(false)
            .resizable(false)
            .position([0.0, menu_bar_size.1], Condition::Always)
            .size([menu_bar_size.0, 0.0], Condition::Always)
            .build(ui, || {
                if ui.tool_button("Move", self.current_tool == CurrentTool::Move) {
                    self.current_tool = CurrentTool::Move;
                }
                ui.same_line();
                if ui.tool_button("Brush", self.current_tool == CurrentTool::Brush) {
                    self.current_tool = CurrentTool::Brush;
                }
                ui.same_line();
                if ui.tool_button("Fill", self.current_tool == CurrentTool::Fill) {
                    self.current_tool = CurrentTool::Fill;
                }
                ui.same_line();
                if ui.tool_button("Rectangle", self.current_tool == CurrentTool::Rectangle) {
                    self.current_tool = CurrentTool::Rectangle;
                }

                ui.same_line();
                ui.text("|");

                ui.same_line();
                if ui.button("Test Stage") {
                    self.test_stage(state, ctx);
                }

                if let Some(tab) = ui.tab_bar("Stages") {
                    for (idx, inst) in self.instances.iter().enumerate() {
                        let mut flags = TabItemFlags::NO_CLOSE_WITH_MIDDLE_MOUSE_BUTTON;
                        if self.switch_tab && self.selected_instance == idx {
                            self.switch_tab = false;
                            flags |= TabItemFlags::SET_SELECTED;
                        }

                        if let Some(item) = TabItem::new(&inst.stage.data.name).flags(flags).begin(ui) {
                            if !self.switch_tab {
                                self.selected_instance = idx;
                            }
                            item.end();
                        }
                    }

                    tab.end();
                }
            });

        self.stage_list.action(state, ctx, ui);

        if let Some(instance) = self.instances.get_mut(self.selected_instance) {
            instance.process(state, ctx, ui, self.current_tool);
        }

        Ok(())
    }
}

struct StageListWindow {
    visible: bool,
    selected_stage: i32,
    actions: Vec<StageListAction>,
}

enum StageListAction {
    OpenStage(usize),
}

impl StageListWindow {
    fn new() -> Self {
        StageListWindow { visible: false, selected_stage: 0, actions: Vec::new() }
    }

    fn show(&mut self) {
        self.visible = true;
    }

    fn action(&mut self, state: &mut SharedGameState, ctx: &mut Context, ui: &mut imgui::Ui) {
        if !self.visible {
            return;
        }

        Window::new("Stage list")
            .resizable(false)
            .collapsible(false)
            .position_pivot([0.5, 0.5])
            .size([300.0, 352.0], Condition::FirstUseEver)
            .build(ui, || {
                let mut stages = Vec::with_capacity(state.stages.len());
                for stage in state.stages.iter() {
                    stages.push(stage.name.as_str());
                }

                ui.push_item_width(-1.0);
                ui.list_box("", &mut self.selected_stage, &stages, 14);

                ui.disabled(self.selected_stage < 0, || {
                    if ui.button("Open") {
                        self.actions.push(StageListAction::OpenStage(self.selected_stage as usize));
                    }

                    ui.same_line();
                    if ui.button("Edit table entry") {}
                });

                ui.same_line();
                if ui.button("Cancel") {
                    self.visible = false;
                }
            });
    }
}
