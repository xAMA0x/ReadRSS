pub mod error;
pub mod feed;
pub mod poller;

pub use error::PollError;
pub use feed::shared_feed_list;
pub use feed::{add_feed, list_feeds, remove_feed};
pub use feed::{FeedDescriptor, FeedEntry, SharedFeedList};
pub use poller::{spawn_poller, PollConfig, PollerHandle};
