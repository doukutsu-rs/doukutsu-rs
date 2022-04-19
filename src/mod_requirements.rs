use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ModRequirements {
    #[serde(default = "current_version")]
    pub version: u32,

    pub beat_hell: bool,
    pub weapons: Vec<u16>,
    pub items: Vec<u16>,
}

#[inline(always)]
fn current_version() -> u32 {
    1
}

impl ModRequirements {
    pub fn load(ctx: &Context) -> GameResult<ModRequirements> {
        if let Ok(file) = filesystem::user_open(ctx, "/mod_req.json") {
            match serde_json::from_reader::<_, ModRequirements>(file) {
                Ok(mod_req) => return Ok(mod_req.upgrade()),
                Err(err) => log::warn!("Failed to deserialize mod requirements: {}", err),
            }
        }

        Ok(ModRequirements::default())
    }

    fn upgrade(self) -> Self {
        self
    }

    pub fn save(&self, ctx: &Context) -> GameResult {
        let file = filesystem::user_create(ctx, "/mod_req.json")?;
        serde_json::to_writer_pretty(file, self)?;

        Ok(())
    }

    pub fn append_weapon(&mut self, ctx: &Context, weapon_id: u16) -> GameResult {
        if self.weapons.contains(&weapon_id) {
            return Ok(());
        }

        self.weapons.push(weapon_id);
        self.save(ctx)
    }

    pub fn append_item(&mut self, ctx: &Context, item_id: u16) -> GameResult {
        if self.items.contains(&item_id) {
            return Ok(());
        }

        self.items.push(item_id);
        self.save(ctx)
    }

    pub fn has_weapon(&self, weapon_id: u16) -> bool {
        self.weapons.contains(&weapon_id)
    }

    pub fn has_item(&self, item_id: u16) -> bool {
        self.items.contains(&item_id)
    }
}

impl Default for ModRequirements {
    fn default() -> Self {
        ModRequirements { version: current_version(), beat_hell: false, weapons: Vec::new(), items: Vec::new() }
    }
}
