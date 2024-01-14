use ratatui::{prelude::*, widgets::*};

pub fn render_welcome<'a>() -> (Paragraph<'a>, Block<'a>) {
    let outer_block = Block::default()
        .title("Position Info")
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);

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
