use tungstenite::{connect, Message};
use url::Url;

pub(crate) mod model;

pub(crate) fn get_spotify_token(discord_token: &str) -> String {
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
            Message::Text(v) => match serde_json::from_str::<model::Response>(&v) {
                Ok(v) => {
                    let model::Response::Ready(v) = v;
                    return v
                        .connected_accounts
                        .iter()
                        .filter_map(|v| match v {
                            model::ConnectedAccount::Spotify { access_token } => Some(access_token),
                            _ => None,
                        })
                        .next()
                        .expect("Failed to lookup Spotify token")
                        .to_string();
                }
                Err(_) => {}
            },
            Message::Close(v) => panic!("Unexpected close by server: {:?}", v),
            _ => {}
        }
    }
}
