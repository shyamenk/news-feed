use crate::db::Database;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SmartView {
    Fresh,
    Starred,
    ReadLater,
    Archived,
}

impl SmartView {
    pub fn title(&self) -> &'static str {
        match self {
            SmartView::Fresh => "Fresh",
            SmartView::Starred => "Starred",
            SmartView::ReadLater => "Read Later",
            SmartView::Archived => "Archived",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SmartView::Fresh => "󰈸",
            SmartView::Starred => "★",
            SmartView::ReadLater => "󰃰",
            SmartView::Archived => "󰆧",
        }
    }

    pub fn all() -> Vec<SmartView> {
        vec![
            SmartView::Fresh,
            SmartView::Starred,
            SmartView::ReadLater,
            SmartView::Archived,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NavNode {
    SmartView(SmartView),
    Category(String),
}

impl NavNode {
    pub fn title(&self) -> String {
        match self {
            NavNode::SmartView(sv) => sv.title().to_string(),
            NavNode::Category(name) => name.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn icon(&self) -> &'static str {
        match self {
            NavNode::SmartView(sv) => sv.icon(),
            NavNode::Category(_) => "󰉋",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Sidebar,
    Posts,
    Article,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarSection {
    SmartViews,
    Categories,
}

pub struct SidebarState {
    pub smart_views: Vec<SmartView>,
    pub categories: Vec<String>,
    pub section: SidebarSection,
    pub smart_view_index: usize,
    pub category_index: usize,
    pub counts: HashMap<NavNode, usize>,
    pub last_fetched: HashMap<NavNode, Instant>,
}

impl SidebarState {
    pub fn new() -> Self {
        SidebarState {
            smart_views: SmartView::all(),
            categories: vec![],
            section: SidebarSection::SmartViews,
            smart_view_index: 0,
            category_index: 0,
            counts: HashMap::new(),
            last_fetched: HashMap::new(),
        }
    }

    pub fn load_categories(&mut self, db: &Database) {
        self.categories = db.get_categories().unwrap_or_default();
        if self.categories.is_empty() {
            self.categories.push("General".to_string());
        }
    }

    pub fn update_counts(&mut self, db: &Database) {
        self.counts.insert(
            NavNode::SmartView(SmartView::Fresh),
            db.get_count("SELECT COUNT(*) FROM posts WHERE is_read = 0").unwrap_or(0),
        );
        self.counts.insert(
            NavNode::SmartView(SmartView::Starred),
            db.get_count("SELECT COUNT(*) FROM posts WHERE is_bookmarked = 1").unwrap_or(0),
        );
        self.counts.insert(
            NavNode::SmartView(SmartView::ReadLater),
            db.get_count("SELECT COUNT(*) FROM posts WHERE is_read_later = 1").unwrap_or(0),
        );
        self.counts.insert(
            NavNode::SmartView(SmartView::Archived),
            db.get_count("SELECT COUNT(*) FROM posts WHERE is_archived = 1").unwrap_or(0),
        );

        for cat in &self.categories {
            let count = db.get_count(&format!(
                "SELECT COUNT(*) FROM posts p JOIN feeds f ON p.feed_id = f.id WHERE f.category = '{}'",
                cat.replace("'", "''")
            )).unwrap_or(0);
            self.counts.insert(NavNode::Category(cat.clone()), count);
        }
    }

    pub fn get_count(&self, node: &NavNode) -> usize {
        *self.counts.get(node).unwrap_or(&0)
    }

    pub fn selected_node(&self) -> NavNode {
        match self.section {
            SidebarSection::SmartViews => {
                NavNode::SmartView(self.smart_views[self.smart_view_index].clone())
            }
            SidebarSection::Categories => {
                if let Some(cat) = self.categories.get(self.category_index) {
                    NavNode::Category(cat.clone())
                } else {
                    NavNode::SmartView(SmartView::Fresh)
                }
            }
        }
    }

    pub fn next(&mut self) {
        match self.section {
            SidebarSection::SmartViews => {
                if self.smart_view_index < self.smart_views.len() - 1 {
                    self.smart_view_index += 1;
                } else {
                    self.section = SidebarSection::Categories;
                    self.category_index = 0;
                }
            }
            SidebarSection::Categories => {
                if !self.categories.is_empty() && self.category_index < self.categories.len() - 1 {
                    self.category_index += 1;
                }
            }
        }
    }

    pub fn previous(&mut self) {
        match self.section {
            SidebarSection::SmartViews => {
                if self.smart_view_index > 0 {
                    self.smart_view_index -= 1;
                }
            }
            SidebarSection::Categories => {
                if self.category_index > 0 {
                    self.category_index -= 1;
                } else {
                    self.section = SidebarSection::SmartViews;
                    self.smart_view_index = self.smart_views.len() - 1;
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn is_stale(&self, node: &NavNode, stale_seconds: u64) -> bool {
        match self.last_fetched.get(node) {
            Some(instant) => instant.elapsed().as_secs() > stale_seconds,
            None => true,
        }
    }

    pub fn mark_fetched(&mut self, node: NavNode) {
        self.last_fetched.insert(node, Instant::now());
    }
}

impl Default for SidebarState {
    fn default() -> Self {
        Self::new()
    }
}
