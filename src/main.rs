mod util;


use crate::util::event:: {
    Event,
    StatefulTable,
    TabsState,
    TokenChart
};


use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use std::{error::Error};


use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Axis,Block, BorderType, Borders, Cell, Chart, Dataset, Row, Table, Tabs},
    symbols,
    Terminal,
};


fn main() -> Result<(), Box<dyn Error>> {

    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut table = StatefulTable::new();

    let mut token_chart = TokenChart::new();

    let mut tabs = TabsState::new(vec!["1Min","1H", "1D", "1M", "3M", "6M", "1Y"]);


    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            // Wrapping block for a group
            // Just draw the block and the group on the same area and build the group
            // with at least a margin of 1
            let size = f.size();

            let block = Block::default()
                .borders(Borders::ALL)
                .title("Top Tokens")
                .border_type(BorderType::Thick);
            f.render_widget(block, size);


            // Table Layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(40), Constraint::Percentage(10)].as_ref())
                .margin(5)
                .split(f.size());

            let selected_style = Style::default().add_modifier(Modifier::REVERSED);
            let normal_style = Style::default().bg(Color::Blue);
            let header_cells = ["Name", "Price", "Volume"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
            let header = Row::new(header_cells)
                .style(normal_style)
                .height(1)
                .bottom_margin(1);
            let rows = table.items.iter().map(|item| {
                let height = item
                    .iter()
                    .map(|content| content.chars().filter(|c| *c == '\n').count())
                    .max()
                    .unwrap_or(0)
                    + 1;
                let cells = item.iter().map(|c| Cell::from(*c));
                Row::new(cells).height(height as u16).bottom_margin(1)
            });

            // Stateful Table for tokens
            let t = Table::new(rows)
                .header(header)
                .highlight_style(selected_style)
                .highlight_symbol(">> ")
                .widths(&[
                    Constraint::Percentage(50),
                    Constraint::Length(30),
                    Constraint::Max(10),
                ]);
            f.render_stateful_widget(t, chunks[0], &mut table.state);

            // Line Graph for selected token
            let x_labels = vec![
                Span::styled(
                    format!("{}", token_chart.window[0]),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{}", (token_chart.window[0] + token_chart.window[1]) / 2.0)),
                Span::styled(
                    format!("{}", token_chart.window[1]),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ];

            // Chart data
            let dataset = vec![Dataset::default()
                    .name("data")
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&token_chart.data)];

            // Chart Styling
            let chart = Chart::new(dataset)
            .block(
                Block::default()
                .title(Span::styled(
                    "Price"
                ,
                Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
            )
            .x_axis(
                Axis::default()
                        .title("Time")
                        .style(Style::default().fg(Color::Gray))
                        .labels(x_labels)
                        .bounds(token_chart.window),
                )
                .y_axis(
                    Axis::default()
                        .title("Price")
                        .style(Style::default().fg(Color::Gray))
                        .labels(vec![
                            Span::styled("1800", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw("0"),
                            Span::styled("2420", Style::default().add_modifier(Modifier::BOLD)),
                        ])
                        .bounds([-20.0, 20.0]),
                );

            //Render the chart at bottom
            f.render_widget(chart, chunks[1]);

            // Tabs
            let titles = tabs.titles.iter().map(|t| {
                let (first, rest) = t.split_at(1);
                Spans::from(vec![
                    Span::styled(first, Style::default().fg(Color::Yellow)),
                    Span::styled(rest, Style::default().fg(Color::Green)),
                ])
            }).collect();

            let tab_blocks = Tabs::new(titles)
                .block(Block::default().borders(Borders::ALL).title("Timeline"))
                .select(tabs.index)
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(Color::Black),
            );

            f.render_widget(tab_blocks, chunks[2]);

            let inner = match tabs.index {
                0 => Block::default().title("Minutes ").borders(Borders::ALL),
                1 => Block::default().title("Last Hour").borders(Borders::ALL),
                2 => Block::default().title("Last Day").borders(Borders::ALL),
                3 => Block::default().title("Last Month").borders(Borders::ALL),
                4 => Block::default().title("Last Three Months").borders(Borders::ALL),
                5 => Block::default().title("Last Six Months").borders(Borders::ALL),
                6 => Block::default().title("Last Year").borders(Borders::ALL),
                _ => unreachable!(),
            };
            f.render_widget(inner, chunks[1]);

        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Down => {
                    table.next();
                }
                KeyCode::Up => {
                    table.previous();
                }
                KeyCode::Right => tabs.next(),
                KeyCode::Left => tabs.previous(),
                _ => {}
            },
            Event::Tick => {
                token_chart.update();
            }
        }
    }
    Ok(())
}