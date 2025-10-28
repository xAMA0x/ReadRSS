pub mod data;
pub mod error;
pub mod feed;
pub mod poller;
pub mod storage;

pub use data::DataApi;
pub use error::PollError;
pub use feed::shared_feed_list;
pub use feed::{add_feed, list_feeds, remove_feed};
pub use feed::{FeedDescriptor, FeedEntry, SharedFeedList};
pub use poller::{poll_once, spawn_poller, Event, PollConfig, PollerHandle};
pub use storage::SeenStore;
