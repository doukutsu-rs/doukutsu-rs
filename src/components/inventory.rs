
pub struct Inventory {
    text_y_pos: usize,
    tick: usize,
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            text_y_pos: 24,
            tick: 0,
        }
    }
}
