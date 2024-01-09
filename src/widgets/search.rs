use ratatui::{prelude::*, widgets::*};

use crate::app::App;

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
        .border_style(Style::default())
        .title(format!("Search by Address / ENS"))
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(searchbar_block);

    Some((input, searchbar))
}
