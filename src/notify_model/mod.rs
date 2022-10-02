use serde::Serialize;
use tokio::sync::watch;

pub type Sender = watch::Sender<Notify>;
pub type Receiver = watch::Receiver<Notify>;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Music {
    pub title: String,
    pub artists: String,
    pub albumart: String,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(tag = "t", content = "c")]
pub enum Notify {
    Playing(Music),
    Unknown,
    Paused,
}
