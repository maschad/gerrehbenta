use chrono::Utc;

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub struct Spinner {
    elements: Vec<String>,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            elements: SPINNER.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        }
    }
}

impl ToString for Spinner {
    fn to_string(&self) -> String {
        let cycle = 1500; //millisec
        self.elements[((Utc::now().timestamp_millis() % cycle)
            / (cycle / self.elements.len() as i64)) as usize]
            .to_owned()
    }
}
