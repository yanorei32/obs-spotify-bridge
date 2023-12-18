use crate::discordapi::model::SpotifyAccessTokenApiResponse;
use anyhow::{bail, Context, Result};
use tokio_tungstenite::tungstenite::{connect, Message};
use url::Url;

pub mod model;

pub fn get_spotify_credentials(discord_token: &str) -> Result<model::SpotifyCredential> {
    let url = Url::parse("wss://gateway.discord.gg/").unwrap();
    let (mut ws, _) = connect(url).with_context(|| "Failed to connect discord ws:// server")?;

    let v = model::ReqOp2 {
        op: 2,
        d: model::ReqOp2D {
            token: discord_token.to_string(),
            properties: model::ReqOp2DProp {},
            compress: false,
        },
    };

    ws.send(Message::Text(serde_json::to_string(&v).unwrap()))
        .with_context(|| "Failed to send op:2")?;

    loop {
        match ws
            .read()
            .with_context(|| "Failed to read_message")?
        {
            Message::Text(v) => {
                if let Ok(model::Response::Ready(v)) = serde_json::from_str::<model::Response>(&v) {
                    let spotify_id = v
                        .connected_accounts
                        .iter()
                        .find_map(|v| match v {
                            model::ConnectedAccount::Spotify(v) => Some(v),
                            model::ConnectedAccount::Other => None,
                        })
                        .with_context(|| "Failed to get Spotify account by Discord")?;

                    return Ok(spotify_id.clone());
                }
            }
            Message::Close(v) => {
                bail!("Unexpected Closed {:?}", v)
            }
            _ => {}
        }
    }
}

pub async fn renew_spotify_token(discord_token: &str, spotify_id: &str) -> Result<String> {
    let resp = reqwest::Client::new()
        .get(format!(
            "https://discord.com/api/v9/users/@me/connections/spotify/{spotify_id}/access-token",
        ))
        .header(reqwest::header::AUTHORIZATION, discord_token)
        .send()
        .await
        .with_context(|| "Failed to send reqeust to Discord HTTP API")?;

    let resp = resp
        .json::<SpotifyAccessTokenApiResponse>()
        .await
        .with_context(|| "Failed to parse JSON")?;

    Ok(resp.access_token)
}
