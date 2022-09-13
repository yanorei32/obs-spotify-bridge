use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

pub mod model;

pub async fn fetch(token: &str, sender: &model::Sender) {
    let url = format!("wss://dealer.spotify.com/?access_token={}", token);
    // println!("Spotify WSS: {}", url);

    let url = Url::parse(&url).expect("Failed to parse URL, Invalid token?");

    let (ws, _) = connect_async(url).await.expect("Failed to connect Spotify");
    let (mut tx, mut rx) = ws.split();
    let mut interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            msg = rx.next() => {
                let msg = match msg {
                    Some(msg) => msg,
                    None => continue
                };

                let msg_str = match msg.expect("Failed to read message") {
                    Message::Text(v) => v,
                    Message::Close(v) => panic!("Unexpected close by server: {:?}", v),
                    _ => { continue }
                };

                let msg = match serde_json::from_str::<model::Response>(&msg_str) {
                    Ok(v) => v,
                    Err(_e) => {
                        // println!("UnknownMes: {}", msg_str);
                        // println!("UnknownMes: {:?}", e);
                        continue;
                    }
                };

                let msg = match msg {
                    model::Response::Message(v) => v,
                    model::Response::Pong => {
                        // println!("pong");
                        continue;
                    }
                };

                match msg {
                    model::MessageLikeObjects::Put(v) => {
                        // println!("ConnectionId: {}", v.headers.spotify_connection_id);
                        let c = reqwest::Client::new();

                        c.put("https://api.spotify.com/v1/me/notifications/player")
                            .query(&[("connection_id", v.headers.spotify_connection_id)])
                            .bearer_auth(token)
                            .header(reqwest::header::CONTENT_LENGTH, "0")
                            .send()
                            .await
                            .expect("Failed to activate Spotify-Connection");
                    },
                    model::MessageLikeObjects::WssEvent(v) => {
                        sender.send(v).await.expect("Failed to send to master");
                    },
                };

            }
            _ = interval.tick() => {
                tx.send(
                    Message::Text(
                        serde_json::to_string(&model::Request::Ping).unwrap()
                    )
                ).await.expect("Failed to send ping");
                // println!("ping");
            }
        }
    }
}
