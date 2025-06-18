use crate::notify_model::{Music, Notify, Receiver};
use anyhow::{Context, Result as AHResult};
use obws::{
    error::Result as OBResult,
    requests::inputs::{InputId, SetSettings},
    Client,
};

pub mod model;

pub async fn obsdriver(
    addr: &str,
    port: u16,
    pass: Option<&str>,
    rx: Receiver,
    mut shutdown_rx: model::ShutdownReceiver,
) -> AHResult<()> {
    let mut rx = rx.clone();

    loop {
        tokio::select! {
            changed = shutdown_rx.changed() => {
                changed.with_context(|| "Failed to recv shutdown by master")?;

                update(addr, port, pass, None).await?;

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

                update(addr, port, pass, v.as_ref()).await?;
            }
        }
    }
}

pub async fn update(addr: &str, port: u16, pass: Option<&str>, v: Option<&Music>) -> OBResult<()> {
    let client = match Client::connect(addr, port, pass).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to OBS, ignoring: {e:?}");
            return Ok(());
        }
    };

    update_text(&client, v).await?;
    update_albumart(&client, v).await?;

    Ok(())
}

pub async fn update_text(c: &Client, v: Option<&Music>) -> OBResult<()> {
    let ii = c.inputs().list(None).await?;

    for i in ii {
        if !i.id.name.ends_with("obs-spotify.text") {
            continue;
        }

        let mut settings = serde_json::Map::new();

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
                input: InputId::Uuid(i.id.uuid),
                settings: &settings,
                overlay: Some(true),
            })
            .await?;
    }

    Ok(())
}

pub async fn update_albumart(c: &Client, v: Option<&Music>) -> OBResult<()> {
    let ii = c.inputs().list(None).await?;

    for i in ii {
        if !i.id.name.ends_with("obs-spotify.albumart") {
            continue;
        }

        let mut settings = serde_json::Map::new();

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
                input: InputId::Uuid(i.id.uuid),
                settings: &settings,
                overlay: Some(true),
            })
            .await?;
    }

    Ok(())
}
