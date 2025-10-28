use thiserror::Error;

#[derive(Debug, Error)]
pub enum PollError {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("feed parsing error: {0}")]
    Parse(#[from] rss::Error),
    #[error("poller task failed: {0}")]
    Task(#[from] tokio::task::JoinError),
    #[error("update channel closed unexpectedly")]
    UpdateChannelClosed,
}
