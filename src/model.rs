use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub discord_token: String,

    #[serde(default = "default_obs_address")]
    pub obs_address: String,

    #[serde(default = "default_obs_port")]
    pub obs_port: u16,

    pub obs_password: Option<String>,
}

fn default_obs_address() -> String {
    "127.0.0.1".to_string()
}

fn default_obs_port() -> u16 {
    4455
}
