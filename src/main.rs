mod util;

use crate::util::event::SinSignal;

use crate::util::event::Event;

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
    text::Span,
    widgets::{Axis,Block, BorderType, Borders, Cell, Chart, Dataset, Row, Table, TableState},
    symbols,
    Terminal,
};

pub struct StatefulTable<'a> {
    state: TableState,
    items: Vec<Vec<&'a str>>,
}

impl<'a> StatefulTable<'a> {
    fn new() -> StatefulTable<'a> {
        StatefulTable {
            state: TableState::default(),
            items: vec![
                vec!["ETH", "$2,400", "-0.08%"],
                vec!["USDC", "$1.01", "-0.02%"],
                vec!["BTC", "$40,000", "+20.54%"],
                vec!["UNI", "$21.30", "+10.00%"],
                vec!["DAI", "$1.00", "0.00%"],
            ],
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}


struct TokenChart {
    signal: SinSignal,
    data: Vec<(f64, f64)>,
    window: [f64; 2],
}


impl TokenChart {

    fn new() -> TokenChart {
        let mut signal = SinSignal::new(0.2, 3.0, 18.0);

        let data = signal.by_ref().take(200).collect::<Vec<(f64, f64)>>();

        TokenChart {
            signal,
            data,
            window: [0.0, 20.0],
        }
    }

    fn update(&mut self) {

        for _ in 0..10 {
            self.data.remove(0);
        }
        self.data.extend(self.signal.by_ref().take(5));
        self.window[0] += 1.0;
        self.window[1] += 1.0
    }
}


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
            let rects = Layout::default()
                .constraints([Constraint::Percentage(100)].as_ref())
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
            f.render_stateful_widget(t, rects[0], &mut table.state);

            // Line Graph for selected token
            let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                ]
                .as_ref(),
            )
            .split(size);

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

            let dataset = vec![Dataset::default()
                    .name("data")
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&token_chart.data)];

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
                        .title("X Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels(x_labels)
                        .bounds(token_chart.window),
                )
                .y_axis(
                    Axis::default()
                        .title("Y Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels(vec![
                            Span::styled("-20", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw("0"),
                            Span::styled("20", Style::default().add_modifier(Modifier::BOLD)),
                        ])
                        .bounds([-20.0, 20.0]),
                );

            f.render_widget(chart, chunks[0]);

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
                _ => {}
            },
            Event::Tick => {
                token_chart.update();
            }
        }
    }
    Ok(())
}