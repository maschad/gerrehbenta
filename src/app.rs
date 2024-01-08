enum InputMode {
    Normal,
    Editing,
}

// App holds the state of the application
pub struct App {
    /// Current value of the input box
    input: Input,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
}

impl App {
    fn default() -> App {
        App {
            input: Input::default(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
        }
    }
}
