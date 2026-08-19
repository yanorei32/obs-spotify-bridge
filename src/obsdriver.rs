use obws::{
    error::Result as OBResult,
    requests::inputs::{InputId, SetSettings},
    Client,
};

use crate::model::{Music, ObsConfig};

pub async fn update(obs_config: &ObsConfig, music: Option<&Music>) -> OBResult<()> {
    let client = match Client::connect(
        &obs_config.obs_address,
        obs_config.obs_port,
        obs_config.obs_password.as_deref(),
    )
    .await
    {
        Ok(client) => client,
        Err(e) => {
            tracing::warn!("Failed to connect to OBS, ignoring: {e:?}");
            return Ok(());
        }
    };

    update_text(&client, &obs_config.format, music).await?;
    update_albumart(&client, music).await?;

    Ok(())
}

fn replace_recursively(s: &str, pattern: &str, to: &str) -> String {
    let mut s = s.to_string();

    loop {
        if s.find(pattern).is_none() {
            return s;
        };

        s = s.replace(pattern, to);
    }
}

async fn update_text(c: &Client, format: &str, music: Option<&Music>) -> OBResult<()> {
    let text = match music {
        Some(music) => {
            let text = replace_recursively(format, "%TITLE%", &music.title);
            let text = replace_recursively(&text, "%ARTISTS%", &music.artists);
            text
        }
        None => String::new(),
    };

    let ii = c.inputs().list(None).await?;

    for i in ii {
        if !i.id.name.ends_with("obs-spotify.text") {
            continue;
        }

        c.inputs()
            .set_settings(SetSettings {
                input: InputId::Uuid(i.id.uuid),
                settings: &serde_json::json!({ "text": text }),
                overlay: Some(true),
            })
            .await?;
    }

    Ok(())
}

async fn update_albumart(c: &Client, music: Option<&Music>) -> OBResult<()> {
    let ii = c.inputs().list(None).await?;

    for i in ii {
        if !i.id.name.ends_with("obs-spotify.albumart") {
            continue;
        }

        c.inputs()
            .set_settings(SetSettings {
                input: InputId::Uuid(i.id.uuid),
                settings: &serde_json::json!({
                    "url": music.map(|m| m.albumart.clone()).unwrap_or("about:blank".to_string()),
                }),
                overlay: Some(true),
            })
            .await?;
    }

    Ok(())
}
