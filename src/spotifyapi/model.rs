use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum Request {
    Ping,
}

#[derive(Debug, Deserialize)]
pub struct Image {
    pub width: i32,
    pub height: i32,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Album {
    pub images: Vec<Image>,
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum AlbumLikeObject {
    Album(Album),
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum ArtistLikeObject {
    Artist(Artist),
}

#[derive(Debug, Deserialize)]
pub struct Track {
    pub name: String,
    pub artists: Vec<ArtistLikeObject>,
    pub album: AlbumLikeObject,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum PSCItem {
    Track(Track),
}

#[derive(Debug, Deserialize)]
pub struct PSCState {
    pub item: Option<PSCItem>,
    pub is_playing: bool,
}

#[derive(Debug, Deserialize)]
pub struct PSCEvent {
    pub state: PSCState,
}

#[derive(Debug, Deserialize)]
pub struct PlayerStateChanged {
    pub event: PSCEvent,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "type")]
pub enum Event {
    PlayerStateChanged(PlayerStateChanged),
}

#[derive(Debug, Deserialize)]
pub struct Payload {
    pub events: Vec<Event>,
}

#[derive(Debug, Deserialize)]
pub struct PutHeader {
    #[serde(rename = "Spotify-Connection-Id")]
    pub spotify_connection_id: String,
}

#[derive(Debug, Deserialize)]
pub struct WssEvent {
    pub payloads: Vec<Payload>,
}

#[derive(Debug, Deserialize)]
pub struct Put {
    pub headers: PutHeader,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MessageLikeObjects {
    WssEvent(WssEvent),
    Put(Put),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum Response {
    Message(MessageLikeObjects),
    Pong,
}
