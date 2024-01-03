use ratatui::{prelude::*, widgets::*};

use super::spinner::Spinner;

pub fn render_search_block<'a>(word: &str) -> Block<'a> {
    let searching_block = Block::default()
        .title(format!(
            "{} Searching for {word}",
            Spinner::default().to_string()
        ))
        .border_style(Style::default().fg(Color::Green))
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);

    searching_block
}
