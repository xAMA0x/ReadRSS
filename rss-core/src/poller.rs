use std::path::Path;
use std::time::Duration;

use chrono::Utc;
use reqwest::Client;
use url::Url;
use futures_util::StreamExt;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::error::PollError;
use crate::feed::{FeedDescriptor, FeedEntry, SharedFeedList};
use crate::storage::SeenStore;

#[derive(Debug, Clone)]
pub struct PollConfig {
    pub interval: Duration,
    /// Per-request timeout
    pub request_timeout: Duration,
    /// Max number of retries on network errors
    pub max_retries: usize,
    /// Base backoff in milliseconds for exponential backoff
    pub retry_backoff_ms: u64,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(300),
            request_timeout: Duration::from_secs(15),
            max_retries: 3,
            retry_backoff_ms: 500,
        }
    }
}

pub struct PollerHandle {
    cancel_tx: broadcast::Sender<()>,
    join: JoinHandle<()>,
}

impl PollerHandle {
    pub async fn stop(self) -> Result<(), PollError> {
        let _ = self.cancel_tx.send(());
        self.join.await.map_err(PollError::from)
    }
}

pub fn spawn_poller(
    feeds: SharedFeedList,
    config: PollConfig,
    client: Client,
    update_tx: mpsc::Sender<Event>,
    seen: SeenStore,
) -> PollerHandle {
    let (cancel_tx, mut cancel_rx) = broadcast::channel(1);
    let join = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(config.interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = cancel_rx.recv() => {
                    info!("poller shutdown requested");
                    break;
                }
                _ = ticker.tick() => {
                    let feeds_snapshot = feeds.read().await.clone();
                    for feed in feeds_snapshot {
                        match fetch_feed_with_retries(&client, &feed, &config).await {
                            Ok(entries) if !entries.is_empty() => {
                                // Filter already seen
                                let mut new_entries = Vec::new();
                                for e in entries {
                                    if seen.is_new_and_mark(&e).await {
                                        new_entries.push(e);
                                    }
                                }
                                if !new_entries.is_empty() {
                                    let evt = Event::NewArticles(feed.id.clone(), new_entries);
                                    if update_tx.send(evt).await.is_err() {
                                        warn!("update receiver dropped");
                                    }
                                }
                            }
                            Ok(_) => {}
                            Err(err) => {
                                warn!(feed = %feed.url, error = %err, "failed to fetch feed");
                            }
                        }
                    }
                }
            }
        }
    });

    PollerHandle { cancel_tx, join }
}

async fn fetch_feed(
    client: &Client,
    feed: &FeedDescriptor,
    timeout: Duration,
) -> Result<Vec<FeedEntry>, PollError> {
    // HTTPS policy enforced in production
    let url = Url::parse(&feed.url)?;
    #[cfg(not(test))]
    if url.scheme() != "https" {
        // Autoriser HTTP uniquement en loopback pour tests/développement local
        let host_ok = match url.host_str() {
            Some("localhost") | Some("127.0.0.1") | Some("::1") => true,
            _ => false,
        };
        if !host_ok {
            return Err(PollError::UnsupportedScheme);
        }
    }

    const MAX_FEED_BYTES: usize = 10 * 1024 * 1024; // 10 MiB
    let response = client.get(url).timeout(timeout).send().await?;
    if let Some(len) = response.content_length() {
        if len > MAX_FEED_BYTES as u64 {
            return Err(PollError::TooLarge(len));
        }
    }
    let mut bytes_buf = bytes::BytesMut::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if bytes_buf.len() + chunk.len() > MAX_FEED_BYTES {
            return Err(PollError::TooLarge((bytes_buf.len() + chunk.len()) as u64));
        }
        bytes_buf.extend_from_slice(&chunk);
    }
    let bytes = bytes_buf.freeze();
    // Try RSS first
    let mut cursor_rss = std::io::Cursor::new(bytes.to_vec());
    match rss::Channel::read_from(&mut cursor_rss) {
        Ok(channel) => {
            let entries = channel
                .items()
                .iter()
                .map(|item| {
                    let mut entry = FeedEntry::from_rss_item(&feed.id, item);
                    if entry.published_at.is_none() {
                        entry.published_at = Some(Utc::now());
                    }
                    entry
                })
                .collect();
            Ok(entries)
        }
        Err(rss_err) => {
            // Fallback: try Atom
            let mut cursor = std::io::Cursor::new(bytes.to_vec());
            match atom_syndication::Feed::read_from(&mut cursor) {
                Ok(atom_feed) => {
                    let entries = atom_feed
                        .entries()
                        .iter()
                        .map(|e| {
                            let mut entry = FeedEntry::from_atom_entry(&feed.id, e);
                            if entry.published_at.is_none() {
                                entry.published_at = Some(Utc::now());
                            }
                            entry
                        })
                        .collect();
                    Ok(entries)
                }
                Err(_e2) => {
                    // Return the original RSS parse error for compatibility
                    Err(PollError::from(rss_err))
                }
            }
        }
    }
}

async fn fetch_feed_with_retries(
    client: &Client,
    feed: &FeedDescriptor,
    cfg: &PollConfig,
) -> Result<Vec<FeedEntry>, PollError> {
    let mut attempt = 0usize;
    loop {
        match fetch_feed(client, feed, cfg.request_timeout).await {
            Ok(entries) => return Ok(entries),
            Err(err) => {
                attempt += 1;
                if attempt > cfg.max_retries {
                    return Err(err);
                }
                let backoff = cfg.retry_backoff_ms * (1u64 << (attempt - 1));
                warn!(feed = %feed.url, %attempt, backoff_ms = backoff, error = %err, "retrying after error");
                tokio::time::sleep(Duration::from_millis(backoff)).await;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    NewArticles(String, Vec<FeedEntry>),
}

impl PollConfig {
    /// Load PollConfig from a JSON file path; if missing or invalid, return defaults.
    pub fn from_file(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let defaults = PollConfig::default();
        match std::fs::read(path) {
            Ok(bytes) => {
                #[derive(serde::Deserialize)]
                struct RawCfg {
                    #[serde(default)]
                    interval: Option<u64>,
                    #[serde(default)]
                    request_timeout: Option<u64>,
                    #[serde(default)]
                    max_retries: Option<usize>,
                    #[serde(default)]
                    retry_backoff_ms: Option<u64>,
                }
                if let Ok(raw) = serde_json::from_slice::<RawCfg>(&bytes) {
                    PollConfig {
                        interval: raw
                            .interval
                            .map(Duration::from_millis)
                            .unwrap_or(defaults.interval),
                        request_timeout: raw
                            .request_timeout
                            .map(Duration::from_millis)
                            .unwrap_or(defaults.request_timeout),
                        max_retries: raw.max_retries.unwrap_or(defaults.max_retries),
                        retry_backoff_ms: raw.retry_backoff_ms.unwrap_or(defaults.retry_backoff_ms),
                    }
                } else {
                    defaults
                }
            }
            Err(_) => defaults,
        }
    }
}

/// Poll all feeds once and return generated events. Useful for tests.
pub async fn poll_once(
    feeds: &[FeedDescriptor],
    cfg: &PollConfig,
    client: &Client,
    seen: &SeenStore,
) -> Vec<Event> {
    let mut out = Vec::new();
    for feed in feeds {
        match fetch_feed_with_retries(client, feed, cfg).await {
            Ok(entries) if !entries.is_empty() => {
                let mut new_entries = Vec::new();
                for e in entries {
                    if seen.is_new_and_mark(&e).await {
                        new_entries.push(e);
                    }
                }
                if !new_entries.is_empty() {
                    out.push(Event::NewArticles(feed.id.clone(), new_entries));
                }
            }
            Ok(_) => {}
            Err(err) => {
                warn!(feed = %feed.url, error = %err, "failed to fetch feed");
            }
        }
    }
    out
}
