use crate::notify_model::{Notify, Receiver, Sender};
use anyhow::{Context, Result};

pub async fn duplicate_filter(rx: Receiver, tx: Sender) -> Result<()> {
    let mut rx = rx.clone();
    let mut pv = Notify::Unknown;

    loop {
        tokio::select! {
            changed = rx.changed() => {
                changed.with_context(|| "Failed to recv event by master")?;
                let v = rx.borrow().clone();

                if v != pv {
                    tx.send(v.clone()).with_context(|| "Failed to send message")?;
                    pv = v;
                }
            }
        }
    }
}
