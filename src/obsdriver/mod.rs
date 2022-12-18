use crate::notify_model::{Music, Notify, Receiver};
use anyhow::{Context, Result};
use obws::{requests::inputs::SetSettings, Client};

pub mod model;

pub async fn obsdriver(
    addr: &str,
    port: u16,
    pass: Option<String>,
    rx: Receiver,
    mut shutdown_rx: model::ShutdownReceiver,
) -> Result<()> {
    let c = Client::connect(addr, port, pass)
        .await
        .with_context(|| "Failed to connect to OBS")?;

    let mut rx = rx.clone();

    loop {
        tokio::select! {
            changed = shutdown_rx.changed() => {
                changed.with_context(|| "Failed to recv shutdown by master")?;

                let v = None;
                update_text(&c, &v).await?;
                update_albumart(&c, &v).await?;

                return Ok(())
            },
            changed = rx.changed() => {
                changed.with_context(|| "Failed to recv event by master")?;

                let v = rx.borrow().clone();

                let v = if let Notify::Playing(v) = v {
                    Some(v)
                } else {
                    None
                };

                update_text(&c, &v).await?;
                update_albumart(&c, &v).await?;
            }
        }
    }
}

pub async fn update_text(c: &Client, v: &Option<Music>) -> Result<()> {
    let ii = c.inputs().list(None).await?;

    for i in ii {
        if !i.name.ends_with("obs-spotify.text") {
            continue;
        }

        let mut settings = serde_json::Map::new();

        let name = i.name.clone();

        settings.insert(
            "text".to_string(),
            serde_json::Value::String(if let Some(v) = &v {
                format!("♪{}/{}", v.title, v.artists)
            } else {
                String::new()
            }),
        );

        c.inputs()
            .set_settings(SetSettings {
                input: name.as_str(),
                settings: &settings,
                overlay: Some(true),
            })
            .await?;
    }

    Ok(())
}

pub async fn update_albumart(c: &Client, v: &Option<Music>) -> Result<()> {
    let ii = c.inputs().list(None).await?;

    for i in ii {
        if !i.name.ends_with("obs-spotify.albumart") {
            continue;
        }

        let mut settings = serde_json::Map::new();

        let name = i.name.clone();

        settings.insert(
            "url".to_string(),
            serde_json::Value::String(if let Some(v) = &v {
                v.albumart.to_string()
            } else {
                "about:blank".to_string()
            }),
        );

        c.inputs()
            .set_settings(SetSettings {
                input: name.as_str(),
                settings: &settings,
                overlay: Some(true),
            })
            .await?;
    }

    Ok(())
}
