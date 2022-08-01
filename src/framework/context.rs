use crate::framework::backend::{init_backend, BackendRenderer};
use crate::framework::error::GameResult;
use crate::framework::filesystem::Filesystem;
use crate::framework::gamepad::GamepadContext;
use crate::framework::keyboard::KeyboardContext;
use crate::graphics::VSyncMode;
use crate::Game;

pub struct Context {
    pub headless: bool,
    pub size_hint: (u16, u16),
    pub(crate) filesystem: Filesystem,
    pub(crate) renderer: Option<Box<dyn BackendRenderer>>,
    pub(crate) gamepad_context: GamepadContext,
    pub(crate) keyboard_context: KeyboardContext,
    pub(crate) real_screen_size: (u32, u32),
    pub(crate) screen_size: (f32, f32),
    pub(crate) screen_insets: (f32, f32, f32, f32),
    pub(crate) vsync_mode: VSyncMode,
}

impl Context {
    pub fn new() -> Context {
        Context {
            headless: false,
            size_hint: (640, 480),
            filesystem: Filesystem::new(),
            renderer: None,
            gamepad_context: GamepadContext::new(),
            keyboard_context: KeyboardContext::new(),
            real_screen_size: (320, 240),
            screen_size: (320.0, 240.0),
            screen_insets: (0.0, 0.0, 0.0, 0.0),
            vsync_mode: VSyncMode::Uncapped,
        }
    }

    pub fn run(&mut self, game: &mut Game) -> GameResult {
        let backend = init_backend(self.headless, self.size_hint)?;
        let mut event_loop = backend.create_event_loop(self)?;
        self.renderer = Some(event_loop.new_renderer(self as *mut Context)?);

        event_loop.run(game, self);

        Ok(())
    }
}
