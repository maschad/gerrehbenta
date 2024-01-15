use ratatui::{prelude::*, widgets::*};

use crate::{app::App, routes::ActiveBlock};

pub fn render_welcome<'a>(
    app: &'a mut App,
    rect: Rect,
) -> Option<(Paragraph<'a>, Paragraph<'a>, Block<'a>, Rect, Rect)> {
    let outer_block = Block::default()
        .title("Position Info")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(
            if let ActiveBlock::Main = app.get_current_route().get_active_block() {
                Color::Green
            } else {
                Color::White
            },
        ));

    let [banner_block, details_block] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)].as_ref())
        .margin(1)
        .split(rect)
    else {
        return None;
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

    let details = Paragraph::new(vec![
        Line::from(
            Span::raw(format!(
                "   {:<13}",
                "A CLI for managing your Uniswap liquidity positions",
            ))
            .fg(Color::White),
        ),
        Line::from(Span::raw(format!("   {:<13}: {}", "Version", "v0.1.0")).fg(Color::White)),
    ])
    .block(Block::default())
    .alignment(Alignment::Center);

    Some((banner, details, outer_block, banner_block, details_block))
}
