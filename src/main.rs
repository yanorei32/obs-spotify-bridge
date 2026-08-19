#![warn(clippy::pedantic)]

mod model;
mod obsdriver;

use clap::Parser;
use itertools::Itertools;
use spotnowplay::*;
use tokio::sync::Mutex;

struct App {
    obs_config: model::ObsConfig,
    previous_notified: Mutex<Option<model::Music>>,
}

#[async_trait::async_trait]
impl EventHandler for App {
    async fn on_playback_state_update(&self, player_state: Option<&api::PlaybackState>) {
        let music = player_state
            .map(|s| {
                if s.is_playing {
                    let title = match s.item.as_ref().unwrap() {
                        api::Item::Track(track) => &track.name,
                        api::Item::Episode(episode) => &episode.name,
                    }
                    .to_string();

                    let artists = match s.item.as_ref().unwrap() {
                        api::Item::Track(track) => track
                            .artists
                            .iter()
                            .map(|a| a.name.as_str())
                            .intersperse(", ")
                            .collect::<String>(),
                        api::Item::Episode(_) => "n/a".to_string(),
                    };

                    let albumart = match s.item.as_ref().unwrap() {
                        api::Item::Track(track) => &track.album.images,
                        api::Item::Episode(episode) => &episode.images,
                    }
                    .iter()
                    .max_by_key(|image| image.width.unwrap_or(0) * image.height.unwrap_or(0))
                    .map(|image| image.url.to_string())
                    .unwrap_or("about:blank".to_string());

                    Some(model::Music {
                        title,
                        artists,
                        albumart,
                    })
                } else {
                    None
                }
            })
            .flatten();

        let mut previous_notified = self.previous_notified.lock().await;

        if music == *previous_notified {
            return;
        }

        *previous_notified = music.clone();

        tracing::info!("{music:?}");

        let result = obsdriver::update(&self.obs_config, music.as_ref()).await;

        if let Err(e) = result {
            tracing::warn!("{e:?}");
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt().with_ansi(cfg!(target_family = "unix")).init();

    let config = model::Config::parse();

    loop {
        let token = discord_integration::get_available_spotify_token(&config.discord_token)
            .await
            .unwrap();

        let client = ClientBuilder::new(&token.access_token)
            .handler(App {
                obs_config: config.obs_config.clone(),
                previous_notified: Mutex::new(None),
            })
            .build();

        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            err = client.run() => {
                match err {
                    Err(RunError::WebSocketError(tungstenite::error::Error::Io(err)))
                        if err.kind() == std::io::ErrorKind::TimedOut =>
                    {
                        tracing::info!("Spotify WebSocket TimedOut detected. Restarting...");
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                        continue;
                    }
                    _ => break,
                }
            }
        }
    }

    let _ = obsdriver::update(&config.obs_config, None).await;
}
