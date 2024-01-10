use crate::{
    models::states::InputMode,
    routes::{ActiveBlock, Route},
};

// App holds the state of the application
pub struct App {
    /// Current input mode
    pub input_mode: InputMode,
    /// History of recorded messages
    pub messages: Vec<String>,
    // Current input into search bar
    pub input: String,
    /// whether to show help dialogue
    pub show_help: bool,
    /// Position of the cursor
    pub cursor_position: usize,
    /// Current route
    pub routes: Vec<Route>,
}

impl App {
    pub fn default() -> App {
        App {
            cursor_position: 0,
            input: "".to_owned(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            routes: vec![Route::default()],
            show_help: false,
        }
    }

    pub fn pop_current_route(&mut self) {
        if self.routes.len() > 1 {
            self.routes.pop();
        }
    }

    pub fn get_current_route(&self) -> Route {
        self.routes
            .last()
            .map_or(Route::default(), |route| route.to_owned())
    }

    pub fn set_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    pub fn change_active_block(&mut self, active_block: ActiveBlock) {
        let current_route = self.get_current_route();
        self.routes.pop();
        self.routes
            .push(Route::new(current_route.get_id(), active_block));
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    pub fn paste(&mut self, data: String) {
        self.input = format!("{}{}", self.input, data);
        for _ in 0..data.len() {
            self.move_cursor_right();
        }
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    pub fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }
}
