use crate::db::Database;
use rusqlite::Result;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AppStats {
    pub total_posts: usize,
    pub read_posts: usize,
    pub unread_posts: usize,
    pub saved_posts: usize,
    pub archived_posts: usize,
    pub read_later_posts: usize,
    pub feeds_count: usize,
    pub categories: Vec<(String, usize)>,
}

impl Default for AppStats {
    fn default() -> Self {
        AppStats {
            total_posts: 0,
            read_posts: 0,
            unread_posts: 0,
            saved_posts: 0,
            archived_posts: 0,
            read_later_posts: 0,
            feeds_count: 0,
            categories: vec![],
        }
    }
}

#[allow(dead_code)]
impl AppStats {
    pub fn from_db(db: &Database) -> Result<Self> {
        let total_posts = db.get_count("SELECT COUNT(*) FROM posts")?;
        let read_posts = db.get_count("SELECT COUNT(*) FROM posts WHERE is_read = 1")?;
        let unread_posts = total_posts - read_posts;
        let saved_posts = db.get_count("SELECT COUNT(*) FROM posts WHERE is_bookmarked = 1")?;
        let archived_posts = db.get_count("SELECT COUNT(*) FROM posts WHERE is_archived = 1")?;
        let read_later_posts = db.get_count("SELECT COUNT(*) FROM posts WHERE is_read_later = 1")?;
        let feeds_count = db.get_count("SELECT COUNT(*) FROM feeds")?;

        let categories = db.get_category_stats()?;

        Ok(AppStats {
            total_posts,
            read_posts,
            unread_posts,
            saved_posts,
            archived_posts,
            read_later_posts,
            feeds_count,
            categories,
        })
    }

    #[allow(dead_code)]
    pub fn reading_progress(&self) -> f64 {
        if self.total_posts > 0 {
            self.read_posts as f64 / self.total_posts as f64
        } else {
            0.0
        }
    }
}
