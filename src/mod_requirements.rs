use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ModRequirements {
    #[serde(default = "current_version")]
    pub version: u32,

    pub beat_hell: bool,
    pub weapons: [u16; 8],
    pub items: [u16; 32],
}

#[inline(always)]
fn current_version() -> u32 {
    1
}

impl ModRequirements {
    pub fn load(ctx: &Context) -> GameResult<ModRequirements> {
        let mut version = current_version();
        let mut beat_hell: bool = false;
        let mut weapons = [0u16; 8];
        let mut items = [0u16; 32];

        if let Ok(mut file) = filesystem::user_open(ctx, "/mod.req") {
            version = file.read_u32::<LE>()?;
            beat_hell = file.read_u16::<LE>()? != 0;

            for weapon in &mut weapons {
                *weapon = file.read_u16::<LE>()?;
            }

            for item in &mut items {
                *item = file.read_u16::<LE>()?;
            }
        }

        let mod_requirements = ModRequirements { version, beat_hell, weapons, items };

        Ok(mod_requirements.upgrade())
    }

    fn upgrade(mut self) -> Self {
        self
    }

    pub fn save(&self, ctx: &Context) -> GameResult {
        let mut file = filesystem::user_create(ctx, "/mod.req")?;
        file.write_u32::<LE>(self.version)?;
        file.write_u16::<LE>(self.beat_hell as u16)?;

        for weapon in &self.weapons {
            file.write_u16::<LE>(*weapon)?;
        }

        for item in &self.items {
            file.write_u16::<LE>(*item)?;
        }

        Ok(())
    }

    pub fn append_weapon(&mut self, ctx: &Context, weapon_id: u16) -> GameResult {
        for i in 0..self.weapons.len() {
            if self.weapons[i] == weapon_id {
                return Ok(());
            }

            if self.weapons[i] == 0 {
                self.weapons[i] = weapon_id;
                break;
            }
        }

        self.save(ctx)
    }

    pub fn append_item(&mut self, ctx: &Context, item_id: u16) -> GameResult {
        for i in 0..self.items.len() {
            if self.items[i] == item_id {
                return Ok(());
            }

            if self.items[i] == 0 {
                self.items[i] = item_id;
                break;
            }
        }

        self.save(ctx)
    }

    pub fn has_weapon(&self, weapon_id: u16) -> bool {
        for weapon in &self.weapons {
            if *weapon == weapon_id {
                return true;
            }
        }

        false
    }

    pub fn has_item(&self, item_id: u16) -> bool {
        for item in &self.items {
            if *item == item_id {
                return true;
            }
        }

        false
    }
}
