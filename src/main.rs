use std::io::{self};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use app::App;
use chrono::Utc;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use network::network::{handle_tokio, Network, NetworkEvent};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Clear, Paragraph};
use util::constants::{GENERAL_HELP_TEXT, TICK_RATE};
use widgets::help::render_help_popup;

use crate::widgets::{
    chart::TokenChart,
    search::render_search_block,
    table::{render_table, StatefulTable},
    welcome::render_welcome,
};
use anyhow::Result;
use clap::Parser;
use models::event_handling::Event;
use models::states::InputMode;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders},
    Terminal,
};
use routes::{ActiveBlock, Route, RouteId};
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger,
};
use tokio::sync::Mutex;

mod app;
mod models;
mod network;
mod routes;
mod util;
mod widgets;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Etherscan Json-RPC URL
    #[arg(short, long, default_value = "https://eth.public-rpc.com")]
    etherscan_endpoint: String,
    /// Uniswap v3 Subgraph URL
    #[arg(
        short,
        long,
        default_value = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3/graphql"
    )]
    uniswap_v3_endpoint: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = std::fs::create_dir("logs");

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Error,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            std::fs::File::create(format!("logs/{}.log", Utc::now().format("%Y%m%d%H%M"))).unwrap(),
        ),
    ])
    .unwrap();

    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(TICK_RATE);

    // Start tick thread
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

    let app = Arc::new(Mutex::new(App::default()));
    let cloned_app = app.clone();
    let args = Args::parse();
    let (sync_network_tx, sync_network_rx) = mpsc::channel::<NetworkEvent>();

    // Start network thread
    std::thread::spawn(move || {
        let mut network = Network::default(app, args.etherscan_endpoint, args.uniswap_v3_endpoint);
        handle_tokio(sync_network_rx, &mut network);
    });

    let stdout = io::stdout();
    let stdout = stdout.lock();
    let backend = CrosstermBackend::new(stdout);

    let mut app = cloned_app.lock().await;
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

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(40),
                        Constraint::Percentage(40),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .margin(1)
                .split(f.size());

            if app.show_help {
                let (help, help_block, help_area) = render_help_popup(size);
                f.render_widget(Clear, help_area); //this clears out the background
                f.render_widget(help_block, help_area);
                f.render_widget(help, help_area);
            }

            if let Some((search_bar, search_bar_rect)) = render_search_block(chunks[0], &mut app) {
                // Render search bar at the stop
                f.render_widget(search_bar, search_bar_rect);
            }

            // Render welcome in the middle
            if let Some((
                welcome_banner,
                welcome_details,
                outer_block,
                welcome_banner_block,
                welcome_details_block,
            )) = render_welcome(&mut app, chunks[1])
            {
                f.render_widget(outer_block, chunks[1]);
                f.render_widget(welcome_banner, welcome_banner_block);
                f.render_widget(welcome_details, welcome_details_block);
            }

            // Render table at the bottom
            f.render_stateful_widget(render_table(&table, &mut app), chunks[2], &mut table.state);

            //Render the help text at the bottom
            let help_text = Paragraph::new(GENERAL_HELP_TEXT)
                .style(Style::default().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Plain),
                );
            f.render_widget(help_text, chunks[3]);
        })?;

        // #TODO: Move this to event handling
        match rx.recv()? {
            Event::Input(event) => match app.get_current_route().get_active_block() {
                ActiveBlock::SearchBar => match app.search_state.input_mode {
                    InputMode::Normal => match event.code {
                        KeyCode::Char('e') => {
                            app.search_state.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') => {
                            disable_raw_mode()?;
                            terminal.clear()?;
                            terminal.show_cursor()?;
                            break;
                        }
                        KeyCode::Char('h') => {
                            app.show_help = true;
                        }
                        KeyCode::Char('1') => {
                            app.change_active_block(ActiveBlock::Main);
                        }
                        KeyCode::Char('2') => {
                            app.change_active_block(ActiveBlock::MyPositions);
                        }
                        KeyCode::Esc => {
                            app.show_help = false;
                        }
                        _ => {}
                    },
                    InputMode::Editing => match event.code {
                        KeyCode::Esc => {
                            app.search_state.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char(c) => {
                            app.enter_char(c);
                        }
                        KeyCode::Left => {
                            app.move_cursor_left();
                        }
                        KeyCode::Right => {
                            app.move_cursor_right();
                        }
                        KeyCode::Backspace => {
                            app.delete_char();
                        }
                        KeyCode::Enter => {
                            app.search_state.input_mode = InputMode::Normal;
                            app.submit_search();
                        }
                        _ => {}
                    },
                },
                ActiveBlock::Main => match event.code {
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        terminal.clear()?;
                        terminal.show_cursor()?;
                        break;
                    }
                    KeyCode::Char('h') => {
                        app.show_help = true;
                    }
                    KeyCode::Char('s') => {
                        app.change_active_block(ActiveBlock::SearchBar);
                    }
                    KeyCode::Esc => {
                        app.show_help = false;
                    }
                    KeyCode::Char('2') => {
                        app.change_active_block(ActiveBlock::MyPositions);
                    }
                    _ => {}
                },
                ActiveBlock::MyPositions => match event.code {
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        terminal.clear()?;
                        terminal.show_cursor()?;
                        break;
                    }
                    KeyCode::Char('h') => {
                        app.show_help = true;
                    }
                    KeyCode::Char('s') => {
                        app.change_active_block(ActiveBlock::SearchBar);
                    }
                    KeyCode::Esc => {
                        app.show_help = false;
                    }
                    KeyCode::Up => {
                        table.previous();
                    }
                    KeyCode::Down => {
                        table.next();
                    }
                    KeyCode::Char('1') => {
                        app.change_active_block(ActiveBlock::Main);
                    }
                    KeyCode::Char('2') => {
                        app.change_active_block(ActiveBlock::MyPositions);
                    }
                    _ => {}
                },
            },
            Event::Tick => {
                token_chart.update();
            }
        }
    }

    Ok(())
}
