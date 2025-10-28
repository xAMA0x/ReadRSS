use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use crate::models::{Feed, Article};

#[derive(Default)]
pub struct AppCache {
    pub feeds: HashMap<i64, Feed>,
    pub articles: HashMap<i64, Vec<Article>>,
}

#[derive(Debug, Clone)]
pub enum UpdateEvent {
    FeedAdded(i64),
    ArticlesUpdated(i64),
}

pub struct AppState {
    pub cache: Arc<RwLock<AppCache>>,
    pub notifier: broadcast::Sender<UpdateEvent>,
}
