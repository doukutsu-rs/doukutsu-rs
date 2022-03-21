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

use crate::netplay::future::RSFuture;
use crate::netplay::protocol::{DRSPacket, ServerInfo};
use crate::{GameError, GameResult};

pub struct Client {
    master_addr: SocketAddr,
    peers: Vec<SocketAddr>,
    thread: JoinHandle<()>,
    sender: Sender<Packet>,
    state: ConnectionState,
    server_info_tasks: Arc<Mutex<VecDeque<(Instant, Sender<GameResult<ServerInfo>>)>>>,
}

pub enum ConnectionState {
    Connecting,
    Connected,
    Failed,
}

pub struct ServerInfoFuture(RefCell<Option<GameResult<ServerInfo>>>, RefCell<Option<Receiver<GameResult<ServerInfo>>>>);

impl RSFuture for ServerInfoFuture {
    type Output = GameResult<ServerInfo>;

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

impl Client {
    pub fn new(ip: &str) -> GameResult<Client> {
        let mut socket = Socket::bind_any()?;
        let cl_sender = socket.get_packet_sender();
        let sender = cl_sender.clone();
        let cl_server_info_tasks: Arc<Mutex<VecDeque<(Instant, Sender<GameResult<ServerInfo>>)>>> =
            Arc::new(Mutex::new(VecDeque::new()));
        let server_info_tasks = cl_server_info_tasks.clone();
        let addr = SocketAddr::from_str(ip)?;

        let thread = thread::spawn(move || {
            let receiver = socket.get_event_receiver();

            loop {
                socket.manual_poll(Instant::now());
                sleep(Duration::from_millis(1));

                while let Ok(packet) = receiver.try_recv() {
                    if let SocketEvent::Packet(p) = packet {
                        if let Ok(p) = DRSPacket::decode(p.payload()) {
                            match p {
                                DRSPacket::ServerInfoResponse(info) => {
                                    let mut deque = server_info_tasks.deref().lock().unwrap();
                                    while let Some((_, tx)) = deque.pop_front() {
                                        let _ = tx.send(Ok(info.clone()));
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }

                let mut deque = server_info_tasks.deref().lock().unwrap();
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
        });

        Ok(Client {
            master_addr: addr,
            peers: Vec::new(),
            thread,
            sender: cl_sender,
            state: ConnectionState::Connecting,
            server_info_tasks: cl_server_info_tasks,
        })
    }

    pub fn get_server_info(&mut self) -> ServerInfoFuture {
        let (tx, rx) = crossbeam_channel::bounded(1);

        {
            let mut tasks = self.server_info_tasks.lock().unwrap();
            tasks.push_back((Instant::now(), tx));
        }

        let _ = self
            .sender
            .send(Packet::reliable_unordered(self.master_addr, DRSPacket::ServerInfoRequest.encode_to_vec()));

        ServerInfoFuture(RefCell::new(None), RefCell::new(Some(rx)))
    }
}
