use std::error::Error;
use std::io::{self};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use app::{App, InputMode};
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
use tokio::sync::Mutex;
use widgets::search::render_search_block;
use widgets::welcome::render_welcome;

mod app;
mod models;
mod network;
mod util;
mod widgets;

use crate::util::event::Event;

use crate::widgets::{
    chart::{render_chart, TokenChart},
    table::{render_table, StatefulTable},
    tabs::{render_tab_titles, TabsState},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    let app = Arc::new(Mutex::new(App::default()));
    let mut app = app.lock().await;

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
    let stdout = stdout.lock();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut table = StatefulTable::new();
    let mut token_chart = TokenChart::new();

    let word = String::new();

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
                        Constraint::Percentage(10),
                        Constraint::Percentage(50),
                        Constraint::Percentage(40),
                    ]
                    .as_ref(),
                )
                .margin(5)
                .split(f.size());

            if let Some((search_bar, search_bar_rect)) = render_search_block(chunks[0], &mut app) {
                // Render search bar at the stop
                f.render_widget(search_bar, search_bar_rect);
            }

            // Render welcome in the middle
            f.render_widget(render_welcome(), chunks[1]);

            // Render table at the bottom
            f.render_stateful_widget(render_table(&mut table), chunks[2], &mut table.state);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('s') => {
                    app.input_mode = InputMode::Editing;
                }
                KeyCode::Down => {
                    table.next();
                }
                KeyCode::Up => {
                    table.previous();
                }
                KeyCode::Esc | KeyCode::Enter => {
                    break;
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
