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
    #[error("unsupported URL scheme (https required)")]
    UnsupportedScheme,
    #[error("invalid feed url: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("feed too large: {0} bytes")] 
    TooLarge(u64),
}
