use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub(crate) struct ReqOp2DProp {}

#[derive(Debug, Serialize)]
pub(crate) struct ReqOp2D {
    pub(crate) token: String,
    pub(crate) properties: ReqOp2DProp,
    pub(crate) compress: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReqOp2 {
    pub(crate) op: i32,
    pub(crate) d: ReqOp2D,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub(crate) enum ConnectedAccount {
    Spotify {
        access_token: String,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Ready {
    pub(crate) connected_accounts: Vec<ConnectedAccount>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "t", content = "d")]
#[serde(rename_all = "UPPERCASE")]
pub(crate) enum Response {
    Ready(Ready),
}
