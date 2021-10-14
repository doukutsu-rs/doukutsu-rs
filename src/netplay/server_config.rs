#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerConfiguration {
    #[serde(default = "default_bind")]
    pub bind_to: String,
}

// 'RS' = 0x5253 = 21075
fn default_bind() -> String {
    "0.0.0.0:21075".to_string()
}
