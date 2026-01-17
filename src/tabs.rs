#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Dashboard,
    Category,
    Favourite,
    ReadLater,
    Archived,
    FeedManager,
}

impl Tab {
    pub fn title(&self) -> String {
        match self {
            Tab::Dashboard => "Dashboard".to_string(),
            Tab::Category => "Category".to_string(),
            Tab::Favourite => "Favourite".to_string(),
            Tab::ReadLater => "Read Later".to_string(),
            Tab::Archived => "Archived".to_string(),
            Tab::FeedManager => "Feeds".to_string(),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Tab::Dashboard => "󰋜 ",
            Tab::Category => "󰻞 ",
            Tab::Favourite => "󰃀 ",
            Tab::ReadLater => "󰃰 ",
            Tab::Archived => "󰆧 ",
            Tab::FeedManager => "󰑫 ",
        }
    }
}

pub struct TabState {
    pub tabs: Vec<Tab>,
    pub active_index: usize,
}

impl TabState {
    pub fn new() -> Self {
        TabState {
            tabs: vec![
                Tab::Dashboard,
                Tab::Category,
                Tab::Favourite,
                Tab::ReadLater,
                Tab::Archived,
                Tab::FeedManager,
            ],
            active_index: 0,
        }
    }

    pub fn next(&mut self) {
        if self.active_index < self.tabs.len() - 1 {
            self.active_index += 1;
        } else {
            self.active_index = 0;
        }
    }

    pub fn previous(&mut self) {
        if self.active_index > 0 {
            self.active_index -= 1;
        } else {
            self.active_index = self.tabs.len() - 1;
        }
    }

    pub fn set_active(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_index = index;
        }
    }

    pub fn get_active(&self) -> &Tab {
        &self.tabs[self.active_index]
    }
}

impl Default for TabState {
    fn default() -> Self {
        Self::new()
    }
}
