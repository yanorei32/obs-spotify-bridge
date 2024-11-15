#![warn(clippy::pedantic)]
use crate::discordapi::{get_spotify_credentials, renew_spotify_token};
use crate::filter::drop_duplicate;
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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let c = envy::from_env::<model::Config>().unwrap();

    println!("Get Spotify credential by Discord WebSocket...");
    let cred = get_spotify_credentials(&c.discord_token)
        .expect("Failed to get Spotify credential from Discord credential.");

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

    let mut filter = tokio::spawn(async move { drop_duplicate(to_f, from_f).await });
    let mut spotify_ws = tokio::spawn(async move { connect_ws(&token, tx).await });

    let rx2 = rx.clone();
    let mut wsserver = tokio::spawn(async move { serve(&c.ws_bind_address, rx2).await });

    let (shutdown_tx, shutdown_rx) = watch::channel(obsdriver::model::ExpectedState::Operational);

    let rx3 = rx.clone();
    let mut obsdriver = tokio::spawn(async move {
        obsdriver(&c.obs_address, c.obs_port, c.obs_password, rx3, shutdown_rx).await
    });

    println!("Entering Main Loop...");

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                shutdown_tx.send(obsdriver::model::ExpectedState::GracefulShutdown).unwrap();
            },
            v = &mut spotify_ws => {
                panic!("Unexpected Shutdown SpotifyWS: {:?}", v.unwrap());
            },
            v = &mut wsserver => {
                panic!("Unexpected Shutdown WSServer: {:?}", v.unwrap());
            },
            v = &mut filter => {
                panic!("Unexpected Shutdown Filter: {:?}", v.unwrap());
            },
            v = &mut obsdriver => {
                if let Err(v) = v.unwrap() {
                    panic!("Unexpected Shutdown OBSDriver: {v:?}");
                } else {
                    println!("Graceful shutdown is complete");
                    return
                }
            },
            changed = rx.changed() => {
                changed.expect("Failed to recv event by master");
                let v = rx.borrow().clone();
                println!("{v:?}");
            },
        }
    }
}
