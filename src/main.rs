#![warn(clippy::pedantic)]
use crate::discordapi::{get_spotify_credentials, renew_spotify_token};
use crate::filter::duplicate_filter;
use crate::obsdriver::obsdriver;
use crate::spotifyapi::{connect_ws, is_available_token};
use crate::wsserver::serve;
use tokio::sync::watch;

mod discordapi;
mod filter;
mod model;
mod notify_model;
mod obsdriver;
mod spotifyapi;
mod wsserver;

#[tokio::main]
async fn main() {
    let c = envy::from_env::<model::Config>().unwrap();

    println!("Get Spotify credential by Discord WebSocket...");
    let cred = get_spotify_credentials(&c.discord_token).unwrap();

    let (tx, to_f) = watch::channel(notify_model::Notify::Paused {});
    let (from_f, mut rx) = watch::channel(notify_model::Notify::Paused {});

    let token = if is_available_token(&cred.access_token).await.is_ok() {
        cred.access_token.clone()
    } else {
        println!("Renew Spotify credential by Discord API...");
        renew_spotify_token(&c.discord_token, &cred.id)
            .await
            .unwrap()
    };

    let mut filter = tokio::spawn(async move { duplicate_filter(to_f, from_f).await });
    let mut spotify_ws = tokio::spawn(async move { connect_ws(&token, tx).await });

    let rx2 = rx.clone();
    let mut wsserver = tokio::spawn(async move { serve(&c.ws_bind_address, rx2).await });

    let rx3 = rx.clone();
    let mut obsdriver =
        tokio::spawn(
            async move { obsdriver(&c.obs_address, c.obs_port, c.obs_password, rx3).await },
        );

    println!("Entering Main Loop...");

    loop {
        tokio::select! {
            v = &mut spotify_ws => {
                panic!("Unexpected Shutdown SpotifyWS: {:?}", v);
            },
            v = &mut wsserver => {
                panic!("Unexpected Shutdown WSServer: {:?}", v);
            },
            v = &mut filter => {
                panic!("Unexpected Shutdown Filter: {:?}", v);
            },
            v = &mut obsdriver => {
                panic!("Unexpected Shutdown OBSDriver: {:?}", v);
            },
            changed = rx.changed() => {
                changed.expect("Failed to recv event by master");
                let v = rx.borrow().clone();
                println!("{:?}", v);
            },
        }
    }
}
