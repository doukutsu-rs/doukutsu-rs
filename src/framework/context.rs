use crate::framework::backend::{init_backend, BackendRenderer};
use crate::framework::error::GameResult;
use crate::framework::filesystem::Filesystem;
use crate::framework::keyboard::KeyboardContext;
use crate::Game;

pub struct Context {
    pub(crate) filesystem: Filesystem,
    pub(crate) renderer: Option<Box<dyn BackendRenderer>>,
    pub(crate) keyboard_context: KeyboardContext,
    pub(crate) screen_size: (f32, f32),
    pub(crate) screen_insets: (f32, f32, f32, f32),
}

impl Context {
    pub fn new() -> Context {
        Context {
            filesystem: Filesystem::new(),
            renderer: None,
            keyboard_context: KeyboardContext::new(),
            screen_size: (320.0, 240.0),
            screen_insets: (0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn run(&mut self, game: &mut Game) -> GameResult {
        let backend = init_backend()?;
        let mut event_loop = backend.create_event_loop()?;
        self.renderer = Some(event_loop.new_renderer()?);

        event_loop.run(game, self);

        Ok(())
    }
}
