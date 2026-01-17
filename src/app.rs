use crate::db::{Database, Post, PostFilter};
use crate::tabs::TabState;
use crate::input::TextInput;
use crate::categories::CategorySelector;
use crate::stats::AppStats;
use crate::ascii_art::get_random_quote;
use std::sync::{Arc, Mutex};

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

pub enum CurrentScreen {
    Home,
    Article,
}

pub enum InputMode {
    Normal,
    AddingFeed,
    SelectingCategoryForFeed,
    SelectingCategoryForView,
    AddingCategory,
}

pub enum ViewContext {
    List,
    Article,
}

pub struct CategoryViewState {
    pub categories: Vec<String>,
    pub selected_index: usize,
    pub active_category: Option<String>,
    pub dropdown_open: bool,
}

impl CategoryViewState {
    pub fn new() -> Self {
        CategoryViewState {
            categories: vec![],
            selected_index: 0,
            active_category: None,
            dropdown_open: false,
        }
    }

    pub fn next(&mut self) {
        if !self.categories.is_empty() && self.selected_index < self.categories.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn get_selected(&self) -> Option<&String> {
        self.categories.get(self.selected_index)
    }

    pub fn select_current(&mut self) {
        if let Some(cat) = self.categories.get(self.selected_index) {
            self.active_category = Some(cat.clone());
            self.dropdown_open = false;
        }
    }
}

impl Default for CategoryViewState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct App {
    pub db: Arc<Mutex<Database>>,
    pub posts: Vec<Post>,
    pub current_screen: CurrentScreen,
    pub view_context: ViewContext,
    pub tabs: TabState,
    pub selected_index: usize,
    pub scroll_offset: u16,
    pub exit: bool,
    pub message: Option<String>,
    pub is_loading: bool,
    pub input_mode: InputMode,
    pub text_input: TextInput,
    pub category_selector: CategorySelector,
    pub feeds: Vec<crate::db::Feed>,
    pub selected_feed_index: usize,
    pub category_view: CategoryViewState,
    pub cached_stats: AppStats,
    pub dashboard_quote: &'static str,
}

impl App {
    pub fn new(db: Database) -> Self {
        let db_arc = Arc::new(Mutex::new(db));
        let feeds = db_arc.lock().unwrap().get_feeds().unwrap_or_default();
        let categories = db_arc.lock().unwrap().get_categories().unwrap_or_default();
        let cached_stats = AppStats::from_db(&db_arc.lock().unwrap()).unwrap_or_default();

        let mut category_view = CategoryViewState::new();
        category_view.categories = categories;

        App {
            db: db_arc,
            posts: vec![],
            current_screen: CurrentScreen::Home,
            view_context: ViewContext::List,
            tabs: TabState::new(),
            selected_index: 0,
            scroll_offset: 0,
            exit: false,
            message: None,
            is_loading: true,
            input_mode: InputMode::Normal,
            text_input: TextInput::new(),
            category_selector: CategorySelector::new(),
            feeds,
            selected_feed_index: 0,
            category_view,
            cached_stats,
            dashboard_quote: get_random_quote(),
        }
    }

    pub fn refresh_cached_stats(&mut self) {
        if let Ok(stats) = AppStats::from_db(&self.db.lock().unwrap()) {
            self.cached_stats = stats;
        }
    }

    pub fn refresh_categories(&mut self) {
        if let Ok(categories) = self.db.lock().unwrap().get_categories() {
            self.category_view.categories = categories;
            if self.category_view.selected_index >= self.category_view.categories.len() {
                self.category_view.selected_index = 0;
            }
        }
    }

    pub fn next(&mut self) {
        if !self.posts.is_empty() {
            if self.selected_index < self.posts.len() - 1 {
                self.selected_index += 1;
            } else {
                self.selected_index = 0;
            }
        }
    }

    pub fn previous(&mut self) {
        if !self.posts.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.posts.len() - 1;
            }
        }
    }

    pub fn next_feed(&mut self) {
        if !self.feeds.is_empty() {
            if self.selected_feed_index < self.feeds.len() - 1 {
                self.selected_feed_index += 1;
            } else {
                self.selected_feed_index = 0;
            }
        }
    }

    pub fn previous_feed(&mut self) {
        if !self.feeds.is_empty() {
            if self.selected_feed_index > 0 {
                self.selected_feed_index -= 1;
            } else {
                self.selected_feed_index = self.feeds.len() - 1;
            }
        }
    }

    pub fn delete_selected_feed(&mut self) {
        if let Some(feed) = self.feeds.get(self.selected_feed_index) {
            let feed_url = feed.url.clone();
            let feed_id = feed.id;
            if self.db.lock().unwrap().delete_feed(feed_id).is_ok() {
                self.reload_feeds();
                self.refresh_categories();
                self.refresh_cached_stats();
                self.message = Some(format!("Deleted: {}", feed_url));
                if self.selected_feed_index >= self.feeds.len() && !self.feeds.is_empty() {
                    self.selected_feed_index = self.feeds.len() - 1;
                }
            }
        }
    }

    pub fn open_article(&mut self) {
        if let Some(post) = self.posts.get(self.selected_index) {
            let _ = self.db.lock().unwrap().mark_as_read(post.id);
            self.posts[self.selected_index].is_read = true;
            self.current_screen = CurrentScreen::Article;
            self.scroll_offset = 0;
        }
    }

    pub fn toggle_bookmark(&mut self) {
        if let Some(post) = self.posts.get_mut(self.selected_index) {
            let _ = self.db.lock().unwrap().toggle_bookmark(post.id);
            post.is_bookmarked = !post.is_bookmarked;

            self.message = Some(if post.is_bookmarked {
                "Added to Favourites".to_string()
            } else {
                "Removed from Favourites".to_string()
            });

            if !post.is_bookmarked && matches!(self.tabs.get_active(), crate::tabs::Tab::Favourite) {
                self.posts.remove(self.selected_index);
                if self.selected_index >= self.posts.len() && !self.posts.is_empty() {
                    self.selected_index = self.posts.len() - 1;
                }
            }
            self.refresh_cached_stats();
        }
    }

    pub fn toggle_archived(&mut self) {
        if let Some(post) = self.posts.get_mut(self.selected_index) {
            let _ = self.db.lock().unwrap().mark_as_archived(post.id);
            post.is_archived = !post.is_archived;

            self.message = Some(if post.is_archived {
                "Archived".to_string()
            } else {
                "Unarchived".to_string()
            });

            if !post.is_archived && matches!(self.tabs.get_active(), crate::tabs::Tab::Archived) {
                self.posts.remove(self.selected_index);
                if self.selected_index >= self.posts.len() && !self.posts.is_empty() {
                    self.selected_index = self.posts.len() - 1;
                }
            }
            self.refresh_cached_stats();
        }
    }

    pub fn toggle_read_later(&mut self) {
        if let Some(post) = self.posts.get_mut(self.selected_index) {
            let _ = self.db.lock().unwrap().mark_as_read_later(post.id);
            post.is_read_later = !post.is_read_later;

            self.message = Some(if post.is_read_later {
                "Added to Read Later".to_string()
            } else {
                "Removed from Read Later".to_string()
            });

            if !post.is_read_later && matches!(self.tabs.get_active(), crate::tabs::Tab::ReadLater) {
                self.posts.remove(self.selected_index);
                if self.selected_index >= self.posts.len() && !self.posts.is_empty() {
                    self.selected_index = self.posts.len() - 1;
                }
            }
            self.refresh_cached_stats();
        }
    }

    pub fn delete_selected_post(&mut self) {
        if let Some(post) = self.posts.get(self.selected_index) {
            let post_title = post.title.clone();
            let post_id = post.id;
            if self.db.lock().unwrap().delete_post(post_id).is_ok() {
                self.posts.remove(self.selected_index);
                if self.selected_index >= self.posts.len() && !self.posts.is_empty() {
                    self.selected_index = self.posts.len() - 1;
                }
                self.refresh_cached_stats();
                self.message = Some(format!("Deleted: {}", truncate_str(&post_title, 30)));
            }
        }
    }

    pub fn switch_to_tab(&mut self, tab_index: usize) {
        self.tabs.set_active(tab_index);
        self.reload_posts_for_current_tab();
        self.selected_index = 0;
    }

    pub fn reload_posts_for_current_tab(&mut self) {
        use crate::tabs::Tab;
        let db = self.db.lock().unwrap();

        let posts = match self.tabs.get_active() {
            Tab::Dashboard => vec![],
            Tab::Category => {
                if let Some(ref cat) = self.category_view.active_category {
                    db.get_posts_by_category(cat).unwrap_or_default()
                } else {
                    vec![]
                }
            }
            Tab::Favourite => db
                .get_posts(PostFilter {
                    only_unread: false,
                    only_bookmarked: true,
                    only_archived: false,
                    only_read_later: false,
                })
                .unwrap_or_default(),
            Tab::ReadLater => db
                .get_posts(PostFilter {
                    only_unread: false,
                    only_bookmarked: false,
                    only_archived: false,
                    only_read_later: true,
                })
                .unwrap_or_default(),
            Tab::Archived => db
                .get_posts(PostFilter {
                    only_unread: false,
                    only_bookmarked: false,
                    only_archived: true,
                    only_read_later: false,
                })
                .unwrap_or_default(),
            Tab::FeedManager => vec![],
        };

        self.posts = posts;
        if self.selected_index >= self.posts.len() && !self.posts.is_empty() {
            self.selected_index = self.posts.len() - 1;
        }
    }

    pub fn reload_feeds(&mut self) {
        self.feeds = self.db.lock().unwrap().get_feeds().unwrap_or_default();
        if self.selected_feed_index >= self.feeds.len() && !self.feeds.is_empty() {
            self.selected_feed_index = self.feeds.len() - 1;
        }
    }

    pub fn add_category(&mut self, name: &str) {
        if !name.trim().is_empty() && !self.category_view.categories.contains(&name.to_string()) {
            if self.db.lock().unwrap().add_category(name).is_ok() {
                self.refresh_categories();
                self.message = Some(format!("Added category: {}", name));
            }
        }
    }

    pub fn delete_selected_category(&mut self) {
        if let Some(cat) = self.category_view.get_selected().cloned() {
            if cat != "General" {
                if self.db.lock().unwrap().delete_category(&cat).is_ok() {
                    if self.category_view.active_category.as_ref() == Some(&cat) {
                        self.category_view.active_category = None;
                        self.posts.clear();
                    }
                    self.refresh_categories();
                    self.refresh_cached_stats();
                    self.message = Some(format!("Deleted category: {}", cat));
                }
            } else {
                self.message = Some("Cannot delete 'General' category".to_string());
            }
        }
    }
}
