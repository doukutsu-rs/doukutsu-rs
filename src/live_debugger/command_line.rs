use num_traits::FromPrimitive;

use crate::framework::error::{GameError::CommandLineError, GameResult};
use crate::npc::NPC;
use crate::scene::game_scene::GameScene;
use crate::scripting::tsc::text_script::{ScriptMode, TextScript, TextScriptEncoding};
use crate::shared_game_state::SharedGameState;
use crate::weapon::WeaponType;

#[derive(Clone)]
pub enum CommandLineCommand {
    AddItem(u16),
    RemoveItem(u16),
    AddWeapon(u16, u16),
    RemoveWeapon(u16),
    AddWeaponAmmo(u16),
    SetWeaponMaxAmmo(u16),
    RefillAmmo,
    RefillHP,
    AddXP(u16),
    RemoveXP(u16),
    SetMaxHP(u16),
    SpawnNPC(u16),
    TSC(String),
}

impl CommandLineCommand {
    pub fn from_components(components: Vec<&str>) -> Option<CommandLineCommand> {
        if components.len() == 0 {
            return None;
        }

        let command = components[0];

        if command.starts_with("<") {
            return Some(CommandLineCommand::TSC(components.join(" ").replace("\\n", "\n")));
        }

        match command.replacen("/", "", 1).as_str() {
            "add_item" => {
                if components.len() < 2 {
                    return None;
                }

                let item_id = components[1].parse::<u16>();
                if item_id.is_ok() {
                    return Some(CommandLineCommand::AddItem(item_id.unwrap()));
                }
            }
            "remove_item" => {
                if components.len() < 2 {
                    return None;
                }

                let item_id = components[1].parse::<u16>();
                if item_id.is_ok() {
                    return Some(CommandLineCommand::RemoveItem(item_id.unwrap()));
                }
            }
            "add_weapon" => {
                if components.len() < 3 {
                    return None;
                }

                let weapon_id = components[1].parse::<u16>();
                let ammo_count = components[2].parse::<u16>();

                if weapon_id.is_ok() && ammo_count.is_ok() {
                    return Some(CommandLineCommand::AddWeapon(weapon_id.unwrap(), ammo_count.unwrap()));
                }
            }
            "remove_weapon" => {
                if components.len() < 2 {
                    return None;
                }

                let weapon_id = components[1].parse::<u16>();
                if weapon_id.is_ok() {
                    return Some(CommandLineCommand::RemoveWeapon(weapon_id.unwrap()));
                }
            }
            "add_weapon_ammo" => {
                if components.len() < 2 {
                    return None;
                }

                let ammo_count = components[1].parse::<u16>();
                if ammo_count.is_ok() {
                    return Some(CommandLineCommand::AddWeaponAmmo(ammo_count.unwrap()));
                }
            }
            "set_weapon_max_ammo" => {
                if components.len() < 2 {
                    return None;
                }

                let max_ammo_count = components[1].parse::<u16>();
                if max_ammo_count.is_ok() {
                    return Some(CommandLineCommand::SetWeaponMaxAmmo(max_ammo_count.unwrap()));
                }
            }
            "refill_ammo" => {
                return Some(CommandLineCommand::RefillAmmo);
            }
            "refill_hp" => {
                return Some(CommandLineCommand::RefillHP);
            }
            "add_xp" => {
                if components.len() < 2 {
                    return None;
                }

                let xp_count = components[1].parse::<u16>();
                if xp_count.is_ok() {
                    return Some(CommandLineCommand::AddXP(xp_count.unwrap()));
                }
            }
            "remove_xp" => {
                if components.len() < 2 {
                    return None;
                }

                let xp_count = components[1].parse::<u16>();
                if xp_count.is_ok() {
                    return Some(CommandLineCommand::RemoveXP(xp_count.unwrap()));
                }
            }
            "set_max_hp" => {
                if components.len() < 2 {
                    return None;
                }

                let hp_count = components[1].parse::<u16>();
                if hp_count.is_ok() {
                    return Some(CommandLineCommand::SetMaxHP(hp_count.unwrap()));
                }
            }
            "spawn_npc" => {
                if components.len() < 2 {
                    return None;
                }
                let npc_id = components[1].parse::<u16>();
                if npc_id.is_ok() {
                    return Some(CommandLineCommand::SpawnNPC(npc_id.unwrap()));
                }
            }
            "tsc" => {
                if components.len() < 2 {
                    return None;
                }

                let script = components[1..].join(" ").replace("\\n", "\n");
                return Some(CommandLineCommand::TSC(script));
            }
            _ => return None,
        }

        None
    }

    pub fn execute(&mut self, game_scene: &mut GameScene, state: &mut SharedGameState) -> GameResult {
        match self.clone() {
            CommandLineCommand::AddItem(item_id) => {
                game_scene.inventory_player1.add_item(item_id);
            }
            CommandLineCommand::RemoveItem(item_id) => {
                if !game_scene.inventory_player1.has_item(item_id) {
                    return Err(CommandLineError(format!("Player does not have item {}", item_id)));
                }

                game_scene.inventory_player1.remove_item(item_id);
            }
            CommandLineCommand::AddWeapon(weapon_id, ammo_count) => {
                let weapon_type: Option<WeaponType> = FromPrimitive::from_u16(weapon_id);
                match weapon_type {
                    Some(weapon_type) => game_scene.inventory_player1.add_weapon(weapon_type, ammo_count),
                    None => return Err(CommandLineError(format!("Invalid weapon id {}", weapon_id))),
                }
            }
            CommandLineCommand::RemoveWeapon(weapon_id) => {
                let weapon_type: Option<WeaponType> = FromPrimitive::from_u16(weapon_id);
                match weapon_type {
                    Some(weapon_type) => {
                        if !game_scene.inventory_player1.has_weapon(weapon_type) {
                            return Err(CommandLineError(format!("Player does not have weapon {:?}", weapon_type)));
                        }

                        game_scene.inventory_player1.remove_weapon(weapon_type);
                    }
                    None => return Err(CommandLineError(format!("Invalid weapon id {}", weapon_id))),
                };
            }
            CommandLineCommand::AddWeaponAmmo(ammo_count) => {
                let weapon = game_scene.inventory_player1.get_current_weapon_mut();
                match weapon {
                    Some(weapon) => weapon.ammo += ammo_count,
                    None => return Err(CommandLineError(format!("Player does not have an active weapon"))),
                }
            }
            CommandLineCommand::SetWeaponMaxAmmo(max_ammo) => {
                let weapon = game_scene.inventory_player1.get_current_weapon_mut();
                match weapon {
                    Some(weapon) => weapon.max_ammo = max_ammo,
                    None => return Err(CommandLineError(format!("Player does not have an active weapon"))),
                }
            }
            CommandLineCommand::RefillAmmo => {
                game_scene.inventory_player1.refill_all_ammo();
            }
            CommandLineCommand::RefillHP => {
                game_scene.player1.life = game_scene.player1.max_life;
            }
            CommandLineCommand::AddXP(xp_count) => {
                game_scene.inventory_player1.add_xp(xp_count, &mut game_scene.player1, state);
            }
            CommandLineCommand::RemoveXP(xp_count) => {
                game_scene.inventory_player1.take_xp(xp_count, state);
            }
            CommandLineCommand::SetMaxHP(hp_count) => {
                game_scene.player1.max_life = hp_count;
                game_scene.player1.life = hp_count;
            }
            CommandLineCommand::SpawnNPC(id) => {
                let mut npc = NPC::create(id, &state.npc_table);
                npc.cond.set_alive(true);
                npc.y = game_scene.player1.y;
                npc.x = game_scene.player1.x + game_scene.player1.direction.vector_x() * (0x2000 * 3);
                game_scene.npc_list.spawn(0x100, npc)?;
            }
            CommandLineCommand::TSC(script) => {
                log::info!("Executing TSC script: {}", format!("#9999\n{}", script));
                match TextScript::compile(format!("#9999\n{}", script).as_bytes(), true, TextScriptEncoding::UTF8) {
                    Ok(text_script) => {
                        state.textscript_vm.set_debug_script(text_script);
                        state.textscript_vm.set_mode(ScriptMode::Debug);
                        state.textscript_vm.start_script(9999);
                    }
                    Err(err) => {
                        return Err(CommandLineError(format!("Error compiling TSC: {}", err)));
                    }
                };
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn to_command(&self) -> String {
        match self {
            CommandLineCommand::AddItem(item_id) => format!("/add_item {}", item_id),
            CommandLineCommand::RemoveItem(item_id) => format!("/remove_item {}", item_id),
            CommandLineCommand::AddWeapon(weapon_id, ammo_count) => format!("/add_weapon {} {}", weapon_id, ammo_count),
            CommandLineCommand::RemoveWeapon(weapon_id) => format!("/remove_weapon {}", weapon_id),
            CommandLineCommand::AddWeaponAmmo(ammo_count) => format!("/add_weapon_ammo {}", ammo_count),
            CommandLineCommand::SetWeaponMaxAmmo(max_ammo_count) => format!("/set_weapon_max_ammo {}", max_ammo_count),
            CommandLineCommand::RefillAmmo => "/refill_ammo".to_string(),
            CommandLineCommand::RefillHP => "/refill_hp".to_string(),
            CommandLineCommand::AddXP(xp_count) => format!("/add_xp {}", xp_count),
            CommandLineCommand::RemoveXP(xp_count) => format!("/remove_xp {}", xp_count),
            CommandLineCommand::SetMaxHP(hp_count) => format!("/set_max_hp {}", hp_count),
            CommandLineCommand::SpawnNPC(npc_id) => format!("/spawn_npc {}", npc_id),
            CommandLineCommand::TSC(script) => format!("/tsc {}", script.replace("\n", "\\n")),
        }
    }

    pub fn feedback_string(&self) -> String {
        match self {
            CommandLineCommand::AddItem(item_id) => format!("Added item with ID {}.", item_id),
            CommandLineCommand::RemoveItem(item_id) => format!("Removed item with ID {}.", item_id),
            CommandLineCommand::AddWeapon(weapon_id, ammo_count) => {
                format!("Added weapon with ID {} and {} ammo.", weapon_id, ammo_count)
            }
            CommandLineCommand::RemoveWeapon(weapon_id) => format!("Removed weapon with ID {}.", weapon_id),
            CommandLineCommand::AddWeaponAmmo(ammo_count) => format!("Added {} ammo to current weapon.", ammo_count),
            CommandLineCommand::SetWeaponMaxAmmo(max_ammo_count) => {
                format!("Set max ammo of current weapon to {}.", max_ammo_count)
            }
            CommandLineCommand::RefillAmmo => "Refilled ammo of all weapons.".to_string(),
            CommandLineCommand::RefillHP => "Refilled HP of player.".to_string(),
            CommandLineCommand::AddXP(xp_count) => format!("Added {} XP to current weapon.", xp_count),
            CommandLineCommand::RemoveXP(xp_count) => format!("Removed {} XP from current weapon.", xp_count),
            CommandLineCommand::SetMaxHP(hp_count) => format!("Set max HP of player to {}.", hp_count),
            CommandLineCommand::SpawnNPC(npc_id) => format!("Spawned NPC ID {} in front of player.", npc_id),
            CommandLineCommand::TSC(_) => "Executed TSC script.".to_string(),
        }
    }
}

pub struct CommandLineParser {
    command_history: Vec<CommandLineCommand>,
    cursor: usize,
    pub last_feedback: String,
    pub last_feedback_color: [f32; 4],
    pub buffer: String,
}

impl CommandLineParser {
    pub fn new() -> CommandLineParser {
        CommandLineParser {
            command_history: Vec::new(),
            last_feedback: "Awaiting command.".to_string(),
            last_feedback_color: [1.0, 1.0, 1.0, 1.0],
            cursor: 0,
            buffer: String::new(),
        }
    }

    pub fn push(&mut self, command: String) -> Option<CommandLineCommand> {
        let components = command.split_whitespace().collect::<Vec<&str>>();
        let command = CommandLineCommand::from_components(components);

        match command {
            Some(command) => {
                self.command_history.push(command.clone());
                self.cursor = self.command_history.len() - 1;

                Some(command)
            }
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn traverse(&mut self, delta: i16) -> Option<&CommandLineCommand> {
        if self.command_history.is_empty() {
            return None;
        }

        let command = self.command_history.get(self.cursor);

        if delta == -1 && self.cursor == 0 {
            self.cursor = self.command_history.len() - 1;
        } else if delta == 1 && self.cursor == self.command_history.len() - 1 {
            self.cursor = 0;
        } else {
            self.cursor = (self.cursor as i16 + delta) as usize;
        }

        command
    }
}
