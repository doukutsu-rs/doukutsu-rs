use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::iter::Peekable;
use std::str::Chars;

use crate::framework::context::Context;
use crate::framework::error::GameResult;
use crate::framework::filesystem;
use crate::mod_requirements::ModRequirements;

#[derive(Debug, Clone)]
pub struct ModInfo {
    pub id: String,
    pub requirement: Requirement,
    pub priority: u32,
    pub save_slot: i32,
    pub path: String,
    pub name: Option<String>,
    pub description: Option<String>,
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

        if let Ok(file) = filesystem::open(ctx, "/mods.txt") {
            let reader = BufReader::new(file);
            let mut lines = reader.lines();

            while let Some(Ok(line)) = lines.next() {
                if line == "=MOD LIST START=" {
                    break;
                }
            }

            while let Some(Ok(line)) = lines.next() {
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
                id.push_str("cspmod_");
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
                let mut name: Option<String> = None;
                let mut description: Option<String> = None;
                let mut save_slot = -1;

                if let Ok(file) = filesystem::open(ctx, [&path, "/mod.txt"].join("")) {
                    valid = true;
                    let reader = BufReader::new(file);
                    let mut lines = reader.lines();
                    if let Some(line) = lines.nth(1) {
                        save_slot = line.unwrap_or("-1".to_string()).parse::<i32>().unwrap_or(-1);
                    }
                    if let Some(line) = lines.next() {
                        name = line.ok().and_then(|read_name| {
                            let name = string_table.get(&read_name).or(Some(&read_name)).cloned();
                            name.filter(|s| !s.is_empty())
                        });
                    }
                    if let Some(line) = lines.next() {
                        description = line.ok().filter(|s| !s.is_empty());
                    }
                }

                log::debug!("CSP Mod Loaded: {:?}", ModInfo { id: id.clone(), requirement, priority, save_slot, path: path.clone(), name: name.clone(), description: description.clone(), valid });
                mods.push(ModInfo { id, requirement, priority, save_slot, path, name, description, valid })
            }
        }

        mods.sort_by(|a, b| a.priority.cmp(&b.priority));

        Ok(ModList { mods })
    }

    pub fn get_info_from_path(&self, mod_path: String) -> Option<&ModInfo> {
        self.mods.iter().find(|x| x.path == mod_path)
    }

    pub fn get_save_from_path(&self, mod_path: String) -> i32 {
        self.get_info_from_path(mod_path).and_then(|mod_info| Some(mod_info.save_slot)).unwrap_or(-1)
    }

    pub fn get_name_from_path(&self, mod_path: String) -> Option<&str> {
        self.get_info_from_path(mod_path).and_then(|mod_info| mod_info.name.as_deref())
    }
}
