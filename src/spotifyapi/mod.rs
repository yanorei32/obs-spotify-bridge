use crate::notify_model::{Music, Notify, Sender};
use anyhow::{bail, Context, Result};
use futures_util::{SinkExt, StreamExt};
use itertools::Itertools;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

pub mod model;

pub async fn is_available_token(token: &str) -> Result<()> {
    let resp = reqwest::Client::new()
        .get("https://api.spotify.com/v1/me/player/devices")
        .bearer_auth(token)
        .send()
        .await
        .with_context(|| "not available")?;

    if !resp.status().is_success() {
        bail!("Maybe unauthorized")
    }

    Ok(())
}

pub async fn connect_ws(token: &str, sender: Sender) -> Result<()> {
    let url = format!("wss://dealer.spotify.com/?access_token={}", token);
    let url = Url::parse(&url).with_context(|| "Failed to parse URL, Invalid token?")?;

    let (ws, _) = connect_async(url)
        .await
        .with_context(|| "Failed to connect Spotify")?;

    let (mut tx, mut rx) = ws.split();
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    let mut playing = model::PlayingState::Paused;

    loop {
        tokio::select! {
            msg = rx.next() => {
                let msg = match msg {
                    Some(msg) => msg,
                    None => continue,
                };

                let msg_str = match msg.with_context(|| "Failed to read message")? {
                    Message::Text(v) => v,
                    Message::Close(v) => bail!("Unexpected close by server: {:?}", v),
                    _ => continue,
                };

                let msg = match serde_json::from_str::<model::Response>(&msg_str) {
                    Ok(v) => v,
                    Err(_e) => continue,
                };

                let msg = match msg {
                    model::Response::Message(v) => v,
                    model::Response::Pong => continue,
                };

                match msg {
                    model::MessageLikeObjects::Put(v) => {
                        reqwest::Client::new().put("https://api.spotify.com/v1/me/notifications/player")
                            .query(&[("connection_id", v.headers.spotify_connection_id)])
                            .bearer_auth(token)
                            .header(reqwest::header::CONTENT_LENGTH, "0")
                            .send()
                            .await
                            .with_context(|| "Failed to activate Spotify-Connection")?;
                    },
                    model::MessageLikeObjects::WssEvent(v) => {
                        let v = v.payloads.first().and_then(|v| v.events.first());

                        let v = match v {
                            Some(v) => v,
                            _ => continue,
                        };

                        let v = match v {
                            model::Event::DeviceStateChanged(v) => {
                                if let model::PlayingState::Playing(id) = &playing {
                                    // active player is die
                                    if !v.event.devices.iter().any(|d| &d.id == id) {
                                        sender
                                            .send(Notify::Paused)
                                            .with_context(|| "Failed to send paused message")?;

                                        playing = model::PlayingState::Paused;
                                    }
                                }

                                continue;
                            },
                            model::Event::PlayerStateChanged(v) => v
                        };

                        if !v.event.state.is_playing {
                            sender
                                .send(Notify::Paused)
                                .with_context(|| "Failed to send paused message")?;

                            playing = model::PlayingState::Paused;
                            continue;
                        }

                        let track = match &v.event.state.item {
                            Some(model::PSCItem::Track(x)) => x,
                            None => {
                                sender
                                    .send(Notify::Unknown)
                                    .with_context(|| "Failed to send unknown message")?;

                                playing = model::PlayingState::Paused;
                                continue;
                            }
                        };

                        #[allow(unstable_name_collisions)]
                        let artists: String = track
                            .artists
                            .iter()
                            .map(|v| match v {
                                model::ArtistLikeObject::Artist(v) => v.name.as_str(),
                            })
                            .intersperse(", ")
                            .collect();

                        let album = match &track.album {
                            model::AlbumLikeObject::Album(a) => a,
                            model::AlbumLikeObject::Other => continue,
                        };

                        let albumart: &str = album
                            .images
                            .iter()
                            .max_by_key(|v| v.width * v.height)
                            .map(|v| v.url.as_str())
                            .unwrap();


                        let p = Notify::Playing(Music {
                            title: track.name.clone(),
                            artists: artists.clone(),
                            albumart: albumart.to_string(),
                        });

                        playing = model::PlayingState::Playing(
                            v.event.state.device.id.clone()
                        );

                        sender.send(p).with_context(|| "Failed to send playing message")?;
                    },
                };

            }
            _ = interval.tick() => {
                tx.send(
                    Message::Text(serde_json::to_string(&model::Request::Ping).unwrap())
                )
                .await
                .with_context(|| "Failed to send ping")?;
            }
        }
    }
}
