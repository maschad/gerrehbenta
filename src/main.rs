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

use log::debug;
use parking_lot::{Mutex, RwLock};

use network::network::{handle_tokio, Network, NetworkEvent};
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
mod theme;
mod util;
mod widgets;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Etherscan Json-RPC URL
    #[arg(short, long, default_value = "https://eth.llamarpc.com/")]
    etherscan_endpoint: String,
    /// Uniswap v3 Subgraph URL
    #[arg(
        short = 'v',
        long,
        default_value = "https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3/graphql"
    )]
    uniswap_v3_endpoint: String,
    // Uniswap limits endpoint
    #[arg(
        short = 'l',
        long,
        default_value = "https://api.uniswap.org/v1/limit-orders?orderStatus=open&chainId=1&limit=100&sortKey=createdAt&desc=true"
    )]
    uniswap_limits_endpoint: String,
}

lazy_static! {
    pub static ref REDRAW_REQUEST: (Sender<()>, Receiver<()>) = bounded(1);
    pub static ref DATA_RECEIVED: (Sender<()>, Receiver<()>) = bounded(1);
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_panic!();
    setup_panic_hook();
    setup_terminal();
    setup_logger();

    debug!("Application starting up");

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    let tick_rate = Duration::from_millis(TICK_RATE);

    let request_redraw = REDRAW_REQUEST.0.clone();
    let data_received = DATA_RECEIVED.1.clone();
    let ui_events = setup_ui_events();

    // let opts = OPTS.clone();

    // #TODO: Store starting mode locally
    let app = Arc::new(Mutex::new(App::default()));
    let cloned_app = app.clone();

    thread::spawn(move || {
        let app = cloned_app;
        let redraw_requested = REDRAW_REQUEST.1.clone();

        debug!("Starting UI thread");

        loop {
            select! {
                recv(redraw_requested) -> _ => {
                    let mut app = app.lock();

                    render::draw(&mut terminal, &mut app);
                }
                // Default redraw on every duration
                default(tick_rate) => {
                    let mut app = app.lock();
                    render::draw(&mut terminal, &mut app);
                }
            }
        }
    });

    let cloned_app = app.clone();
    let args: Args = Args::parse();
    let (sync_network_tx, sync_network_rx) = mpsc::channel::<NetworkEvent>();
    app.lock().network_txn = Some(sync_network_tx);

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

fn setup_logger() {
    let _ = std::fs::create_dir("logs");

    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        std::fs::File::create(format!("logs/{}.log", Utc::now().format("%Y%m%d%H%M"))).unwrap(),
    )])
    .unwrap();
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
    debug!("Setting up UI events");
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
