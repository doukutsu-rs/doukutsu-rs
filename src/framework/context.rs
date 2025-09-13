// SPDX-License-Identifier: MIT
// Copyright (c) 2016 ggez-dev
// Copyright (c) 2020 doukutsu-rs contributors (see AUTHORS.md)
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

use super::backend::{init_backend, BackendFlag, BackendRenderer, WindowParams};
use super::error::GameResult;
use super::filesystem::Filesystem;
use super::gamepad::GamepadContext;
use super::graphics::SwapMode;
use super::keyboard::KeyboardContext;
use super::ui::init_imgui;
use crate::game::Game;

pub struct Context {
    pub headless: bool,
    pub shutdown_requested: bool,
    pub suspended: bool,
    pub window: WindowParams,
    pub flags: BackendFlag,
    pub(crate) imgui: Rc<RefCell<imgui::Context>>,
    pub(crate) filesystem: Filesystem,
    pub(crate) renderer: Option<Box<dyn BackendRenderer>>,
    pub(crate) gamepad_context: GamepadContext,
    pub(crate) keyboard_context: KeyboardContext,
    pub(crate) real_screen_size: (u32, u32),
    pub(crate) screen_size: (f32, f32),
    pub(crate) screen_insets: (f32, f32, f32, f32),
    pub(crate) swap_mode: SwapMode,
}

impl Context {
    pub fn new() -> Context {
        Context {
            headless: false,
            shutdown_requested: false,
            suspended: false,
            window: WindowParams::default(),
            flags: BackendFlag::new(),
            imgui: init_imgui(),
            filesystem: Filesystem::new(),
            renderer: None,
            gamepad_context: GamepadContext::new(),
            keyboard_context: KeyboardContext::new(),
            real_screen_size: (320, 240),
            screen_size: (320.0, 240.0),
            screen_insets: (0.0, 0.0, 0.0, 0.0),
            swap_mode: SwapMode::VSync,
        }
    }

    pub fn run(mut self: Pin<Box<Self>>, game: Pin<Box<Game>>) -> GameResult {
        let backend = init_backend(self.headless, self.window)?;
        let mut event_loop = backend.create_event_loop(&self)?;
        self.renderer = Some(event_loop.new_renderer(&mut self)?);

        event_loop.run(game, self);

        Ok(())
    }

    /// Requests the game to shut down.
    pub fn shutdown(&mut self) {
        self.shutdown_requested = true;
    }
}
