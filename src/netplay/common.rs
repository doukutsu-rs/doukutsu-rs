use std::net::SocketAddr;
use std::time::Duration;

use crossbeam_channel::Sender;
use laminar::{Config, Packet};

use crate::netplay::protocol::DRSPacket;

pub fn make_socket_config() -> Config {
    let mut config = Config::default();
    config.idle_connection_timeout = Duration::new(30, 0);
    config
}

pub trait SenderExt {
    fn kick(&self, addr: SocketAddr, reason: &'static str) {
        self.send_reliable(addr, DRSPacket::Kicked(reason.to_owned()));
    }

    fn send_reliable(&self, addr: SocketAddr, packet: DRSPacket);

    fn send_unreliable(&self, addr: SocketAddr, packet: DRSPacket);

    fn broadcast_reliable<I>(&self, addrs: I, packet: DRSPacket) where I: Iterator<Item = SocketAddr>;

    fn broadcast_unreliable<I>(&self, addrs: I, packet: DRSPacket) where I: Iterator<Item = SocketAddr>;
}

impl SenderExt for Sender<Packet> {
    fn send_reliable(&self, addr: SocketAddr, packet: DRSPacket) {
        let _ = self.send(Packet::reliable_sequenced(addr, packet.encode_to_vec(), Some(0)));
    }

    fn send_unreliable(&self, addr: SocketAddr, packet: DRSPacket) {
        let _ = self.send(Packet::unreliable_sequenced(addr, packet.encode_to_vec(), Some(0)));
    }

    fn broadcast_reliable<I>(&self, addrs: I, packet: DRSPacket) where I: Iterator<Item=SocketAddr> {
        let payload = packet.encode_to_vec();
        for addr in addrs {
            let _ = self.send(Packet::reliable_sequenced(addr, payload.clone(), Some(0)));
        }
    }

    fn broadcast_unreliable<I>(&self, addrs: I, packet: DRSPacket) where I: Iterator<Item=SocketAddr> {
        let payload = packet.encode_to_vec();
        for addr in addrs {
            let _ = self.send(Packet::unreliable_sequenced(addr, payload.clone(), Some(0)));
        }
    }
}
