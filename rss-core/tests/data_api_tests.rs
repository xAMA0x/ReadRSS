use rss_core::{shared_feed_list, DataApi, FeedDescriptor};

#[tokio::test]
async fn data_api_persists_feeds_and_read_state() {
    // Use a temp directory under the system temp
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "readrss_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    tokio::fs::create_dir_all(&dir).await.unwrap();

    let feeds = shared_feed_list(Vec::new());
    let api = DataApi::load_from_dir(feeds.clone(), &dir).await;

    // Add a feed and ensure feeds.json is written
    let fd = FeedDescriptor {
        id: "f1".into(),
        title: "Feed 1".into(),
        url: "http://example.com/feed".into(),
    };
    api.add_feed(fd.clone()).await;

    let feeds_list = api.list_feeds().await;
    assert_eq!(feeds_list.len(), 1);

    // Reload a new API from disk and ensure the feed is present
    let feeds2 = shared_feed_list(Vec::new());
    let api2 = DataApi::load_from_dir(feeds2.clone(), &dir).await;
    let feeds_list2 = api2.list_feeds().await;
    assert_eq!(feeds_list2.len(), 1);
    assert_eq!(feeds_list2[0].id, "f1");

    // Mark read and check is_read persists
    let entry = rss_core::FeedEntry {
        feed_id: "f1".into(),
        title: "A".into(),
        summary: None,
        url: "http://e/1".into(),
        published_at: None,
        guid: Some("guid-1".into()),
        author: None,
        category: None,
        content_html: None,
        image_url: None,
    };
    api2.mark_read(&entry).await;

    // Reopen
    let feeds3 = shared_feed_list(Vec::new());
    let api3 = DataApi::load_from_dir(feeds3, &dir).await;
    assert!(api3.is_read(&entry).await);

    // Cleanup: remove temp dir
    let _ = tokio::fs::remove_dir_all(&dir).await;
}
