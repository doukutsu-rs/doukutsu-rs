use crate::Game;

pub(crate) trait Backend {
    fn create_event_loop(&self) -> Box<dyn BackendEventLoop>;
}

pub(crate) trait BackendEventLoop {
    fn run(&self, game: &mut Game);
}