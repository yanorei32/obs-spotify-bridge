use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) discord_token: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct Music {
    pub(crate) title: String,
    pub(crate) artists: String,
    pub(crate) albumart: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "t", content = "c")]
pub(crate) enum Notify {
    Playing(Music),
    Paused,
}
