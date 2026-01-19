use crate::db::{Database, Post, PostFilter};
use crate::input::TextInput;
use crate::navigation::{FocusPane, NavNode, SidebarState, SmartView};
use std::sync::{Arc, Mutex};

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Welcome,
    AddingFeed,
    AddingCategory,
    SelectingCategory,
    Confirming(ConfirmAction),
    Help,
    EditingCategoryFeeds(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    DeletePost(i64),
    #[allow(dead_code)]
    DeleteFeed(i64),
    DeleteCategory(String),
}

pub struct App {
    pub db: Arc<Mutex<Database>>,
    pub posts: Vec<Post>,
    pub focus: FocusPane,
    pub sidebar: SidebarState,
    pub active_node: NavNode,
    pub selected_index: usize,
    pub scroll_offset: u16,
    pub exit: bool,
    pub message: Option<String>,
    pub is_loading: bool,
    pub input_mode: InputMode,
    pub text_input: TextInput,
    pub feeds: Vec<crate::db::Feed>,
    pub selected_feed_index: usize,
    pub show_read: bool,
    pub pending_feed_url: Option<String>,
    pub category_feeds: Vec<crate::db::Feed>,
    pub category_feed_index: usize,
}

impl App {
    pub fn new(db: Database) -> Self {
        let db_arc = Arc::new(Mutex::new(db));
        let feeds = db_arc.lock().unwrap().get_feeds().unwrap_or_default();

        let mut sidebar = SidebarState::new();
        {
            let db = db_arc.lock().unwrap();
            sidebar.load_categories(&db);
            sidebar.update_counts(&db);
        }

        let is_first_run = feeds.is_empty();
        let active_node = NavNode::SmartView(SmartView::Fresh);

        let posts = if !is_first_run {
            db_arc.lock().unwrap().get_fresh_feed(10).unwrap_or_default()
        } else {
            vec![]
        };

        App {
            db: db_arc,
            posts,
            focus: FocusPane::Sidebar,
            sidebar,
            active_node,
            selected_index: 0,
            scroll_offset: 0,
            exit: false,
            message: None,
            is_loading: !is_first_run,
            input_mode: if is_first_run {
                InputMode::Welcome
            } else {
                InputMode::Normal
            },
            text_input: TextInput::new(),
            feeds,
            selected_feed_index: 0,
            show_read: false,
            pending_feed_url: None,
            category_feeds: vec![],
            category_feed_index: 0,
        }
    }

    pub fn load_category_feeds(&mut self, category: &str) {
        self.category_feeds = self
            .db
            .lock()
            .unwrap()
            .get_feeds_by_category(category)
            .unwrap_or_default();
        self.category_feed_index = 0;
    }

    pub fn next_category_feed(&mut self) {
        if !self.category_feeds.is_empty() && self.category_feed_index < self.category_feeds.len() - 1 {
            self.category_feed_index += 1;
        }
    }

    pub fn previous_category_feed(&mut self) {
        if self.category_feed_index > 0 {
            self.category_feed_index -= 1;
        }
    }

    pub fn delete_category_feed(&mut self) {
        if let Some(feed) = self.category_feeds.get(self.category_feed_index) {
            let feed_id = feed.id;
            let feed_title = feed.title.clone().unwrap_or_else(|| feed.url.clone());
            if self.db.lock().unwrap().delete_feed(feed_id).is_ok() {
                self.category_feeds.remove(self.category_feed_index);
                if self.category_feed_index >= self.category_feeds.len() && !self.category_feeds.is_empty() {
                    self.category_feed_index = self.category_feeds.len() - 1;
                }
                self.reload_feeds();
                self.refresh_sidebar();
                self.message = Some(format!("Deleted feed: {}", truncate_str(&feed_title, 30)));
            }
        }
    }

    pub fn focus_left(&mut self) {
        self.focus = match self.focus {
            FocusPane::Article => FocusPane::Posts,
            FocusPane::Posts => FocusPane::Sidebar,
            FocusPane::Sidebar => FocusPane::Sidebar,
        };
    }

    pub fn focus_right(&mut self) {
        self.focus = match self.focus {
            FocusPane::Sidebar => FocusPane::Posts,
            FocusPane::Posts => {
                if !self.posts.is_empty() {
                    FocusPane::Posts
                } else {
                    FocusPane::Posts
                }
            }
            FocusPane::Article => FocusPane::Article,
        };
    }

    pub fn select_sidebar_item(&mut self) {
        self.active_node = self.sidebar.selected_node();
        self.reload_posts_for_active_node();
        self.selected_index = 0;
        self.focus = FocusPane::Posts;
    }

    pub fn reload_posts_for_active_node(&mut self) {
        let db = self.db.lock().unwrap();
        let posts = match &self.active_node {
            NavNode::SmartView(sv) => match sv {
                SmartView::Fresh => {
                    if self.show_read {
                        db.get_posts(PostFilter {
                            only_unread: false,
                            only_bookmarked: false,
                            only_archived: false,
                            only_read_later: false,
                        })
                        .unwrap_or_default()
                    } else {
                        db.get_fresh_feed(15).unwrap_or_default()
                    }
                }
                SmartView::Starred => db
                    .get_posts(PostFilter {
                        only_unread: false,
                        only_bookmarked: true,
                        only_archived: false,
                        only_read_later: false,
                    })
                    .unwrap_or_default(),
                SmartView::ReadLater => db
                    .get_posts(PostFilter {
                        only_unread: false,
                        only_bookmarked: false,
                        only_archived: false,
                        only_read_later: true,
                    })
                    .unwrap_or_default(),
                SmartView::Archived => db
                    .get_posts(PostFilter {
                        only_unread: false,
                        only_bookmarked: false,
                        only_archived: true,
                        only_read_later: false,
                    })
                    .unwrap_or_default(),
            },
            NavNode::Category(cat) => db.get_posts_by_category(cat).unwrap_or_default(),
        };

        self.posts = posts;
        if self.selected_index >= self.posts.len() && !self.posts.is_empty() {
            self.selected_index = self.posts.len() - 1;
        }
    }

    pub fn refresh_sidebar(&mut self) {
        let db = self.db.lock().unwrap();
        self.sidebar.load_categories(&db);
        self.sidebar.update_counts(&db);
    }

    pub fn next_post(&mut self) {
        if !self.posts.is_empty() {
            if self.selected_index < self.posts.len() - 1 {
                self.selected_index += 1;
            }
        }
    }

    pub fn previous_post(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn open_article(&mut self) {
        if let Some(post) = self.posts.get(self.selected_index) {
            let _ = self.db.lock().unwrap().mark_as_read(post.id);
            self.posts[self.selected_index].is_read = true;
            self.focus = FocusPane::Article;
            self.scroll_offset = 0;

            if !self.show_read {
                if let NavNode::SmartView(SmartView::Fresh) = &self.active_node {
                    self.refresh_sidebar();
                }
            }
        }
    }

    pub fn close_article(&mut self) {
        self.focus = FocusPane::Posts;
        self.scroll_offset = 0;

        if !self.show_read {
            if let NavNode::SmartView(SmartView::Fresh) = &self.active_node {
                self.remove_read_posts();
            }
        }
    }

    fn remove_read_posts(&mut self) {
        let old_id = self.posts.get(self.selected_index).map(|p| p.id);
        self.posts.retain(|p| !p.is_read);

        if let Some(old_id) = old_id {
            self.selected_index = self
                .posts
                .iter()
                .position(|p| p.id == old_id)
                .unwrap_or(0);
        }
        if self.selected_index >= self.posts.len() && !self.posts.is_empty() {
            self.selected_index = self.posts.len() - 1;
        }
    }

    pub fn toggle_bookmark(&mut self) {
        if let Some(post) = self.posts.get_mut(self.selected_index) {
            let _ = self.db.lock().unwrap().toggle_bookmark(post.id);
            post.is_bookmarked = !post.is_bookmarked;

            self.message = Some(if post.is_bookmarked {
                "★ Added to Starred".to_string()
            } else {
                "Removed from Starred".to_string()
            });

            if !post.is_bookmarked {
                if let NavNode::SmartView(SmartView::Starred) = &self.active_node {
                    self.posts.remove(self.selected_index);
                    if self.selected_index >= self.posts.len() && !self.posts.is_empty() {
                        self.selected_index = self.posts.len() - 1;
                    }
                }
            }
            self.refresh_sidebar();
        }
    }

    pub fn toggle_archived(&mut self) {
        if let Some(post) = self.posts.get_mut(self.selected_index) {
            let _ = self.db.lock().unwrap().mark_as_archived(post.id);
            post.is_archived = !post.is_archived;

            self.message = Some(if post.is_archived {
                "󰆧 Archived".to_string()
            } else {
                "Unarchived".to_string()
            });

            if !post.is_archived {
                if let NavNode::SmartView(SmartView::Archived) = &self.active_node {
                    self.posts.remove(self.selected_index);
                    if self.selected_index >= self.posts.len() && !self.posts.is_empty() {
                        self.selected_index = self.posts.len() - 1;
                    }
                }
            }
            self.refresh_sidebar();
        }
    }

    pub fn toggle_read_later(&mut self) {
        if let Some(post) = self.posts.get_mut(self.selected_index) {
            let _ = self.db.lock().unwrap().mark_as_read_later(post.id);
            post.is_read_later = !post.is_read_later;

            self.message = Some(if post.is_read_later {
                "󰃰 Added to Read Later".to_string()
            } else {
                "Removed from Read Later".to_string()
            });

            if !post.is_read_later {
                if let NavNode::SmartView(SmartView::ReadLater) = &self.active_node {
                    self.posts.remove(self.selected_index);
                    if self.selected_index >= self.posts.len() && !self.posts.is_empty() {
                        self.selected_index = self.posts.len() - 1;
                    }
                }
            }
            self.refresh_sidebar();
        }
    }

    pub fn toggle_read(&mut self) {
        if let Some(post) = self.posts.get_mut(self.selected_index) {
            let new_state = !post.is_read;
            if new_state {
                let _ = self.db.lock().unwrap().mark_as_read(post.id);
            } else {
                let _ = self.db.lock().unwrap().mark_as_unread(post.id);
            }
            post.is_read = new_state;

            self.message = Some(if new_state {
                "Marked as read".to_string()
            } else {
                "Marked as unread".to_string()
            });

            if !self.show_read && new_state {
                if let NavNode::SmartView(SmartView::Fresh) = &self.active_node {
                    self.posts.remove(self.selected_index);
                    if self.selected_index >= self.posts.len() && !self.posts.is_empty() {
                        self.selected_index = self.posts.len() - 1;
                    }
                }
            }
            self.refresh_sidebar();
        }
    }

    #[allow(dead_code)]
    pub fn delete_selected_post(&mut self) {
        if let Some(post) = self.posts.get(self.selected_index) {
            let post_title = post.title.clone();
            let post_id = post.id;
            if self.db.lock().unwrap().delete_post(post_id).is_ok() {
                self.posts.remove(self.selected_index);
                if self.selected_index >= self.posts.len() && !self.posts.is_empty() {
                    self.selected_index = self.posts.len() - 1;
                }
                self.refresh_sidebar();
                self.message = Some(format!("Deleted: {}", truncate_str(&post_title, 30)));
            }
        }
    }

    #[allow(dead_code)]
    pub fn delete_selected_feed(&mut self) {
        if let Some(feed) = self.feeds.get(self.selected_feed_index) {
            let feed_url = feed.url.clone();
            let feed_id = feed.id;
            if self.db.lock().unwrap().delete_feed(feed_id).is_ok() {
                self.reload_feeds();
                self.refresh_sidebar();
                self.reload_posts_for_active_node();
                self.message = Some(format!("Deleted feed: {}", truncate_str(&feed_url, 30)));
            }
        }
    }

    pub fn reload_feeds(&mut self) {
        self.feeds = self.db.lock().unwrap().get_feeds().unwrap_or_default();
        if self.selected_feed_index >= self.feeds.len() && !self.feeds.is_empty() {
            self.selected_feed_index = self.feeds.len() - 1;
        }
    }

    pub fn add_feed(&mut self, url: &str, category: &str) {
        if !url.trim().is_empty() {
            if self.db.lock().unwrap().add_feed_with_category(url, category).is_ok() {
                self.reload_feeds();
                self.refresh_sidebar();
                self.message = Some(format!("Added feed: {}", truncate_str(url, 40)));
            }
        }
    }

    pub fn add_category(&mut self, name: &str) {
        if !name.trim().is_empty() {
            if self.db.lock().unwrap().add_category(name).is_ok() {
                self.refresh_sidebar();
                self.message = Some(format!("Added category: {}", name));
            }
        }
    }

    #[allow(dead_code)]
    pub fn delete_selected_category(&mut self) {
        if let Some(cat) = self.sidebar.categories.get(self.sidebar.category_index).cloned() {
            if cat != "General" {
                if self.db.lock().unwrap().delete_category(&cat).is_ok() {
                    self.refresh_sidebar();
                    self.reload_posts_for_active_node();
                    self.message = Some(format!("Deleted category: {}", cat));
                }
            } else {
                self.message = Some("Cannot delete 'General' category".to_string());
            }
        }
    }

    pub fn toggle_show_read(&mut self) {
        self.show_read = !self.show_read;
        self.reload_posts_for_active_node();
        self.message = Some(if self.show_read {
            "Showing all posts".to_string()
        } else {
            "Showing unread only".to_string()
        });
    }

    pub fn copy_url_to_clipboard(&mut self) {
        if let Some(post) = self.posts.get(self.selected_index) {
            let url = &post.url;
            print!("\x1b]52;c;{}\x07", base64_encode(url));
            self.message = Some("URL copied to clipboard".to_string());
        }
    }

    pub fn get_selected_category(&self) -> String {
        self.sidebar
            .categories
            .get(self.sidebar.category_index)
            .cloned()
            .unwrap_or_else(|| "General".to_string())
    }
}

fn base64_encode(input: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let mut n: u32 = 0;
        for (i, &byte) in chunk.iter().enumerate() {
            n |= (byte as u32) << (16 - 8 * i);
        }

        let chars_to_output = chunk.len() + 1;
        for i in 0..4 {
            if i < chars_to_output {
                let idx = ((n >> (18 - 6 * i)) & 0x3F) as usize;
                result.push(ALPHABET[idx] as char);
            } else {
                result.push('=');
            }
        }
    }

    result
}
