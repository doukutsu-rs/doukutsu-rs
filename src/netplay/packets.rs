#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "i")]
pub enum DRSPacket {
    #[serde(rename = "\x01")]
    Ping(u16),
    #[serde(rename = "\x02")]
    Pong(u16),
}
