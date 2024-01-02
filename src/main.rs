use std::error::Error;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders},
    Terminal,
};

mod util;
mod widgets;

use crate::util::event::Event;

use crate::widgets::{
    chart::{render_chart, TokenChart},
    table::{render_table, StatefulTable},
    tabs::{render_tab_blocks, render_tab_titles, TabsState},
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

    let mut tabs = TabsState::new(vec!["1Min", "1H", "1D", "1M", "3M", "6M", "1Y"]);

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

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(50),
                        Constraint::Percentage(40),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .margin(5)
                .split(f.size());

            // Render the table at the top
            f.render_stateful_widget(render_table(&mut table), chunks[0], &mut table.state);

            // Render the chart at bottom
            f.render_widget(render_chart(&mut token_chart), chunks[1]);

            // Render Tab Titles at the top
            f.render_widget(render_tab_titles(&mut tabs), chunks[1]);

            // Render tabs at the bery bottom
            f.render_widget(render_tab_blocks(&mut tabs), chunks[2]);
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
