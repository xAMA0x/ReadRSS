use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use reqwest::Client;
use rss_core::{poller::poll_once, FeedDescriptor, PollConfig, SeenStore};

fn sample_rss() -> String {
    r#"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<rss version=\"2.0\">
  <channel>
    <title>Test Feed</title>
    <link>http://example.com/</link>
    <description>Test description</description>
    <item>
      <title>Item 1</title>
      <link>http://example.com/1</link>
      <guid>1</guid>
      <pubDate>Mon, 21 Oct 2024 07:28:00 GMT</pubDate>
      <description>First</description>
    </item>
    <item>
      <title>Item 2</title>
      <link>http://example.com/2</link>
      <guid>2</guid>
      <pubDate>Mon, 21 Oct 2024 08:00:00 GMT</pubDate>
      <description>Second</description>
    </item>
  </channel>
</rss>"#
        .to_string()
}

#[tokio::test]
async fn poll_once_emits_new_articles_and_deduplicates() {
  let server = MockServer::start().await;

  Mock::given(method("GET"))
    .and(path("/feed"))
    .respond_with(
      ResponseTemplate::new(200)
        .insert_header("content-type", "application/rss+xml")
        .set_body_string(sample_rss()),
    )
    .mount(&server)
    .await;

  let feed = FeedDescriptor {
        id: "feed1".into(),
        title: "Test".into(),
    url: format!("{}/feed", server.uri()),
    };
    let feeds = vec![feed];
    let cfg = PollConfig {
        interval: std::time::Duration::from_millis(10),
        request_timeout: std::time::Duration::from_secs(2),
        max_retries: 1,
        retry_backoff_ms: 10,
    };
    let client = Client::new();
    let seen = SeenStore::in_memory();

    // First poll -> 2 new articles
    let events = poll_once(&feeds, &cfg, &client, &seen).await;
    assert_eq!(events.len(), 1);
    match &events[0] {
        rss_core::Event::NewArticles(fid, entries) => {
            assert_eq!(fid, "feed1");
            assert_eq!(entries.len(), 2);
        }
    }

    // Second poll -> 0 new articles after dedup
    let events2 = poll_once(&feeds, &cfg, &client, &seen).await;
    assert!(events2.is_empty());
}
