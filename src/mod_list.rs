use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

use drs_framework::io::Read;

use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::mod_requirements::ModRequirements;

#[derive(Debug)]
pub struct ModInfo {
    pub id: String,
    pub requirement: Requirement,
    pub priority: u32,
    pub save_slot: i32,
    pub path: String,
    pub name: String,
    pub description: String,
    pub valid: bool,
}

impl ModInfo {
    pub fn satisfies_requirement(&self, mod_requirements: &ModRequirements) -> bool {
        match self.requirement {
            Requirement::Unlocked => true,
            Requirement::Locked => false,
            Requirement::RequireHell => mod_requirements.beat_hell,
            Requirement::RequireItem(item_id) => mod_requirements.has_item(item_id),
            Requirement::RequireWeapon(weapon_id) => mod_requirements.has_weapon(weapon_id),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Requirement {
    /// R+
    Unlocked,
    /// R-
    Locked,
    /// RI##
    RequireItem(u16),
    /// RA##
    RequireWeapon(u16),
    /// RG
    RequireHell,
}

pub struct ModList {
    pub mods: Vec<ModInfo>,
}

impl ModList {
    pub fn load(ctx: &mut Context, string_table: &HashMap<String, String>) -> GameResult<ModList> {
        let mut mods = Vec::new();

        if let Ok(mut file) = filesystem::open(ctx, "/mods.txt") {
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;
            let mut lines = buf.lines();

            while let Some(line) = lines.next() {
                if line == "=MOD LIST START=" {
                    break;
                }
            }

            while let Some(line) = lines.next() {
                let mut id = String::new();
                let mut requirement = Requirement::Unlocked;
                let mut priority = 1000u32;
                let mut path = String::new();
                let mut chars = line.chars().peekable();

                fn consume_spaces(chars: &mut Peekable<Chars>) {
                    while let Some(&c) = chars.peek() {
                        if c != ' ' {
                            break;
                        }
                        chars.next();
                    }
                }

                consume_spaces(&mut chars);
                id.push_str("csmod_");
                for c in &mut chars {
                    if c == ' ' {
                        break;
                    }

                    id.push(c);
                }
                consume_spaces(&mut chars);

                loop {
                    if let Some(c) = chars.next() {
                        if c == 'R' {
                            if let Some(c) = chars.next() {
                                if c == '+' {
                                    requirement = Requirement::Unlocked;
                                } else if c == '-' {
                                    requirement = Requirement::Locked;
                                } else if c == 'I' {
                                    let mut item_id = String::new();
                                    for c in &mut chars {
                                        if c == ' ' {
                                            break;
                                        }

                                        item_id.push(c);
                                    }
                                    requirement = Requirement::RequireItem(item_id.parse().unwrap_or(0));
                                } else if c == 'A' {
                                    let mut weapon_id = String::new();
                                    for c in &mut chars {
                                        if c == ' ' {
                                            break;
                                        }

                                        weapon_id.push(c);
                                    }
                                    requirement = Requirement::RequireWeapon(weapon_id.parse().unwrap_or(0));
                                } else if c == 'G' {
                                    requirement = Requirement::RequireHell;
                                }
                            }
                        } else if c == 'P' {
                            priority = 0;

                            for c in &mut chars {
                                if c == ' ' {
                                    break;
                                }

                                priority = priority.saturating_mul(10).saturating_add(c.to_digit(10).unwrap_or(0));
                            }
                        } else if c == '/' {
                            path.push(c);
                            while let Some(&c) = chars.peek() {
                                path.push(c);
                                chars.next();
                            }

                            break;
                        } else if c == ' ' {
                            continue;
                        }
                    } else {
                        break;
                    }
                }

                let mut valid = false;
                let mut name = String::new();
                let mut description = String::new();
                let mut save_slot = -1;

                if let Ok(mut file) = filesystem::open(ctx, [&path, "/mod.txt"].join("")) {
                    valid = true;
                    let mut buf = String::new();
                    file.read_to_string(&mut buf)?;
                    let mut lines = buf.lines();

                    if let Some(line) = lines.nth(1) {
                        save_slot = line.parse::<i32>().unwrap_or(-1);
                    } else {
                        save_slot = -1;
                    }

                    if let Some(line) = lines.next() {
                        if let Some(name_str) = string_table.get(line) {
                            name = name_str.to_string();
                        } else {
                            name = line.to_string();
                        }
                    } else {
                        name = "No Mod Name".to_string();
                    }

                    if let Some(line) = lines.next() {
                        description = line.to_string();
                    } else {
                        description = "No Description".to_string();
                    }
                } else {
                    name = path.clone();
                    description = "mod.txt not found".to_string();
                }

                mods.push(ModInfo { id, requirement, priority, save_slot, path, name, description, valid })
            }
        }

        mods.sort_by(|a, b| a.priority.cmp(&b.priority));

        Ok(ModList { mods })
    }

    pub fn get_save_from_path(&self, mod_path: String) -> i32 {
        if let Some(mod_sel) = self.mods.iter().find(|x| x.path == mod_path) {
            mod_sel.save_slot
        } else {
            -1
        }
    }

    pub fn get_name_from_path(&self, mod_path: String) -> &str {
        if let Some(mod_sel) = self.mods.iter().find(|x| x.path == mod_path) {
            &mod_sel.name
        } else {
            "NoName"
        }
    }
}
