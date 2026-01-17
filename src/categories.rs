#[allow(dead_code)]
pub const CATEGORIES: &[&str] = &[
    "General",
    "Technology",
    "News",
    "Productivity",
    "Science",
    "Business",
    "Entertainment",
    "Sports",
    "Health",
    "Education",
];

#[allow(dead_code)]
pub struct CategorySelector {
    pub selected_index: usize,
}

#[allow(dead_code)]
impl CategorySelector {
    pub fn new() -> Self {
        CategorySelector {
            selected_index: 0,
        }
    }

    pub fn get_selected(&self) -> &str {
        CATEGORIES.get(self.selected_index).unwrap_or(&"General")
    }

    pub fn next(&mut self) {
        if self.selected_index < CATEGORIES.len() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    pub fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = CATEGORIES.len() - 1;
        }
    }
}

#[allow(dead_code)]
impl Default for CategorySelector {
    fn default() -> Self {
        Self::new()
    }
}
