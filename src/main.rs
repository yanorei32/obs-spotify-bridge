#![warn(clippy::pedantic, clippy::nursery)]

use crate::discordws::get_spotify_token;
use crate::spotifyws::fetch;
use itertools::Itertools;
use tokio::sync::{mpsc, watch};

mod discordws;
mod model;
mod spotifyws;
mod wsserver;

#[tokio::main]
async fn main() {
    let c = envy::from_env::<model::Config>().unwrap();

    println!("Get token by discord...");
    let token = get_spotify_token(&c.discord_token);

    let (notify_tx, notify_rx) = watch::channel(wsserver::model::Notify::Paused {});

    println!("Listen on: ws://{}", &c.bind_address);
    tokio::spawn(async move {
        wsserver::wsserver(&c.bind_address, &notify_rx).await;
    });

    let (response_tx, mut response_rx) = mpsc::channel::<spotifyws::model::WssEvent>(1);
    println!("Connect to Spotify with: {}", token);
    tokio::spawn(async move {
        fetch(&token, &response_tx).await;
    });

    loop {
        let v = match response_rx.recv().await {
            Some(v) => v,
            _ => continue,
        };

        let v = v.payloads.first().and_then(|v| v.events.first());

        let spotifyws::model::Event::PlayerStateChanged(v) = match v {
            Some(v) => v,
            _ => continue,
        };

        if !v.event.state.is_playing {
            println!("♪ Paused");

            notify_tx.send(wsserver::model::Notify::Paused).unwrap();

            continue;
        }

        let v = match &v.event.state.item {
            Some(spotifyws::model::PSCItem::Track(x)) => x,
            None => {
                println!("♪ Unknown");

                notify_tx.send(wsserver::model::Notify::Unknown {}).unwrap();

                continue;
            }
        };

        #[allow(unstable_name_collisions)]
        let artists: String = v
            .artists
            .iter()
            .map(|v| match v {
                spotifyws::model::ArtistLikeObject::Artist(v) => v.name.as_str(),
            })
            .intersperse(", ")
            .collect();

        let album = match &v.album {
            spotifyws::model::AlbumLikeObject::Album(a) => a,
            spotifyws::model::AlbumLikeObject::Other => return,
        };

        let albumart: &str = album
            .images
            .iter()
            .max_by_key(|v| v.width * v.height)
            .map(|v| v.url.as_str())
            .unwrap();

        notify_tx
            .send(wsserver::model::Notify::Playing(wsserver::model::Music {
                title: v.name.clone(),
                artists: artists.clone(),
                albumart: albumart.to_string(),
            }))
            .unwrap();

        println!("♪ {} {}: {}", v.name, artists, albumart);
    }
}
