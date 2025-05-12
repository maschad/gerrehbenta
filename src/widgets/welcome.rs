use log::debug;
use ratatui::{prelude::*, widgets::*};

use super::enter_ens::{EnterENS, EnterEnsState};
use crate::{app::App, routes::ActiveBlock};

pub fn render_welcome<'a>(
    rect: Rect,
) -> (
    Paragraph<'a>,
    Paragraph<'a>,
    Paragraph<'a>,
    EnterENS,
    Rect,
    Rect,
    Rect,
    Rect,
) {
    debug!("Rendering welcome");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 10),
                Constraint::Ratio(1, 10),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(rect);

    let banner_block = chunks[0];
    let details_block = chunks[1];
    let prompt_message_block = chunks[2];
    let ens_block = chunks[3];

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
        Line::from(Span::raw(format!("{:<13}: {}", "Author", "Chad Nehemiah")).fg(Color::White)),
        Line::from(
            Span::raw(format!(" {:<13}: {}", "Version", env!("CARGO_PKG_VERSION")))
                .fg(Color::White),
        ),
    ])
    .block(Block::default())
    .alignment(Alignment::Center);

    // Create a message prompting the user to enter their ETH wallet or ENS
    let prompt_message = Paragraph::new("Please enter your ETH wallet or ENS")
        .alignment(Alignment::Center)
        .block(Block::default());

    let ens_widget = EnterENS {};

    (
        banner,
        details,
        prompt_message,
        ens_widget,
        banner_block,
        details_block,
        ens_block,
        prompt_message_block,
    )
}
