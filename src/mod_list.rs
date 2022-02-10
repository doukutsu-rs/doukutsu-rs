use std::io::{BufRead, BufReader};
use std::iter::Peekable;
use std::str::Chars;

use crate::framework::filesystem;
use crate::{Context, GameResult};

#[derive(Debug)]
pub struct ModInfo {
    pub id: String,
    pub requirement: Requirement,
    pub priority: u32,
    pub path: String,
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
    pub fn load(ctx: &mut Context) -> GameResult<ModList> {
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
                                    chars.next();
                                    let mut item_id = String::new();
                                    for c in &mut chars {
                                        if c == ' ' {
                                            break;
                                        }

                                        item_id.push(c);
                                    }
                                    requirement = Requirement::RequireItem(item_id.parse().unwrap_or(0));
                                } else if c == 'A' {
                                    chars.next();
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

                mods.push(ModInfo { id, requirement, priority, path })
            }
        }

        mods.sort_by(|a, b| a.priority.cmp(&b.priority));

        Ok(ModList { mods })
    }
}
