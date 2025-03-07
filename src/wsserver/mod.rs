use crate::notify_model::Receiver;
use anyhow::{Context, Result};
use futures::StreamExt;
use futures_util::SinkExt;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error as TSError, Message, Result as TSResult},
};

pub async fn serve(bind_address: &str, rx: Receiver) -> Result<()> {
    let listener = TcpListener::bind(bind_address)
        .await
        .with_context(|| "Failed to listen TCP")?;

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream, rx.clone()));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream, rx: Receiver) {
    if let Err(e) = handle_connection(stream, rx).await {
        match e {
            TSError::ConnectionClosed | TSError::Protocol(_) | TSError::Utf8 => (),
            err => println!("Error processing connection: {err}"),
        }
    }
}

async fn handle_connection(stream: TcpStream, mut rx: Receiver) -> TSResult<()> {
    let ws = accept_async(stream).await.expect("Failed to accept");

    let (mut tx, _) = ws.split();

    let mut interval = tokio::time::interval(Duration::from_millis(1000));

    loop {
        tokio::select! {
            changed = rx.changed() => {
                changed.expect("Failed to recv event by master");
                let v = rx.borrow().clone();
                tx.send(Message::Text(serde_json::to_string(&v).unwrap().into())).await?;
            }
            _ = interval.tick() => {
                tx.send(Message::Ping(vec![].into())).await?;
            }
        }
    }
}
