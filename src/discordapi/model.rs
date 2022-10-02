use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ReqOp2DProp {}

#[derive(Debug, Serialize)]
pub struct ReqOp2D {
    pub token: String,
    pub properties: ReqOp2DProp,
    pub compress: bool,
}

#[derive(Debug, Serialize)]
pub struct ReqOp2 {
    pub op: i32,
    pub d: ReqOp2D,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpotifyCredential {
    pub id: String,
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum ConnectedAccount {
    Spotify(SpotifyCredential),
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct Ready {
    pub connected_accounts: Vec<ConnectedAccount>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "t", content = "d")]
#[serde(rename_all = "UPPERCASE")]
pub enum Response {
    Ready(Ready),
}

#[derive(Debug, Deserialize)]
pub struct SpotifyAccessTokenApiResponse {
    pub access_token: String,
}
