use crate::db::Database;

#[allow(dead_code)]
pub struct CategorySelector {
    pub categories: Vec<String>,
    pub selected_index: usize,
}

#[allow(dead_code)]
impl CategorySelector {
    pub fn new() -> Self {
        CategorySelector {
            categories: vec!["General".to_string()],
            selected_index: 0,
        }
    }

    pub fn load_from_db(&mut self, db: &Database) {
        if let Ok(cats) = db.get_categories() {
            if !cats.is_empty() {
                self.categories = cats;
            }
        }
        if !self.categories.contains(&"General".to_string()) {
            self.categories.insert(0, "General".to_string());
        }
    }

    pub fn get_selected(&self) -> &str {
        self.categories.get(self.selected_index).map(|s| s.as_str()).unwrap_or("General")
    }

    pub fn next(&mut self) {
        if !self.categories.is_empty() && self.selected_index < self.categories.len() - 1 {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }

    pub fn previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if !self.categories.is_empty() {
            self.selected_index = self.categories.len() - 1;
        }
    }
}

impl Default for CategorySelector {
    fn default() -> Self {
        Self::new()
    }
}
