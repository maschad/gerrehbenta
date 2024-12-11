use std::io::{self};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};
use std::{panic, thread};

use app::{App, Mode};
use chrono::Utc;
use crossbeam_channel::{bounded, select, unbounded, Receiver, Sender};
use crossterm::{cursor, execute, terminal};
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use human_panic::{metadata, setup_panic, Metadata};
use lazy_static::lazy_static;

use parking_lot::{Mutex, RwLock};

use network::network::{handle_tokio, Network, NetworkEvent};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Clear, Paragraph};
use util::constants::{GENERAL_HELP_TEXT, TICK_RATE};

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

mod app;
mod event_handling;
mod models;
mod network;
mod render;
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
    // Uniswap limits endpoint
    #[arg(
        short,
        long,
        default_value = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3/graphql"
    )]
    uniswap_limits_endpoint: String,
}

lazy_static! {
    pub static ref REDRAW_REQUEST: (Sender<()>, Receiver<()>) = bounded(1);
    pub static ref DATA_RECEIVED: (Sender<()>, Receiver<()>) = bounded(1);
    // pub static ref OPTS: opts::Opts = opts::resolve_opts();
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_panic!();
    setup_panic_hook();
    setup_terminal();

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

    let request_redraw = REDRAW_REQUEST.0.clone();
    let data_received = DATA_RECEIVED.1.clone();
    let ui_events = setup_ui_events();

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    // let opts = OPTS.clone();

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

    // #TODO: Store starting mode locally
    let app = Arc::new(Mutex::new(App::default()));
    let cloned_app = app.clone();
    let args = Args::parse();
    let (_, sync_network_rx) = mpsc::channel::<NetworkEvent>();

    // Start network thread
    thread::spawn(move || {
        let mut network = Network::default(
            cloned_app,
            args.etherscan_endpoint,
            args.uniswap_v3_endpoint,
            args.uniswap_limits_endpoint,
        );
        handle_tokio(sync_network_rx, &mut network);
    });

    let cloned_app = app.clone();

    thread::spawn(move || {
        let app = cloned_app;

        let redraw_requested = REDRAW_REQUEST.1.clone();
        loop {
            select! {
                recv(redraw_requested) -> _ => {
                    let mut app = app.lock();

                    render::draw(&mut terminal, &mut app);
                }
                // Default redraw on every duration
                default(Duration::from_millis(500)) => {
                    let mut app = app.lock();
                    render::draw(&mut terminal, &mut app);
                }
            }
        }
    });

    loop {
        select! {
            // Notified that new data has been fetched from API, update widgets
            // so they can update their state with this new information
            recv(data_received) -> _ => {
                let mut app = app.lock();

                app.update();
            }
            recv(ui_events) -> message => {
                let mut app = app.lock();

                match message {
                    Ok(CEvent::Key(key_event)) => {
                        event_handling::handle_key_bindings(app.mode, key_event, &mut app, &request_redraw);
                    }
                    Ok(CEvent::Resize(..)) => {
                        let _ = request_redraw.try_send(());
                    }
                    _ => {}
                }
            }
        }
    }
}

fn setup_terminal() {
    let mut stdout = io::stdout();

    execute!(stdout, cursor::Hide).unwrap();
    execute!(stdout, terminal::EnterAlternateScreen).unwrap();

    execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();

    terminal::enable_raw_mode().unwrap();
}

fn cleanup_terminal() {
    let mut stdout = io::stdout();

    execute!(stdout, cursor::MoveTo(0, 0)).unwrap();
    execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();

    execute!(stdout, terminal::LeaveAlternateScreen).unwrap();
    execute!(stdout, cursor::Show).unwrap();

    terminal::disable_raw_mode().unwrap();
}

fn setup_ui_events() -> Receiver<CEvent> {
    let (sender, receiver) = unbounded();
    std::thread::spawn(move || loop {
        sender.send(crossterm::event::read().unwrap()).unwrap();
    });

    receiver
}

fn setup_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        cleanup_terminal();
        human_panic::handle_dump(
            &Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
            panic_info,
        );
    }));
}
