use crate::GameResult;

#[derive(bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct ServerInfo {
    pub motd: String,
    // online/max
    pub players: (u16, u16),
}

#[derive(bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct PlayerInfo {
    pub name: String,
    pub public_key: String,
    pub signed_challenge: String,
}

#[derive(bincode::Encode, bincode::Decode, Clone, Debug)]
pub struct HelloData {
    pub challenge: String,
}

#[derive(bincode::Encode, bincode::Decode, Clone, Debug)]
pub enum DRSPacket {
    Hello(HelloData),
    ServerInfoRequest,
    ServerInfoResponse(ServerInfo),
}

impl DRSPacket {
    pub fn decode(data: &[u8]) -> GameResult<DRSPacket> {
        let (packet, _) = bincode::decode_from_slice(data, bincode::config::standard())?;
        Ok(packet)
    }

    pub fn encode(&self, data: &mut [u8]) -> GameResult {
        bincode::encode_into_slice(self, data, bincode::config::standard())?;
        Ok(())
    }

    pub fn encode_to_vec(&self) -> Vec<u8> {
        bincode::encode_to_vec(self,  bincode::config::standard()).unwrap()
    }
}
