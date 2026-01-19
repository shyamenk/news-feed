#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    FreshFeed,
    Category,
    Favourite,
    ReadLater,
    Archived,
    FeedManager,
}

#[allow(dead_code)]
impl Tab {
    pub fn title(&self) -> String {
        match self {
            Tab::FreshFeed => "Fresh".to_string(),
            Tab::Category => "Category".to_string(),
            Tab::Favourite => "Favourite".to_string(),
            Tab::ReadLater => "Read Later".to_string(),
            Tab::Archived => "Archived".to_string(),
            Tab::FeedManager => "Feeds".to_string(),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Tab::FreshFeed => "󰈸 ",
            Tab::Category => "󰻞 ",
            Tab::Favourite => "󰃀 ",
            Tab::ReadLater => "󰃰 ",
            Tab::Archived => "󰆧 ",
            Tab::FeedManager => "󰑫 ",
        }
    }
}

#[allow(dead_code)]
pub struct TabState {
    pub tabs: Vec<Tab>,
    pub active_index: usize,
}

#[allow(dead_code)]
impl TabState {
    pub fn new() -> Self {
        TabState {
            tabs: vec![
                Tab::FreshFeed,
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
