pub enum InputMode {
    Normal,
    Editing,
}

// App holds the state of the application
pub struct App {
    /// Current input mode
    pub input_mode: InputMode,
    /// History of recorded messages
    pub messages: Vec<String>,
    // Current input into search bar
    pub input: String,
}

impl App {
    pub fn default() -> App {
        App {
            input: "".to_owned(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
        }
    }
}
