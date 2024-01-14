use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render_help_popup<'a>(size: Rect) -> (Paragraph<'a>, Block<'a>, Rect) {
    let block = Block::default()
        .title("Keybindings - Press Esc to close the popup")
        .borders(Borders::ALL);

    let input = Paragraph::new(vec![
        Line::from(
            Span::raw(format!(" {:<4}: {}", "s", "Move to the Search Bar")).fg(Color::White),
        ),
        Line::from(
            Span::raw(format!(" {:<4}: {}", "1", "Move to Positions Info area")).fg(Color::White),
        ),
        Line::from(
            Span::raw(format!(" {:<4}: {}", "2", "Move to the My Positions")).fg(Color::White),
        ),
    ])
    .style(Style::default().fg(Color::Green))
    .block(block.to_owned());

    let area = centered_rect(40, 10, size);

    (input, block, area)
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
