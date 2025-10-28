use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::feed::FeedEntry;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SeenData {
    // feed_id -> set of entry identities
    pub seen: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Clone)]
pub struct SeenStore {
    inner: Arc<RwLock<SeenData>>,
    path: Option<PathBuf>,
}

impl SeenStore {
    pub fn in_memory() -> Self {
        Self {
            inner: Arc::new(RwLock::new(SeenData::default())),
            path: None,
        }
    }

    pub async fn load_from(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let data = match tokio::fs::read(&path).await {
            Ok(bytes) => serde_json::from_slice::<SeenData>(&bytes).unwrap_or_default(),
            Err(_) => SeenData::default(),
        };
        Self {
            inner: Arc::new(RwLock::new(data)),
            path: Some(path),
        }
    }

    pub async fn is_new_and_mark(&self, entry: &FeedEntry) -> bool {
        let key = entry.identity();
        let feed_id = entry.feed_id.clone();
        let mut inner = self.inner.write().await;
        let set = inner.seen.entry(feed_id).or_default();
        if set.contains(&key) {
            false
        } else {
            set.insert(key);
            drop(inner);
            if let Err(err) = self.persist().await {
                warn!(%err, "failed to persist seen store");
            }
            true
        }
    }

    async fn persist(&self) -> Result<(), std::io::Error> {
        if let Some(path) = &self.path {
            let inner = self.inner.read().await;
            let bytes = serde_json::to_vec_pretty(&*inner).expect("serialize seen data");
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await.ok();
            }
            tokio::fs::write(path, bytes).await?;
        } else {
            debug!("seen store is in-memory only; skipping persist");
        }
        Ok(())
    }
}
