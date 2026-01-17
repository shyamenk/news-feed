#[allow(dead_code)]
pub struct TextInput {
    pub value: String,
    pub cursor_position: usize,
}

#[allow(dead_code)]
impl TextInput {
    pub fn new() -> Self {
        TextInput {
            value: String::new(),
            cursor_position: 0,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.value.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.value.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.value.len() {
            self.cursor_position += 1;
        }
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_position = 0;
    }
}

#[allow(dead_code)]
impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}
