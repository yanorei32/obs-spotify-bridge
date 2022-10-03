use tokio::sync::watch;

pub type ShutdownReceiver = watch::Receiver<ExpectedState>;

#[derive(Debug)]
pub enum ExpectedState {
    Operational,
    GracefulShutdown,
}
