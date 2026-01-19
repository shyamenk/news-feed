use rusqlite::{params, Connection, Result};
use std::error::Error;
use std::path::Path;
use chrono::{DateTime, Utc};

pub struct Database {
    conn: Connection,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Feed {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub category: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Post {
    pub id: i64,
    pub feed_id: i64,
    pub title: String,
    pub url: String,
    pub content: Option<String>,
    pub pub_date: Option<DateTime<Utc>>,
    pub is_read: bool,
    pub is_bookmarked: bool,
    pub is_archived: bool,
    pub is_read_later: bool,
    pub feed_title: Option<String>,
}

#[allow(dead_code)]
impl Database {
    pub fn init() -> Result<Self, Box<dyn Error>> {
        Self::init_with_path("news_feed.db")
    }

    pub fn init_with_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS feeds (
                id INTEGER PRIMARY KEY,
                url TEXT NOT NULL UNIQUE,
                title TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS posts (
                id INTEGER PRIMARY KEY,
                feed_id INTEGER NOT NULL REFERENCES feeds(id),
                title TEXT NOT NULL,
                url TEXT NOT NULL UNIQUE,
                content TEXT,
                pub_date TEXT,
                is_read BOOLEAN NOT NULL DEFAULT 0,
                is_bookmarked BOOLEAN NOT NULL DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_preferences (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        let db = Database { conn };
        db.migrate_schema()?;
        Ok(db)
    }

    pub fn add_feed(&self, url: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO feeds (url) VALUES (?1)",
            params![url],
        )?;
        let id: i64 = self.conn.query_row(
            "SELECT id FROM feeds WHERE url = ?1",
            params![url],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    pub fn get_feeds(&self) -> Result<Vec<Feed>> {
        let mut stmt = self.conn.prepare("SELECT id, url, title, COALESCE(category, 'General') FROM feeds")?;
        let feed_iter = stmt.query_map([], |row| {
            Ok(Feed {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                category: row.get(3)?,
            })
        })?;

        let mut feeds = Vec::new();
        for feed in feed_iter {
            feeds.push(feed?);
        }
        Ok(feeds)
    }

    pub fn insert_post(&self, feed_id: i64, title: &str, url: &str, content: Option<&str>, pub_date: Option<DateTime<Utc>>) -> Result<()> {
        let pub_date_str = pub_date.map(|d| d.to_rfc3339());
        self.conn.execute(
            "INSERT OR IGNORE INTO posts (feed_id, title, url, content, pub_date) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![feed_id, title, url, content, pub_date_str],
        )?;
        Ok(())
    }

    pub fn get_posts(&self, filter: PostFilter) -> Result<Vec<Post>> {
        let mut query = "SELECT p.id, p.feed_id, p.title, p.url, p.content, p.pub_date, p.is_read, p.is_bookmarked, COALESCE(p.is_archived, 0), COALESCE(p.is_read_later, 0), f.title
                         FROM posts p
                         JOIN feeds f ON p.feed_id = f.id".to_string();

        let mut conditions = Vec::new();
        if filter.only_unread {
            conditions.push("p.is_read = 0");
        }
        if filter.only_bookmarked {
            conditions.push("p.is_bookmarked = 1");
        }
        if filter.only_archived {
            conditions.push("p.is_archived = 1");
        }
        if filter.only_read_later {
            conditions.push("p.is_read_later = 1");
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY p.pub_date DESC LIMIT 100"); // Limit for MVP

        let mut stmt = self.conn.prepare(&query)?;
        let post_iter = stmt.query_map([], |row| {
            let pub_date_str: Option<String> = row.get(5)?;
            let pub_date = pub_date_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc)));

            Ok(Post {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                title: row.get(2)?,
                url: row.get(3)?,
                content: row.get(4)?,
                pub_date,
                is_read: row.get(6)?,
                is_bookmarked: row.get(7)?,
                is_archived: row.get(8)?,
                is_read_later: row.get(9)?,
                feed_title: row.get(10)?,
            })
        })?;

        let mut posts = Vec::new();
        for post in post_iter {
            posts.push(post?);
        }
        Ok(posts)
    }

    pub fn mark_as_read(&self, post_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE posts SET is_read = 1 WHERE id = ?1",
            params![post_id],
        )?;
        Ok(())
    }

    pub fn mark_as_unread(&self, post_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE posts SET is_read = 0 WHERE id = ?1",
            params![post_id],
        )?;
        Ok(())
    }

    pub fn toggle_bookmark(&self, post_id: i64) -> Result<()> {
         self.conn.execute(
            "UPDATE posts SET is_bookmarked = NOT is_bookmarked WHERE id = ?1",
            params![post_id],
        )?;
        Ok(())
    }

    pub fn cleanup_non_bookmarked_posts(&self) -> Result<()> {
        self.conn.execute(
            "DELETE FROM posts WHERE is_bookmarked = 0",
            [],
        )?;
        Ok(())
    }

    pub fn delete_post(&self, post_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM posts WHERE id = ?1",
            params![post_id],
        )?;
        Ok(())
    }

    fn migrate_schema(&self) -> Result<()> {
        // Check and add new columns to posts table if they don't exist
        let has_is_archived = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('posts') WHERE name='is_archived'",
            [],
            |row| row.get::<_, i64>(0),
        )? > 0;

        if !has_is_archived {
            self.conn.execute(
                "ALTER TABLE posts ADD COLUMN is_archived BOOLEAN NOT NULL DEFAULT 0",
                [],
            )?;
        }

        let has_is_read_later = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('posts') WHERE name='is_read_later'",
            [],
            |row| row.get::<_, i64>(0),
        )? > 0;

        if !has_is_read_later {
            self.conn.execute(
                "ALTER TABLE posts ADD COLUMN is_read_later BOOLEAN NOT NULL DEFAULT 0",
                [],
            )?;
        }

        let has_created_at = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('posts') WHERE name='created_at'",
            [],
            |row| row.get::<_, i64>(0),
        )? > 0;

        if !has_created_at {
            self.conn.execute(
                "ALTER TABLE posts ADD COLUMN created_at TEXT",
                [],
            )?;
        }

        // Check and add category column to feeds table if it doesn't exist
        let has_category = self.conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('feeds') WHERE name='category'",
            [],
            |row| row.get::<_, i64>(0),
        )? > 0;

        if !has_category {
            self.conn.execute(
                "ALTER TABLE feeds ADD COLUMN category TEXT DEFAULT 'General'",
                [],
            )?;
        }

        Ok(())
    }

    pub fn mark_as_archived(&self, post_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE posts SET is_archived = NOT is_archived WHERE id = ?1",
            params![post_id],
        )?;
        Ok(())
    }

    pub fn mark_as_read_later(&self, post_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE posts SET is_read_later = NOT is_read_later WHERE id = ?1",
            params![post_id],
        )?;
        Ok(())
    }

    pub fn get_posts_by_category(&self, category: &str) -> Result<Vec<Post>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.feed_id, p.title, p.url, p.content, p.pub_date, p.is_read, p.is_bookmarked, p.is_archived, p.is_read_later, f.title
             FROM posts p
             JOIN feeds f ON p.feed_id = f.id
             WHERE f.category = ?1
             ORDER BY p.pub_date DESC LIMIT 100"
        )?;

        let post_iter = stmt.query_map(params![category], |row| {
            let pub_date_str: Option<String> = row.get(5)?;
            let pub_date = pub_date_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc)));

            Ok(Post {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                title: row.get(2)?,
                url: row.get(3)?,
                content: row.get(4)?,
                pub_date,
                is_read: row.get(6)?,
                is_bookmarked: row.get(7)?,
                is_archived: row.get(8)?,
                is_read_later: row.get(9)?,
                feed_title: row.get(10)?,
            })
        })?;

        let mut posts = Vec::new();
        for post in post_iter {
            posts.push(post?);
        }
        Ok(posts)
    }

    pub fn get_feeds_by_category(&self, category: &str) -> Result<Vec<Feed>> {
        let mut stmt = self.conn.prepare("SELECT id, url, title, category FROM feeds WHERE category = ?1")?;
        let feed_iter = stmt.query_map(params![category], |row| {
            Ok(Feed {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                category: row.get(3)?,
            })
        })?;

        let mut feeds = Vec::new();
        for feed in feed_iter {
            feeds.push(feed?);
        }
        Ok(feeds)
    }

    pub fn get_categories(&self) -> Result<Vec<String>> {
        // Get categories from both the categories table and feeds table
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT name FROM (
                SELECT name FROM categories
                UNION
                SELECT DISTINCT category AS name FROM feeds WHERE category IS NOT NULL
            ) ORDER BY name"
        )?;
        let category_iter = stmt.query_map([], |row| row.get(0))?;

        let mut categories = Vec::new();
        for category in category_iter {
            categories.push(category?);
        }
        Ok(categories)
    }

    pub fn delete_feed(&self, feed_id: i64) -> Result<()> {
        // Delete posts associated with this feed first
        self.conn.execute(
            "DELETE FROM posts WHERE feed_id = ?1",
            params![feed_id],
        )?;

        // Then delete the feed
        self.conn.execute(
            "DELETE FROM feeds WHERE id = ?1",
            params![feed_id],
        )?;
        Ok(())
    }

    pub fn update_feed_category(&self, feed_id: i64, category: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE feeds SET category = ?1 WHERE id = ?2",
            params![category, feed_id],
        )?;
        Ok(())
    }

    pub fn add_feed_with_category(&self, url: &str, category: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO feeds (url, category) VALUES (?1, ?2)",
            params![url, category],
        )?;
        let id: i64 = self.conn.query_row(
            "SELECT id FROM feeds WHERE url = ?1",
            params![url],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    pub fn get_count(&self, query: &str) -> Result<usize> {
        let count: i64 = self.conn.query_row(query, [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn get_category_stats(&self) -> Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT f.category, COUNT(p.id)
             FROM feeds f
             LEFT JOIN posts p ON f.id = p.feed_id
             GROUP BY f.category
             ORDER BY f.category"
        )?;

        let stats_iter = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
        })?;

        let mut stats = Vec::new();
        for stat in stats_iter {
            stats.push(stat?);
        }
        Ok(stats)
    }

    pub fn add_category(&self, name: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO categories (name) VALUES (?1)",
            params![name],
        )?;
        Ok(())
    }

    pub fn delete_category(&self, name: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE feeds SET category = 'General' WHERE category = ?1",
            params![name],
        )?;
        self.conn.execute(
            "DELETE FROM categories WHERE name = ?1",
            params![name],
        )?;
        Ok(())
    }

    pub fn rename_category(&self, old_name: &str, new_name: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE feeds SET category = ?1 WHERE category = ?2",
            params![new_name, old_name],
        )?;
        self.conn.execute(
            "UPDATE categories SET name = ?1 WHERE name = ?2",
            params![new_name, old_name],
        )?;
        Ok(())
    }

    pub fn ensure_categories_table(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            )",
            [],
        )?;
        let existing: Vec<String> = self.get_categories().unwrap_or_default();
        for cat in existing {
            let _ = self.conn.execute(
                "INSERT OR IGNORE INTO categories (name) VALUES (?1)",
                params![cat],
            );
        }
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO categories (name) VALUES ('General')",
            [],
        );
        Ok(())
    }

    /// Reset the database by deleting all data (feeds, posts, categories)
    pub fn reset(&self) -> Result<()> {
        self.conn.execute("DELETE FROM posts", [])?;
        self.conn.execute("DELETE FROM feeds", [])?;
        self.conn.execute("DELETE FROM categories", [])?;
        self.conn.execute("DELETE FROM user_preferences", [])?;
        Ok(())
    }

    /// Clean up old posts older than specified days
    pub fn cleanup_old_posts(&self, days: u32) -> Result<usize> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let cutoff_str = cutoff.to_rfc3339();

        let count = self.conn.execute(
            "DELETE FROM posts WHERE pub_date < ?1 AND is_bookmarked = 0",
            params![cutoff_str],
        )?;
        Ok(count)
    }

    /// Get total counts for statistics
    pub fn get_total_posts_count(&self) -> Result<usize> {
        self.get_count("SELECT COUNT(*) FROM posts")
    }

    pub fn get_total_feeds_count(&self) -> Result<usize> {
        self.get_count("SELECT COUNT(*) FROM feeds")
    }
}

pub struct PostFilter {
    pub only_unread: bool,
    pub only_bookmarked: bool,
    pub only_archived: bool,
    pub only_read_later: bool,
}

impl Database {
    /// Get fresh feed: latest N unread posts per category
    pub fn get_fresh_feed(&self, per_category_limit: usize) -> Result<Vec<Post>> {
        let categories = self.get_categories().unwrap_or_default();
        let mut all_posts = Vec::new();

        for category in categories {
            let query = format!(
                "SELECT p.id, p.feed_id, p.title, p.url, p.content, p.pub_date, p.is_read, p.is_bookmarked, 
                        COALESCE(p.is_archived, 0), COALESCE(p.is_read_later, 0), f.title
                 FROM posts p
                 JOIN feeds f ON p.feed_id = f.id
                 WHERE f.category = ?1 AND p.is_read = 0
                 ORDER BY p.pub_date DESC
                 LIMIT ?2"
            );

            let mut stmt = self.conn.prepare(&query)?;
            let post_iter = stmt.query_map(params![category, per_category_limit as i64], |row| {
                let pub_date_str: Option<String> = row.get(5)?;
                let pub_date = pub_date_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc)));

                Ok(Post {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    url: row.get(3)?,
                    content: row.get(4)?,
                    pub_date,
                    is_read: row.get(6)?,
                    is_bookmarked: row.get(7)?,
                    is_archived: row.get(8)?,
                    is_read_later: row.get(9)?,
                    feed_title: row.get(10)?,
                })
            })?;

            for post in post_iter {
                all_posts.push(post?);
            }
        }

        // Sort all posts by pub_date descending
        all_posts.sort_by(|a, b| b.pub_date.cmp(&a.pub_date));
        Ok(all_posts)
    }

    /// Update post content (for fetching full article)
    #[allow(dead_code)]
    pub fn update_post_content(&self, post_id: i64, content: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE posts SET content = ?1 WHERE id = ?2",
            params![content, post_id],
        )?;
        Ok(())
    }
}
