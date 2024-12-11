use log::debug;
use ratatui::{prelude::*, widgets::*};

use crate::{app::App, routes::ActiveBlock};

pub fn render_welcome<'a>(rect: Rect) -> (Paragraph<'a>, Paragraph<'a>, Rect, Rect) {
    debug!("Rendering welcome");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)].as_ref())
        .margin(1)
        .split(rect);

    let banner_block = chunks[0];
    let details_block = chunks[1];

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

    let details = Paragraph::new(vec![
        Line::from(
            Span::raw(format!("   {:<13}", "A CLI tool for querying Uniswap info",))
                .fg(Color::White),
        ),
        Line::from(
            Span::raw(format!(
                "   {:<13}: {}",
                "Version",
                env!("CARGO_PKG_VERSION")
            ))
            .fg(Color::White),
        ),
    ])
    .block(Block::default())
    .alignment(Alignment::Center);

    (banner, details, banner_block, details_block)
}
