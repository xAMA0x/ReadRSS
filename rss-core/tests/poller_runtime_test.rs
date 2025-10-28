use httpmock::prelude::*;
use reqwest::Client;
use tokio::sync::mpsc;

use rss_core::{shared_feed_list, spawn_poller, Event, FeedDescriptor, PollConfig, SeenStore};

#[tokio::test]
async fn spawn_poller_emits_event() {
    let server = MockServer::start();
    let _m = server.mock(|when, then| {
        when.method(GET).path("/feed");
        then.status(200)
            .header("content-type", "application/rss+xml")
            .body(r#"<?xml version=\"1.0\"?><rss version=\"2.0\"><channel><title>T</title><item><title>A</title><link>http://e/1</link><guid>1</guid></item></channel></rss>"#);
    });

    let feeds = shared_feed_list(vec![FeedDescriptor {
        id: "feed1".into(),
        title: "t".into(),
        url: format!("{}/feed", server.base_url()),
    }]);

    let cfg = PollConfig { interval: std::time::Duration::from_millis(50), request_timeout: std::time::Duration::from_secs(2), max_retries: 1, retry_backoff_ms: 10 };
    let client = Client::new();
    let (tx, mut rx) = mpsc::channel(8);
    let seen = SeenStore::in_memory();

    let handle = rss_core::spawn_poller(feeds, cfg, client, tx, seen);

    // Wait for an event up to 2 seconds
    let evt = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
        .await
        .expect("timed out")
        .expect("channel closed");

    match evt {
        Event::NewArticles(fid, entries) => {
            assert_eq!(fid, "feed1");
            assert!(!entries.is_empty());
        }
    }

    handle.stop().await.expect("stop poller");
}
