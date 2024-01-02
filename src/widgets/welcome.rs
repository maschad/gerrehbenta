use ratatui::{prelude::*, widgets::*};

pub fn render_welcome<'a>(rect: Rect) -> Block<'a> {
    let welcome_block = Block::default()
        .title("Welcome")
        .border_style(Style::default().fg(Color::White))
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);

    let [logo_rect, details_rect] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)].as_ref())
        .margin(1)
        .split(rect)
    else {
        return;
    };

    let banner = Paragraph::new(Text::from(
        cfonts::render(cfonts::Options {
            text: String::from("gerrehbenta"),
            font: cfonts::Fonts::FontBlock,
            ..cfonts::Options::default()
        })
        .text,
    ))
    .wrap(Wrap { trim: false })
    .alignment(Alignment::Center);

    welcome_block
}
