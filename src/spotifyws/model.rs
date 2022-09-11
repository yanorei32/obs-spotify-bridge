use serde::{Deserialize, Serialize};

pub(crate) type EventCallback = fn(WssEvent);

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub(crate) enum Request {
    Ping,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Image {
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) url: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Album {
    pub(crate) images: Vec<Image>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Artist {
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub(crate) enum AlbumLikeObject {
    Album(Album),
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub(crate) enum ArtistLikeObject {
    Artist(Artist),
}

#[derive(Debug, Deserialize)]
pub(crate) struct Track {
    pub(crate) name: String,
    pub(crate) artists: Vec<ArtistLikeObject>,
    pub(crate) album: AlbumLikeObject,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub(crate) enum PSCItem {
    Track(Track),
}

#[derive(Debug, Deserialize)]
pub(crate) struct PSCState {
    pub(crate) item: Option<PSCItem>,
    pub(crate) is_playing: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PSCEvent {
    pub(crate) state: PSCState,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PlayerStateChanged {
    pub(crate) event: PSCEvent,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "type")]
pub(crate) enum Event {
    PlayerStateChanged(PlayerStateChanged),
}

#[derive(Debug, Deserialize)]
pub(crate) struct Payload {
    pub(crate) events: Vec<Event>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PutHeader {
    #[serde(rename = "Spotify-Connection-Id")]
    pub(crate) spotify_connection_id: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct WssEvent {
    pub(crate) payloads: Vec<Payload>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Put {
    pub(crate) headers: PutHeader,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum MessageLikeObjects {
    WssEvent(WssEvent),
    Put(Put),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub(crate) enum Response {
    Message(MessageLikeObjects),
    Pong,
}
