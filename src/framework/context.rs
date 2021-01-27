use crate::framework::filesystem::Filesystem;
use crate::Game;

pub struct Context {
    pub(crate) filesystem: Filesystem,
}

impl Context {
    pub fn new() -> Context {
        Context {
            filesystem: Filesystem::new(),
        }
    }

    pub fn run(&mut self, game: &mut Game) {
        loop {}
    }
}