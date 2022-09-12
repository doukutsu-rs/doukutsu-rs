use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use laminar::{Socket, SocketEvent};
use rand::rngs::ThreadRng;
use rand::{RngCore, SeedableRng};

use crate::framework::error::GameResult;
use crate::netplay::common::{make_socket_config, SenderExt};
use crate::netplay::protocol::{DRSPacket, HelloData, PlayerData, PlayerMove, ServerInfo, StageData, TextScriptData};
use crate::netplay::server_config::ServerConfiguration;
use crate::player::TargetPlayer;
use crate::profile::GameProfile;
use crate::scene::game_scene::GameScene;
use crate::SharedGameState;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
enum DeliveryType {
    Reliable,
    Unreliable,
}

enum SyncMessage {
    SyncStageToPlayer(TargetPlayer),
    SyncNewPlayer(TargetPlayer),
    PlayerLeft(TargetPlayer),
}

pub struct Server {
    _thread: JoinHandle<()>,
    game_packet_queue: Receiver<DRSPacket>,
    sync_message_queue: Receiver<SyncMessage>,
    broadcast_packet_queue: Sender<(DRSPacket, DeliveryType)>,
    send_packet_queue: Sender<(TargetPlayer, DRSPacket, DeliveryType)>,
    tick: usize,
}

struct PlayerState {
    target: TargetPlayer,
    name: String,
}

impl Server {
    pub fn start(config: ServerConfiguration) -> GameResult<Server> {
        let mut socket = Socket::bind_with_config(config.bind_to, make_socket_config())?;
        let (pq_tx, pq_rx) = crossbeam_channel::bounded(2048);
        let (bq_tx, bq_rx) = crossbeam_channel::bounded::<(DRSPacket, DeliveryType)>(2048);
        let (sq_tx, sq_rx) = crossbeam_channel::bounded::<(TargetPlayer, DRSPacket, DeliveryType)>(2048);
        let (mq_tx, mq_rx) = crossbeam_channel::bounded::<SyncMessage>(32);

        log::info!("Listening on {:?}.", socket.local_addr()?);

        let thread = thread::spawn(move || {
            let receiver = socket.get_event_receiver();
            let sender = socket.get_packet_sender();
            let mut players = HashMap::<SocketAddr, PlayerState>::new();
            let mut rng = ThreadRng::default();

            loop {
                let mut test = [0u8; 4];
                if rng.try_fill_bytes(&mut test).is_ok() {
                    break;
                }

                log::warn!("The system RNG is not ready, waiting for initialization...");
                sleep(Duration::from_millis(1000));
            }

            loop {
                socket.manual_poll(Instant::now());
                sleep(Duration::from_millis(1));

                while let Ok((packet, delivery)) = bq_rx.try_recv() {
                    match delivery {
                        DeliveryType::Reliable => {
                            sender.broadcast_reliable(players.keys().into_iter().copied(), packet);
                        }
                        DeliveryType::Unreliable => {
                            sender.broadcast_unreliable(players.keys().into_iter().copied(), packet);
                        }
                    }
                }

                while let Ok((target, packet, delivery)) = sq_rx.try_recv() {
                    let entry = players.iter().find(|(_, state)| state.target == target);

                    if let Some(entry) = entry {
                        match delivery {
                            DeliveryType::Reliable => {
                                sender.send_reliable(*entry.0, packet);
                            }
                            DeliveryType::Unreliable => {
                                sender.send_unreliable(*entry.0, packet);
                            }
                        }
                    }
                }

                while let Ok(packet) = receiver.try_recv() {
                    match packet {
                        SocketEvent::Packet(p) => {
                            let player = players.get(&p.addr());

                            if let Ok(payload) = DRSPacket::decode(p.payload()) {
                                match payload {
                                    DRSPacket::KeepAlive => {
                                        sender.send_reliable(p.addr(), DRSPacket::KeepAlive);
                                    }
                                    DRSPacket::ServerInfoRequest => {
                                        sender.send_reliable(
                                            p.addr(),
                                            DRSPacket::ServerInfoResponse(ServerInfo {
                                                motd: "A doukutsu-rs server".to_string(),
                                                players: (0, 2),
                                            }),
                                        );
                                    }
                                    DRSPacket::Connect(info) => {
                                        if player.is_some() {
                                            sender.kick(p.addr(), "Invalid state.");
                                            continue;
                                        }

                                        if players.len() == 2 {
                                            sender.kick(p.addr(), "Too many players are connected right now.");
                                            continue;
                                        }

                                        let mut target = TargetPlayer::Player1;
                                        for (_, state) in players.iter() {
                                            if state.target == TargetPlayer::Player1 {
                                                target = TargetPlayer::Player2;
                                            }
                                        }

                                        players.insert(p.addr(), PlayerState { target, name: info.name.clone() });

                                        sender.send_reliable(p.addr(), DRSPacket::ConnectResponse(target));

                                        sender.broadcast_reliable(
                                            players.keys().into_iter().copied(),
                                            DRSPacket::ChatMessage(format!(
                                                "§2(§a+§2) §7Player §f{} §7joined the game.",
                                                info.name
                                            )),
                                        );

                                        let _ = mq_tx.send(SyncMessage::SyncNewPlayer(target));
                                        let _ = mq_tx.send(SyncMessage::SyncStageToPlayer(target));
                                    }
                                    DRSPacket::Move(mut plr_move) => {
                                        if let Some(player) = player {
                                            plr_move.target = player.target;
                                            let _ = pq_tx.send(DRSPacket::Move(plr_move));
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                        SocketEvent::Connect(addr) => {
                            log::info!("Client {:?} connected.", addr);
                            let mut challenge = [0u8; 32];
                            if rng.try_fill_bytes(&mut challenge).is_err() {
                                log::error!("Failed to generate challenge, the RNG is not ready?");
                                continue;
                            }

                            sender.send_reliable(addr, DRSPacket::Hello(HelloData { challenge }))
                        }
                        SocketEvent::Timeout(addr) => {
                            log::info!("Client {:?} timed out.", addr);
                        }
                        SocketEvent::Disconnect(addr) => {
                            log::info!("Client {:?} disconnected.", addr);
                            let info = players.remove(&addr);

                            if let Some(info) = info {
                                sender.broadcast_reliable(
                                    players.keys().into_iter().copied(),
                                    DRSPacket::ChatMessage(format!(
                                        "§4(§c-§4) §7Player §f{} §7left the game.",
                                        info.name
                                    )),
                                );
                                let _ = mq_tx.send(SyncMessage::PlayerLeft(info.target));
                            }
                        }
                    }
                }
            }
        });

        Ok(Server {
            _thread: thread,
            game_packet_queue: pq_rx,
            sync_message_queue: mq_rx,
            broadcast_packet_queue: bq_tx,
            send_packet_queue: sq_tx,
            tick: 0,
        })
    }

    pub fn process(&mut self, state: &mut SharedGameState, game_scene: &mut GameScene) {
        self.tick = self.tick.wrapping_add(1);

        while let Ok(packet) = self.game_packet_queue.try_recv() {
            match packet {
                DRSPacket::Move(plr_move) => {
                    let player = if plr_move.target == TargetPlayer::Player1 {
                        &mut game_scene.player1
                    } else {
                        &mut game_scene.player2
                    };
                    player.x = plr_move.x;
                    player.y = plr_move.y;
                    player.vel_x = plr_move.vel_x;
                    player.vel_y = plr_move.vel_y;
                    player.direction = plr_move.direction;
                    player.cond = plr_move.cond;
                    player.controller.set_state((plr_move.state, plr_move.old_state, plr_move.trigger));

                    let other_player = if plr_move.target == TargetPlayer::Player1 {
                        TargetPlayer::Player2
                    } else {
                        TargetPlayer::Player1
                    };

                    let _ = self.send_packet_queue.send((
                        other_player,
                        DRSPacket::Move(plr_move),
                        DeliveryType::Unreliable,
                    ));
                }
                _ => (),
            }
        }

        while let Ok(msg) = self.sync_message_queue.try_recv() {
            match msg {
                SyncMessage::SyncStageToPlayer(target) => {
                    self.sync_transfer_stage(state, game_scene, Some(target));
                    self.sync_flags(state, game_scene, Some(target));
                    self.sync_players(state, game_scene);
                }
                SyncMessage::SyncNewPlayer(target) => {
                    if target == TargetPlayer::Player2 {
                        game_scene.add_player2(state);
                    }

                    {
                        let player = if target == TargetPlayer::Player1 {
                            &mut game_scene.player1
                        } else {
                            &mut game_scene.player2
                        };
                        player.cond.set_alive(true);
                    }

                    self.sync_players(state, game_scene);
                }
                SyncMessage::PlayerLeft(target) => {
                    {
                        let player = if target == TargetPlayer::Player1 {
                            &mut game_scene.player1
                        } else {
                            &mut game_scene.player2
                        };

                        player.cond.set_alive(false);
                    }

                    self.sync_players(state, game_scene);
                }
            }
        }

        if self.tick % 300 == 50 {
            for npc in game_scene.npc_list.iter_alive() {
                let _ = self.broadcast_packet_queue.send((DRSPacket::SyncNPC(npc.clone()), DeliveryType::Unreliable));
            }
        }

        if self.tick % 10 == 0 {
            self.sync_tsc(state, game_scene, None);
        }
    }

    pub fn sync_players(&mut self, state: &mut SharedGameState, game_scene: &mut GameScene) {
        let players = [TargetPlayer::Player1, TargetPlayer::Player2];
        for target in players {
            let player =
                if target == TargetPlayer::Player1 { &mut game_scene.player1 } else { &mut game_scene.player2 };
            let inventory =
                if target == TargetPlayer::Player1 { &mut game_scene.inventory_player1 } else { &mut game_scene.inventory_player2 };

            let (state, old_state, trigger) = player.controller.dump_state();

            let sync_packet = DRSPacket::SyncPlayer(PlayerData {
                target,
                life: player.life,
                max_life: player.max_life,
                control_mode: player.control_mode,
                question: player.question,
                popup: player.popup,
                shock_counter: player.shock_counter,
                xp_counter: player.xp_counter,
                current_weapon: player.current_weapon,
                stars: player.stars,
                damage: player.damage,
                air_counter: player.air_counter,
                air: player.air,
            });

            let move_packet = DRSPacket::Move(PlayerMove {
                target,
                x: player.x,
                y: player.y,
                vel_x: player.vel_x,
                vel_y: player.vel_y,
                state,
                old_state,
                trigger,
                direction: player.direction,
                cond: player.cond,
            });

            let inventory_packet = DRSPacket::SyncInventory(target, inventory.clone());

            let _ = self.broadcast_packet_queue.send((sync_packet, DeliveryType::Reliable));
            let _ = self.broadcast_packet_queue.send((move_packet, DeliveryType::Reliable));
            let _ = self.broadcast_packet_queue.send((inventory_packet, DeliveryType::Reliable));
        }
    }

    pub fn sync_tsc(&mut self, state: &mut SharedGameState, game_scene: &mut GameScene, target: Option<TargetPlayer>) {
        let tsc_packet = DRSPacket::SyncTSC(TextScriptData {
            state: state.textscript_vm.state,
            stack: state.textscript_vm.stack.clone(),
            flags: state.textscript_vm.flags,
            mode: state.textscript_vm.mode,
            executor_player: state.textscript_vm.executor_player,
            strict_mode: state.textscript_vm.strict_mode,
            suspend: state.textscript_vm.suspend,
            reset_invincibility: state.textscript_vm.reset_invincibility,
            numbers: state.textscript_vm.numbers,
            face: state.textscript_vm.face,
            item: state.textscript_vm.item,
            current_line: state.textscript_vm.current_line,
            line_1: state.textscript_vm.line_1.clone(),
            line_2: state.textscript_vm.line_2.clone(),
            line_3: state.textscript_vm.line_3.clone(),
            current_illustration: state.textscript_vm.current_illustration.clone(),
            illustration_state: state.textscript_vm.illustration_state.clone(),
            prev_char: state.textscript_vm.prev_char,
            fade_state: state.fade_state,
            prev_song: state.sound_manager.prev_song() as u32,
            current_song: state.sound_manager.current_song() as u32,
        });

        if let Some(target) = target {
            let _ = self.send_packet_queue.send((target, tsc_packet, DeliveryType::Unreliable));
        } else {
            let _ = self.broadcast_packet_queue.send((tsc_packet, DeliveryType::Unreliable));
        }
    }

    pub fn sync_transfer_stage(
        &mut self,
        state: &mut SharedGameState,
        game_scene: &mut GameScene,
        target: Option<TargetPlayer>,
    ) {
        let stage_packet = DRSPacket::SyncStageData(StageData {
            stage_id: game_scene.stage_id as u32,
            stage: game_scene.stage.clone(),
            player_pos: (game_scene.player1.x, game_scene.player1.y),
        });

        let tsc_packet = DRSPacket::SyncTSCScripts(state.textscript_vm.scripts.deref().borrow().clone());

        let ctrlf_packet = DRSPacket::SyncControlFlags(state.control_flags);

        if let Some(target) = target {
            let _ = self.send_packet_queue.send((target, stage_packet, DeliveryType::Reliable));
            let _ = self.send_packet_queue.send((target, tsc_packet, DeliveryType::Reliable));
            let _ = self.send_packet_queue.send((target, ctrlf_packet, DeliveryType::Reliable));

            for npc in game_scene.npc_list.iter_alive() {
                let _ = self.send_packet_queue.send((target, DRSPacket::SyncNPC(npc.clone()), DeliveryType::Reliable));
            }
        } else {
            let _ = self.broadcast_packet_queue.send((stage_packet, DeliveryType::Reliable));
            let _ = self.broadcast_packet_queue.send((tsc_packet, DeliveryType::Reliable));
            let _ = self.broadcast_packet_queue.send((ctrlf_packet, DeliveryType::Reliable));

            for npc in game_scene.npc_list.iter_alive() {
                let _ = self.broadcast_packet_queue.send((DRSPacket::SyncNPC(npc.clone()), DeliveryType::Reliable));
            }
        }

        self.sync_tsc(state, game_scene, target);
    }

    pub fn sync_flags(
        &mut self,
        state: &mut SharedGameState,
        game_scene: &mut GameScene,
        target: Option<TargetPlayer>,
    ) {
        let flags = GameProfile::dump(state, game_scene).flags;

        if let Some(target) = target {
            let _ = self.send_packet_queue.send((target, DRSPacket::SyncFlags(flags), DeliveryType::Reliable));
        } else {
            let _ = self.broadcast_packet_queue.send((DRSPacket::SyncFlags(flags), DeliveryType::Reliable));
        }
    }
}
