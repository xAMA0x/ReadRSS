use sqlx::{SqlitePool, FromRow};
use crate::models::{Feed, Article};
use anyhow::Result;

pub async fn init_db(db_url: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(db_url).await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}

pub async fn add_feed(pool: &SqlitePool, title: &str, url: &str) -> Result<()> {
    sqlx::query("INSERT INTO feeds (title, url, created_at) VALUES (?, ?, datetime('now'))")
        .bind(title)
        .bind(url)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_feeds(pool: &SqlitePool) -> Result<Vec<Feed>> {
    let feeds = sqlx::query_as::<_, Feed>("SELECT * FROM feeds")
        .fetch_all(pool)
        .await?;
    Ok(feeds)
}
