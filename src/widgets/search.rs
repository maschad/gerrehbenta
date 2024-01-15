use ratatui::{prelude::*, widgets::*};

use crate::{app::App, models::states::InputMode, routes::ActiveBlock};

pub fn render_search_block<'a>(
    outer: Rect,
    app: &'a mut App,
) -> Option<(ratatui::widgets::Paragraph<'a>, Rect)> {
    let [searchbar, _, _] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Max(3), Constraint::Min(0), Constraint::Max(1)].as_ref())
        .split(outer)
    else {
        return None;
    };

    let searchbar_block = Block::default()
        .border_style(Style::default().fg(
            if let ActiveBlock::SearchBar = app.get_current_route().get_active_block() {
                Color::Green
            } else {
                Color::White
            },
        ))
        .title(format!(
            "Type either an ENS or Ethereum-based address to search for a position ({})",
            match app.search_state.input_mode {
                InputMode::Normal => "Press 'q' to exit, 'e' to start editing.",
                InputMode::Editing => "Press 'Esc' to stop editing, 'Enter' to search.",
            }
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);

    let input = Paragraph::new(app.search_state.current_search_query.as_str())
        .style(Style::default().fg(Color::White))
        .block(searchbar_block);

    Some((input, searchbar))
}
