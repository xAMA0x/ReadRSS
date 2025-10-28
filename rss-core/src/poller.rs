use std::time::Duration;

use chrono::Utc;
use reqwest::Client;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::error::PollError;
use crate::feed::{FeedDescriptor, FeedEntry, SharedFeedList};

#[derive(Debug, Clone, Copy)]
pub struct PollConfig {
    pub interval: Duration,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(300),
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
    update_tx: mpsc::Sender<Vec<FeedEntry>>,
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
                        match fetch_feed(&client, &feed).await {
                            Ok(entries) if !entries.is_empty() => {
                                if let Err(_) = update_tx.send(entries).await {
                                    warn!("update receiver dropped");
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

async fn fetch_feed(client: &Client, feed: &FeedDescriptor) -> Result<Vec<FeedEntry>, PollError> {
    let response = client.get(feed.url.clone()).send().await?;
    let bytes = response.bytes().await?;
    let mut cursor = std::io::Cursor::new(bytes.to_vec());
    let channel = rss::Channel::read_from(&mut cursor)?;

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
