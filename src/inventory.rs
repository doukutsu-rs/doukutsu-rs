#[derive(Clone)]
pub struct Weapon {
    id: u16,
    level: u16,
    experience: u16,
    ammo: u16,
    max_ammo: u16,
}

#[derive(Clone, Copy)]
pub struct Item(u16);

#[derive(Clone)]
pub struct Inventory {
    current_item: u16,
    current_weapon: u16,
    items: Vec<Item>,
    weapons: Vec<Weapon>,
}

impl Inventory {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Inventory {
        Inventory {
            current_item: 0,
            current_weapon: 0,
            items: Vec::with_capacity(16),
            weapons: Vec::with_capacity(16),
        }
    }

    pub fn add_item(&mut self, item_id: u16) {
        if !self.has_item(item_id) {
            self.items.push(Item(item_id));
        }
    }

    pub fn remove_item(&mut self, item_id: u16) {
        self.items.retain(|item| item.0 != item_id);
    }

    pub fn has_item(&self, item_id: u16) -> bool {
        self.items.iter().any(|item| item.0 == item_id)
    }
}
