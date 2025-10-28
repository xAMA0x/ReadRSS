use crate::storage;
use crate::models::*;
use sqlx::SqlitePool;
use anyhow::Result;

pub async fn add_new_feed(pool: &SqlitePool, url: &str, title: Option<String>) -> Result<()> {
    let title = title.unwrap_or_else(|| "Untitled Feed".to_string());
    storage::add_feed(pool, &title, url).await
}

pub async fn list_all_feeds(pool: &SqlitePool) -> Result<Vec<Feed>> {
    storage::get_feeds(pool).await
}