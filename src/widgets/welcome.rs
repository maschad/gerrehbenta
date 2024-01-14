use ratatui::{prelude::*, widgets::*};

use crate::{app::App, routes::ActiveBlock};

pub fn render_welcome<'a>(app: &'a mut App) -> (Paragraph<'a>, Block<'a>) {
    let outer_block = Block::default()
        .title("Position Info")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(
            if let ActiveBlock::PositionInfo = app.get_current_route().get_active_block() {
                Color::Green
            } else {
                Color::White
            },
        ));

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

    (banner, outer_block)
}
