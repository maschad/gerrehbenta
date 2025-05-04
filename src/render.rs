use crate::{
    app::{App, Mode},
    network::network::NetworkEvent,
    widgets::{
        table::{render_table, StatefulTable},
        welcome::render_welcome,
    },
};

use log::debug;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    prelude::Terminal,
    widgets::{Block, Clear, Tabs},
    Frame,
};

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    let current_size = terminal.size().unwrap_or_default();

    if current_size.width <= 10 || current_size.height <= 10 {
        return;
    }

    terminal
        .draw(|frame| {
            debug!("Drawing UI frame");

            // Clear the screen
            frame.render_widget(Clear, frame.area());

            // Set background color
            frame.render_widget(Block::default(), frame.area());

            match app.mode {
                Mode::Welcome => {
                    let layout = Layout::default()
                        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
                        .split(frame.area());

                    let (
                        banner,
                        details,
                        prompt_message,
                        ens_widget,
                        banner_block,
                        details_block,
                        ens_block,
                        prompt_message_block,
                    ) = render_welcome(layout[0]);
                    frame.render_widget(banner, banner_block);
                    frame.render_widget(details, details_block);
                    frame.render_widget(prompt_message, prompt_message_block);
                    frame.render_stateful_widget(
                        ens_widget,
                        ens_block,
                        &mut app.search_state.ens_state,
                    );
                }
                Mode::MyPositions => {
                    debug!("Drawing My Positions");
                    let mut table = StatefulTable::new();
                    table.update_positions(&app.positions, &[]);
                    render_table(frame, &table, app, frame.area());
                }
                _ => {}
            }
        })
        .unwrap();
}

fn draw_main<B: Backend>(frame: &mut Frame, app: &mut App, area: Rect) {
    // layout[0] - Header
    // layout[1] - Main widget
    let mut layout = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(10)].as_ref())
        .split(layout[0]);
}

pub fn add_padding(mut rect: Rect, n: u16, direction: PaddingDirection) -> Rect {
    match direction {
        PaddingDirection::Top => {
            rect.y += n;
            rect.height = rect.height.saturating_sub(n);
            rect
        }
        PaddingDirection::Bottom => {
            rect.height = rect.height.saturating_sub(n);
            rect
        }
        PaddingDirection::Left => {
            rect.x += n;
            rect.width = rect.width.saturating_sub(n);
            rect
        }
        PaddingDirection::Right => {
            rect.width = rect.width.saturating_sub(n);
            rect
        }
        PaddingDirection::All => {
            rect.y += n;
            rect.height = rect.height.saturating_sub(n * 2);

            rect.x += n;
            rect.width = rect.width.saturating_sub(n * 2);

            rect
        }
    }
}

#[allow(dead_code)]
pub enum PaddingDirection {
    Top,
    Bottom,
    Left,
    Right,
    All,
}
