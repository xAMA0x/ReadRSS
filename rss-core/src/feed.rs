use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FeedDescriptor {
    pub id: String,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedEntry {
    pub feed_id: String,
    pub title: String,
    pub summary: Option<String>,
    pub url: String,
    pub published_at: Option<DateTime<Utc>>,
    pub guid: Option<String>,
    pub author: Option<String>,
    pub category: Option<String>,
}

impl FeedEntry {
    pub fn from_rss_item(feed_id: &str, item: &rss::Item) -> Self {
        let published_at = item
            .pub_date()
            .and_then(|value| DateTime::parse_from_rfc2822(value).ok())
            .map(|dt| dt.with_timezone(&Utc));

        // Extract author from Dublin Core extension or author field
        let author = item
            .dublin_core_ext()
            .and_then(|dc| dc.creators().first().map(|s| s.to_string()))
            .or_else(|| item.author().map(|s| s.to_string()));

        // Extract category from categories or Dublin Core subject
        let category = item
            .categories()
            .first()
            .map(|cat| cat.name().to_string())
            .or_else(|| {
                item.dublin_core_ext()
                    .and_then(|dc| dc.subjects().first().map(|s| s.to_string()))
            });

        Self {
            feed_id: feed_id.to_owned(),
            title: item.title().unwrap_or_default().to_owned(),
            summary: item.description().map(ToOwned::to_owned),
            url: item.link().unwrap_or_default().to_owned(),
            published_at,
            guid: item.guid().map(|guid| guid.value().to_owned()),
            author,
            category,
        }
    }
}

pub type SharedFeedList = Arc<RwLock<Vec<FeedDescriptor>>>;

pub fn shared_feed_list(initial: Vec<FeedDescriptor>) -> SharedFeedList {
    Arc::new(RwLock::new(initial))
}

pub async fn add_feed(store: &SharedFeedList, feed: FeedDescriptor) {
    let mut feeds = store.write().await;
    feeds.retain(|existing| existing.id != feed.id);
    feeds.push(feed);
}

pub async fn remove_feed(store: &SharedFeedList, feed_id: &str) {
    let mut feeds = store.write().await;
    feeds.retain(|existing| existing.id != feed_id);
}

pub async fn list_feeds(store: &SharedFeedList) -> Vec<FeedDescriptor> {
    store.read().await.clone()
}
