use rss_core::{shared_feed_list, DataApi, FeedDescriptor};

#[tokio::test]
async fn load_uses_tmp_fallback_on_corrupted_json() {
    // Create temp dir
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "readrss_corrupt_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    tokio::fs::create_dir_all(&dir).await.unwrap();

    // Write corrupted feeds.json
    let feeds_path = dir.join("feeds.json");
    tokio::fs::write(&feeds_path, b"{ this is not json ").await.unwrap();

    // Write valid tmp file
    let tmp_path = dir.join("feeds.json.tmp");
    let fd = FeedDescriptor { id: "x".into(), title: "T".into(), url: "http://example.com".into() };
    let vec = vec![fd.clone()];
    let bytes = serde_json::to_vec(&vec).unwrap();
    tokio::fs::write(&tmp_path, bytes).await.unwrap();

    // Load
    let feeds_store = shared_feed_list(Vec::new());
    let api = DataApi::load_from_dir(feeds_store.clone(), &dir).await;
    let feeds = api.list_feeds().await;
    assert_eq!(feeds.len(), 1, "should fall back to tmp file when main is corrupted");
    assert_eq!(feeds[0].id, fd.id);

    // cleanup
    let _ = tokio::fs::remove_dir_all(&dir).await;
}
