CREATE TABLE feeds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    url TEXT NOT NULL UNIQUE,
    site_url TEXT,
    etag TEXT,
    last_modified TEXT,
    polling_interval_sec INTEGER DEFAULT 3600,
    created_at TEXT NOT NULL
);

CREATE TABLE articles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    feed_id INTEGER NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    guid TEXT,
    title TEXT NOT NULL,
    link TEXT,
    summary TEXT,
    content TEXT,
    author TEXT,
    published_at TEXT,
    fetched_at TEXT NOT NULL,
    is_read INTEGER DEFAULT 0,
    is_starred INTEGER DEFAULT 0
);
