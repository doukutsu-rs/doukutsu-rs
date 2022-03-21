use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, Instant};

use laminar::{Packet, Socket, SocketEvent};

use crate::framework::error::GameResult;
use crate::netplay::protocol::{DRSPacket, ServerInfo};
use crate::netplay::server_config::ServerConfiguration;
use crate::SharedGameState;

pub struct Server {
    thread: JoinHandle<()>,
}

impl Server {
    pub fn start(config: ServerConfiguration) -> GameResult<Server> {
        let mut socket = Socket::bind(config.bind_to)?;
        log::info!("Listening on {:?}.", socket.local_addr()?);

        let thread = thread::spawn(move || {
            let receiver = socket.get_event_receiver();
            let sender = socket.get_packet_sender();

            loop {
                socket.manual_poll(Instant::now());
                sleep(Duration::from_millis(1));

                while let Ok(packet) = receiver.try_recv() {
                    match packet {
                        SocketEvent::Packet(p) => {
                            if let Ok(payload) = DRSPacket::decode(p.payload()) {
                                match payload {
                                    DRSPacket::ServerInfoRequest => {
                                        let _ = sender.send(Packet::reliable_sequenced(
                                            p.addr(),
                                            DRSPacket::ServerInfoResponse(ServerInfo {
                                                motd: "A doukutsu-rs server".to_string(),
                                                players: (21, 37),
                                            })
                                            .encode_to_vec(),
                                            Some(0),
                                        ));
                                    }
                                    _ => (),
                                }
                            }
                        }
                        SocketEvent::Connect(addr) => {
                            log::info!("Client {:?} connected.", addr);
                        }
                        SocketEvent::Timeout(addr) => {
                            log::info!("Client {:?} timed out.", addr);
                        }
                        SocketEvent::Disconnect(addr) => {
                            log::info!("Client {:?} disconnected.", addr);
                        }
                    }
                }
            }
        });

        Ok(Server { thread })
    }

    pub fn process(&mut self, state: &mut SharedGameState) {}
}
