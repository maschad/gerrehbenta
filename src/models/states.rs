pub enum InputMode {
    Normal,
    Editing,
}

pub struct Query {
    pub query: String,
    pub error_message: Option<String>,
}

pub struct AppSearchState {
    pub current_search_query: String,
    pub cursor_position: usize,
    pub input_mode: InputMode,
    pub is_invalid_search: bool,
    pub is_blank_search: bool,
    pub query: Option<Query>,
    pub is_searching: bool,
}

impl Default for AppSearchState {
    fn default() -> Self {
        AppSearchState {
            current_search_query: "".to_owned(),
            cursor_position: 0,
            input_mode: InputMode::Normal,
            is_invalid_search: false,
            is_blank_search: false,
            is_searching: false,
            query: None,
        }
    }
}

impl AppSearchState {
    /// Resets the [`AppSearchState`] to its default state, albeit still enabled.
    pub fn reset(&mut self) {
        *self = AppSearchState {
            ..AppSearchState::default()
        }
    }

    /// Returns whether the [`AppSearchState`] has an invalid or blank search.
    pub fn is_invalid_or_blank_search(&self) -> bool {
        self.is_blank_search || self.is_invalid_search
    }
}
