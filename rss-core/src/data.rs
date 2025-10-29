use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::feed::{add_feed, list_feeds, remove_feed, FeedDescriptor, FeedEntry, SharedFeedList};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ReadData {
    // feed_id -> set of entry identities (marked as read)
    read: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Clone)]
pub struct DataApi {
    feeds: SharedFeedList,
    read_inner: Arc<RwLock<ReadData>>,
    feeds_path: PathBuf,
    read_path: PathBuf,
    articles_inner: Arc<RwLock<HashMap<String, Vec<FeedEntry>>>>, // feed_id -> entries cache
    articles_path: PathBuf,
}

impl DataApi {
    /// Initialize the DataApi by loading persisted feeds and read state from a config directory.
    pub async fn load_from_dir(feeds: SharedFeedList, dir: impl AsRef<Path>) -> Self {
        let dir = dir.as_ref();
        let feeds_path = dir.join("feeds.json");
        let read_path = dir.join("read_store.json");
        let articles_path = dir.join("articles_store.json");

        // Ensure directory exists
        if let Err(e) = tokio::fs::create_dir_all(dir).await {
            warn!(error = %e, "failed to create config dir");
        }

        // helper: read JSON with fallback to temp file on corruption
        async fn read_json_with_tmp_fallback<T: DeserializeOwned + Default>(path: &Path) -> T {
            match tokio::fs::read(path).await {
                Ok(bytes) => match serde_json::from_slice::<T>(&bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(error = %e, path = %path.display(), "failed to parse JSON, trying tmp fallback");
                        let tmp = path.with_extension("json.tmp");
                        match tokio::fs::read(&tmp).await {
                            Ok(tmp_bytes) => {
                                serde_json::from_slice::<T>(&tmp_bytes).unwrap_or_default()
                            }
                            Err(_) => Default::default(),
                        }
                    }
                },
                Err(_) => Default::default(),
            }
        }

        // Load feeds.json and populate the shared store
        let initial_feeds: Vec<FeedDescriptor> = read_json_with_tmp_fallback(&feeds_path).await;
        if !initial_feeds.is_empty() {
            let mut store = feeds.write().await;
            *store = initial_feeds;
        }

        // Load read_store.json
        let read_inner: ReadData = read_json_with_tmp_fallback(&read_path).await;

        // Load articles_store.json (cache des derniers articles)
        let articles_inner: HashMap<String, Vec<FeedEntry>> =
            read_json_with_tmp_fallback(&articles_path).await;

        Self {
            feeds,
            read_inner: Arc::new(RwLock::new(read_inner)),
            feeds_path,
            read_path,
            articles_inner: Arc::new(RwLock::new(articles_inner)),
            articles_path,
        }
    }

    async fn persist_feeds(&self) {
        let feeds = list_feeds(&self.feeds).await;
        match serde_json::to_vec_pretty(&feeds) {
            Ok(bytes) => {
                if let Some(parent) = self.feeds_path.parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                // Ecriture atomique
                let tmp = self.feeds_path.with_extension("json.tmp");
                if let Err(e) = tokio::fs::write(&tmp, &bytes).await {
                    warn!(error = %e, path = %tmp.display(), "failed to write temp feeds.json");
                }
                if let Err(e) = tokio::fs::rename(&tmp, &self.feeds_path).await {
                    warn!(error = %e, path = %self.feeds_path.display(), "failed to persist feeds.json");
                }
            }
            Err(e) => warn!(error = %e, "failed to serialize feeds for persistence"),
        }
    }

    async fn persist_read(&self) {
        let inner = self.read_inner.read().await;
        match serde_json::to_vec_pretty(&*inner) {
            Ok(bytes) => {
                if let Some(parent) = self.read_path.parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                let tmp = self.read_path.with_extension("json.tmp");
                if let Err(e) = tokio::fs::write(&tmp, &bytes).await {
                    warn!(error = %e, path = %tmp.display(), "failed to write temp read_store.json");
                }
                if let Err(e) = tokio::fs::rename(&tmp, &self.read_path).await {
                    warn!(error = %e, path = %self.read_path.display(), "failed to persist read_store.json");
                }
            }
            Err(e) => warn!(error = %e, "failed to serialize read map"),
        }
    }

    async fn persist_articles(&self) {
        let inner = self.articles_inner.read().await;
        match serde_json::to_vec_pretty(&*inner) {
            Ok(bytes) => {
                if let Some(parent) = self.articles_path.parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                let tmp = self.articles_path.with_extension("json.tmp");
                if let Err(e) = tokio::fs::write(&tmp, &bytes).await {
                    warn!(error = %e, path = %tmp.display(), "failed to write temp articles_store.json");
                }
                if let Err(e) = tokio::fs::rename(&tmp, &self.articles_path).await {
                    warn!(error = %e, path = %self.articles_path.display(), "failed to persist articles_store.json");
                }
            }
            Err(e) => warn!(error = %e, "failed to serialize articles map"),
        }
    }

    pub async fn add_feed(&self, feed: FeedDescriptor) {
        add_feed(&self.feeds, feed).await;
        self.persist_feeds().await;
    }

    pub async fn remove_feed(&self, feed_id: &str) {
        remove_feed(&self.feeds, feed_id).await;
        self.persist_feeds().await;
        // Optionally drop read marks for this feed
        let mut inner = self.read_inner.write().await;
        inner.read.remove(feed_id);
        drop(inner);
        self.persist_read().await;
    }

    pub async fn list_feeds(&self) -> Vec<FeedDescriptor> {
        list_feeds(&self.feeds).await
    }

    pub async fn is_read(&self, entry: &FeedEntry) -> bool {
        let key = entry.identity();
        let inner = self.read_inner.read().await;
        inner
            .read
            .get(&entry.feed_id)
            .map(|set| set.contains(&key))
            .unwrap_or(false)
    }

    pub async fn mark_read(&self, entry: &FeedEntry) {
        let key = entry.identity();
        let mut inner = self.read_inner.write().await;
        let set = inner.read.entry(entry.feed_id.clone()).or_default();
        if set.insert(key) {
            drop(inner);
            self.persist_read().await;
        } else {
            debug!("entry already marked as read");
        }
    }

    /// Upsert et persiste un lot d'articles pour un feed (dedup + tri + truncate)
    pub async fn upsert_articles(&self, feed_id: &str, entries: Vec<FeedEntry>) {
        const MAX_PER_FEED: usize = 300;
        let mut inner = self.articles_inner.write().await;
        let slot = inner.entry(feed_id.to_string()).or_default();
        // Index existants par identity
        let mut existing: HashSet<String> = slot.iter().map(|e| e.identity()).collect();
        for e in entries {
            let id = e.identity();
            if existing.insert(id) {
                slot.push(e);
            }
        }
        // Tri par date décroissante
        slot.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        if slot.len() > MAX_PER_FEED {
            slot.truncate(MAX_PER_FEED);
        }
        drop(inner);
        self.persist_articles().await;
    }

    /// Liste les articles persistés pour un feed donné
    pub async fn list_articles(&self, feed_id: &str) -> Vec<FeedEntry> {
        let inner = self.articles_inner.read().await;
        inner.get(feed_id).cloned().unwrap_or_default()
    }

    /// Liste tous les articles persistés, toutes sources confondues
    pub async fn list_all_articles(&self) -> Vec<FeedEntry> {
        let inner = self.articles_inner.read().await;
        let mut all = Vec::new();
        for v in inner.values() {
            all.extend(v.clone());
        }
        all.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        all
    }
}
