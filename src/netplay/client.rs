use std::cell::{Ref, RefCell};
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use laminar::{Packet, Socket, SocketEvent};

use crate::input::dummy_player_controller::DummyPlayerController;
use crate::netplay::common::{make_socket_config, SenderExt};
use crate::netplay::future::RSFuture;
use crate::netplay::protocol::{DRSPacket, PlayerInfo, PlayerMove, ServerInfo};
use crate::player::{Player, TargetPlayer};
use crate::scene::game_scene::GameScene;
use crate::scripting::tsc::text_script::TextScriptLine;
use crate::{Context, GameError, GameResult, SharedGameState};

pub struct FutureStruct<T>(RefCell<Option<GameResult<T>>>, RefCell<Option<Receiver<GameResult<T>>>>);

impl<T> RSFuture for FutureStruct<T> {
    type Output = GameResult<T>;

    fn poll(&self) -> Option<Ref<Self::Output>> {
        let mut destroy = false;
        if let Some(chan) = self.1.borrow_mut().as_mut() {
            if let Ok(x) = chan.try_recv() {
                self.0.replace(Some(x));
                destroy = true;
            }
        }

        if destroy {
            self.1.replace(None);
        }

        let val = self.0.borrow();
        if val.is_some() {
            Some(Ref::map(val, |v| v.as_ref().unwrap()))
        } else {
            None
        }
    }
}

pub struct FutureTasks<T>(Arc<Mutex<VecDeque<(Instant, Sender<GameResult<T>>)>>>);

impl<T> FutureTasks<T> {
    pub fn new() -> FutureTasks<T> {
        FutureTasks(Arc::new(Mutex::new(VecDeque::new())))
    }

    pub fn process_timeout(&self) {
        let mut deque = self.0.deref().lock().unwrap();
        let n = deque.len();
        for _ in 0..n {
            if let Some((inst, tx)) = deque.pop_front() {
                if inst.elapsed().as_secs() > 30 {
                    let _ = tx.send(Err(GameError::NetworkError("Timed out.".to_owned())));
                    continue;
                }

                deque.push_back((inst, tx));
            }
        }
    }
}

impl<T> Clone for FutureTasks<T> {
    fn clone(&self) -> Self {
        FutureTasks(self.0.clone())
    }
}

pub struct Client {
    master_addr: SocketAddr,
    peers: Arc<Mutex<Vec<SocketAddr>>>,
    thread: JoinHandle<()>,
    sender: Sender<Packet>,
    game_packet_queue: Receiver<DRSPacket>,
    control_packet_queue: Sender<DRSPacket>,
    state: Arc<Mutex<ConnectionState>>,
    server_info_tasks: FutureTasks<ServerInfo>,
    join_tasks: FutureTasks<()>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ConnectionState {
    PreHello,
    Connecting,
    Connected,
    Failed,
}

// this code is a horror already but I'm too lazy to scrap it and rewrite

impl Client {
    pub fn new(ip: &str) -> GameResult<Client> {
        let mut socket = Socket::bind_any_with_config(make_socket_config())?;

        let (pq_tx, pq_rx) = crossbeam_channel::bounded(512);
        let (cq_tx, cq_rx) = crossbeam_channel::bounded(8);
        let cl_sender = socket.get_packet_sender();
        let cl_server_info_tasks = FutureTasks::new();
        let cl_join_tasks = FutureTasks::new();

        let cl_peers = Arc::new(Mutex::new(Vec::new()));
        let cl_state = Arc::new(Mutex::new(ConnectionState::PreHello));

        let sender = cl_sender.clone();
        let server_info_tasks = cl_server_info_tasks.clone();
        let join_tasks = cl_join_tasks.clone();
        let peers = cl_peers.clone();
        let state = cl_state.clone();

        let addr = SocketAddr::from_str(ip)?;
        cl_sender.send_reliable(addr, DRSPacket::KeepAlive);

        let thread = thread::spawn(move || {
            let receiver = socket.get_event_receiver();
            let mut challenge = [0u8; 32];
            let mut last_keepalive = Instant::now();

            loop {
                socket.manual_poll(Instant::now());
                sleep(Duration::from_millis(1));

                let now = Instant::now();
                if last_keepalive.duration_since(now).as_secs() > 5 {
                    sender.send_reliable(addr, DRSPacket::KeepAlive);
                    last_keepalive = now;
                }

                let mut state_locked = state.lock().unwrap();
                while let Ok(packet) = receiver.try_recv() {
                    if let SocketEvent::Packet(p) = packet {
                        match DRSPacket::decode(p.payload()) {
                            Ok(p) => match *state_locked {
                                ConnectionState::PreHello => {
                                    if let DRSPacket::Hello(data) = p {
                                        challenge.copy_from_slice(&data.challenge);
                                        log::info!("Received challenge: {:?}", challenge);
                                    }

                                    *state_locked = ConnectionState::Connecting;
                                }
                                ConnectionState::Connecting => match p {
                                    DRSPacket::ServerInfoResponse(info) => {
                                        let mut deque = server_info_tasks.0.deref().lock().unwrap();
                                        while let Some((_, tx)) = deque.pop_front() {
                                            let _ = tx.send(Ok(info.clone()));
                                        }
                                    }
                                    DRSPacket::ConnectResponse(_) => {
                                        let mut deque = join_tasks.0.deref().lock().unwrap();
                                        while let Some((_, tx)) = deque.pop_front() {
                                            let _ = tx.send(Ok(()));
                                        }

                                        let _ = pq_tx.send(p);
                                        *state_locked = ConnectionState::Connected;
                                    }
                                    DRSPacket::Kicked(reason) => {
                                        log::info!("Kicked from the server: {}", reason);
                                        return;
                                    }
                                    _ => (),
                                },
                                ConnectionState::Connected => {
                                    let _ = pq_tx.send(p);

                                    while let Ok(packet) = cq_rx.try_recv() {
                                        sender.send_reliable(addr, packet);
                                    }
                                }
                                ConnectionState::Failed => (),
                            },
                            Err(e) => {
                                log::error!("Failed to decode a packet: {}", e);
                            }
                        }
                    }
                }

                if *state_locked != ConnectionState::PreHello {
                    while let Ok(packet) = cq_rx.try_recv() {
                        sender.send_reliable(addr, packet);
                    }
                }

                server_info_tasks.process_timeout();
                join_tasks.process_timeout();
            }
        });

        Ok(Client {
            master_addr: addr,
            peers: cl_peers,
            thread,
            sender: cl_sender,
            game_packet_queue: pq_rx,
            control_packet_queue: cq_tx,
            state: cl_state,
            server_info_tasks: cl_server_info_tasks,
            join_tasks: cl_join_tasks,
        })
    }

    pub fn fetch_server_info(&mut self) -> FutureStruct<ServerInfo> {
        let (tx, rx) = crossbeam_channel::bounded(1);

        {
            let mut tasks = self.server_info_tasks.0.lock().unwrap();
            tasks.push_back((Instant::now(), tx));
        }

        log::info!("Fetch server info...");
        let _ = self.control_packet_queue.send(DRSPacket::ServerInfoRequest);

        FutureStruct(RefCell::new(None), RefCell::new(Some(rx)))
    }

    pub fn join(&mut self, player_name: String) -> FutureStruct<()> {
        let (tx, rx) = crossbeam_channel::bounded(1);

        {
            let mut tasks = self.join_tasks.0.lock().unwrap();
            tasks.push_back((Instant::now(), tx));
        }

        let player_info = PlayerInfo { name: player_name, public_key: [0u8; 32], challenge_signature: [0u8; 64] };

        log::info!("Join...");
        let _ = self.control_packet_queue.send(DRSPacket::Connect(player_info));

        FutureStruct(RefCell::new(None), RefCell::new(Some(rx)))
    }

    pub fn get_state(&self) -> ConnectionState {
        *self.state.lock().unwrap()
    }

    pub fn process(&mut self, state: &mut SharedGameState, game_scene: &mut GameScene, ctx: &mut Context) {
        let mut skip_move = false;

        while let Ok(packet) = self.game_packet_queue.try_recv() {
            match packet {
                DRSPacket::ConnectResponse(target) => {
                    log::info!("Connected, local player = {}", target.index());
                    game_scene.player1.controller = Box::new(DummyPlayerController::new());
                    game_scene.player2.controller = Box::new(DummyPlayerController::new());

                    let player =
                        if target == TargetPlayer::Player1 { &mut game_scene.player1 } else { &mut game_scene.player2 };

                    game_scene.local_player = target;
                    player.cond.set_alive(true);
                    player.controller = state.settings.create_player1_controller();
                }
                DRSPacket::Kicked(reason) => {
                    log::info!("Kicked from the server: {}", reason);
                    return;
                }
                DRSPacket::ChatMessage(message) => {
                    log::info!("Chat: {}", message);
                    let chat = state.chat.clone();
                    chat.borrow_mut().push_message(message);
                }
                DRSPacket::SyncStageData(data) => {
                    let mut new_scene = GameScene::from_stage(state, ctx, data.stage, data.stage_id as usize).unwrap();
                    let (pos_x, pos_y) = data.player_pos;

                    new_scene.inventory_player1 = game_scene.inventory_player1.clone();
                    new_scene.inventory_player2 = game_scene.inventory_player2.clone();
                    new_scene.player1 = game_scene.player1.clone();
                    new_scene.player1.vel_x = 0;
                    new_scene.player1.vel_y = 0;
                    new_scene.player1.x = pos_x;
                    new_scene.player1.y = pos_y;
                    new_scene.player2 = game_scene.player2.clone();
                    new_scene.player2.vel_x = 0;
                    new_scene.player2.vel_y = 0;
                    new_scene.player2.x = pos_x;
                    new_scene.player2.y = pos_y;
                    // Reset player interaction flag upon TRA
                    new_scene.player1.cond.set_interacted(false);
                    new_scene.player2.cond.set_interacted(false);
                    // Reset ground collision for WAS / WaitStanding
                    new_scene.player1.flags.set_hit_bottom_wall(false);
                    new_scene.player2.flags.set_hit_bottom_wall(false);
                    new_scene.frame.wait = game_scene.frame.wait;
                    new_scene.nikumaru = game_scene.nikumaru;
                    new_scene.local_player = game_scene.local_player;

                    let skip = state.textscript_vm.flags.cutscene_skip();
                    state.control_flags.set_tick_world(true);
                    state.control_flags.set_interactions_disabled(true);
                    state.textscript_vm.flags.0 = 0;
                    state.textscript_vm.flags.set_cutscene_skip(skip);
                    state.textscript_vm.face = 0;
                    state.textscript_vm.item = 0;
                    state.textscript_vm.current_line = TextScriptLine::Line1;
                    state.textscript_vm.line_1.clear();
                    state.textscript_vm.line_2.clear();
                    state.textscript_vm.line_3.clear();
                    state.textscript_vm.suspend = true;
                    state.next_scene = Some(Box::new(new_scene));

                    return; // process remaining packets on new stage
                }
                DRSPacket::SyncTSCScripts(scripts) => {
                    state.textscript_vm.scripts.replace(scripts);
                }
                DRSPacket::SyncTSC(data) => {
                    state.textscript_vm.state = data.state;
                    state.textscript_vm.stack = data.stack;
                    state.textscript_vm.flags = data.flags;
                    state.textscript_vm.mode = data.mode;
                    state.textscript_vm.executor_player = data.executor_player;
                    state.textscript_vm.strict_mode = data.strict_mode;
                    state.textscript_vm.suspend = data.suspend;
                    state.textscript_vm.reset_invincibility = data.reset_invincibility;
                    state.textscript_vm.numbers = data.numbers;
                    state.textscript_vm.face = data.face;
                    state.textscript_vm.item = data.item;
                    state.textscript_vm.current_line = data.current_line;
                    state.textscript_vm.line_1 = data.line_1;
                    state.textscript_vm.line_2 = data.line_2;
                    state.textscript_vm.line_3 = data.line_3;
                    state.textscript_vm.current_illustration = data.current_illustration;
                    state.textscript_vm.illustration_state = data.illustration_state;
                    state.textscript_vm.prev_char = data.prev_char;
                    state.fade_state = data.fade_state;

                    let _ = state.sound_manager.play_song(
                        data.current_song as usize,
                        &state.constants,
                        &state.settings,
                        ctx,
                    );
                }
                DRSPacket::SyncNPC(npc) => {
                    if let Some(npc_ref) = game_scene.npc_list.get_npc(npc.id as usize) {
                        *npc_ref = npc;
                    }
                }
                DRSPacket::SyncControlFlags(flags) => {
                    state.control_flags = flags;
                }
                DRSPacket::SyncPlayer(data) => {
                    let player = if data.target == TargetPlayer::Player1 {
                        &mut game_scene.player1
                    } else {
                        &mut game_scene.player2
                    };

                    player.life = data.life;
                    player.max_life = data.max_life;
                    player.control_mode = data.control_mode;
                    player.question = data.question;
                    player.popup = data.popup;
                    player.shock_counter = data.shock_counter;
                    player.xp_counter = data.xp_counter;
                    player.current_weapon = data.current_weapon;
                    player.stars = data.stars;
                    player.damage = data.damage;
                    player.air_counter = data.air_counter;
                    player.air = data.air;
                }
                DRSPacket::Move(data) => {
                    let player = if data.target == TargetPlayer::Player1 {
                        &mut game_scene.player1
                    } else {
                        &mut game_scene.player2
                    };

                    player.x = data.x;
                    player.y = data.y;
                    player.vel_x = data.vel_x;
                    player.vel_y = data.vel_y;
                    player.direction = data.direction;
                    player.cond = data.cond;

                    if data.target == game_scene.local_player {
                        skip_move = true;
                    } else {
                        player.controller.set_state((data.state, data.old_state, data.trigger));
                    }
                }
                _ => (),
            }
        }

        if !skip_move {
            let p = if game_scene.local_player == TargetPlayer::Player1 {
                &game_scene.player1
            } else {
                &game_scene.player2
            };
            let (state, old_state, trigger) = p.controller.dump_state();
            let Player { x, y, vel_x, vel_y, direction, cond, .. } = *p;

            self.sender.send_unreliable(
                self.master_addr,
                DRSPacket::Move(PlayerMove {
                    target: game_scene.local_player, // ignored
                    x,
                    y,
                    vel_x,
                    vel_y,
                    state,
                    old_state,
                    trigger,
                    direction,
                    cond,
                }),
            );
        }
    }
}
