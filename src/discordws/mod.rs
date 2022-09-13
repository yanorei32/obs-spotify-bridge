use tungstenite::{connect, Message};
use url::Url;

pub mod model;

pub fn get_spotify_token(discord_token: &str) -> String {
    let url = Url::parse("wss://gateway.discord.gg/").unwrap();
    let (mut ws, _) = connect(url).expect("Failed to connect Discord");

    let v = model::ReqOp2 {
        op: 2,
        d: model::ReqOp2D {
            token: discord_token.to_string(),
            properties: model::ReqOp2DProp {},
            compress: false,
        },
    };

    // println!("TokenRequest: {}", serde_json::to_string(&v).unwrap());
    ws.write_message(Message::Text(serde_json::to_string(&v).unwrap()))
        .expect("Failed to send token");

    loop {
        match ws.read_message().expect("Error reading message") {
            Message::Text(v) => {
                if let Ok(model::Response::Ready(v)) = serde_json::from_str::<model::Response>(&v) {
                    return v
                        .connected_accounts
                        .iter()
                        .find_map(|v| match v {
                            model::ConnectedAccount::Spotify { access_token } => Some(access_token),
                            model::ConnectedAccount::Other => None,
                        })
                        .expect("Failed to lookup Spotify token")
                        .to_string();
                }
            }
            Message::Close(v) => panic!("Unexpected close by server: {:?}", v),
            _ => {}
        }
    }
}
