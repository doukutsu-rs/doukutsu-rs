#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerConfiguration {
    #[serde(default = "default_bind")]
    pub bind_to: String,
    #[serde(default = "default_max_players")]
    pub max_players: u16,
}

impl Default for ServerConfiguration {
    fn default() -> Self {
        ServerConfiguration { bind_to: default_bind(), max_players: default_max_players() }
    }
}

// 'RS' = 0x5253 = 21075
fn default_bind() -> String {
    "0.0.0.0:21075".to_string()
}

fn default_max_players() -> u16 {
    2
}
