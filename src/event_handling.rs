use crossbeam_channel::Sender;
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};

use crate::{
    app::{self, Mode},
    cleanup_terminal,
    models::states::InputMode,
    network::network::NetworkEvent,
    routes::ActiveBlock,
    widgets::table,
};

pub fn handle_key_bindings(
    mode: Mode,
    key_event: KeyEvent,
    app: &mut app::App,
    request_redraw: &Sender<()>,
) {
    match (mode, key_event.modifiers, key_event.code) {
        (_, KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            cleanup_terminal();
            std::process::exit(0);
        }
        (_, KeyModifiers::CONTROL, KeyCode::Char('l')) => {
            let _ = request_redraw.try_send(());
        }
        (Mode::Welcome, _, key_code) => match key_code {
            KeyCode::Char(c) => {
                app.search_state.ens_state.add_char(c);
                let _ = request_redraw.try_send(());
            }
            KeyCode::Backspace => {
                app.search_state.ens_state.del_char();
                let _ = request_redraw.try_send(());
            }
            KeyCode::Enter => {
                let search_string = app
                    .search_state
                    .ens_state
                    .get_search_ens_string()
                    .to_string();
                if !search_string.is_empty() {
                    app.search_state.is_searching = true;
                    app.search_state.ens_state.is_searching = true;
                    app.search_state.current_search_query = search_string;
                    app.submit_search();
                    let _ = request_redraw.try_send(());
                }
            }
            KeyCode::Char('h') => {
                app.show_help = true;
            }
            KeyCode::Char('q') => {
                cleanup_terminal();
                std::process::exit(0);
            }
            _ => {}
        },
        (_, _, key_code) => match app.get_current_route().get_active_block() {
            ActiveBlock::SearchBar => match app.search_state.input_mode {
                InputMode::Normal => match key_code {
                    KeyCode::Char('e') => {
                        app.search_state.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        cleanup_terminal();
                        std::process::exit(0);
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
                    KeyCode::Char('3') => {
                        app.mode = Mode::LimitOrders;
                        app.change_active_block(ActiveBlock::LimitOrders);
                        if let Some(network_txn) = &app.network_txn {
                            let _ = network_txn.send(NetworkEvent::FetchLimitOrders);
                        }
                    }
                    KeyCode::Esc => {
                        app.show_help = false;
                    }
                    _ => {}
                },
                InputMode::Editing => match key_code {
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
            ActiveBlock::Main => match key_code {
                KeyCode::Char('q') => {
                    cleanup_terminal();
                    std::process::exit(0);
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
                KeyCode::Char('3') => {
                    app.mode = Mode::LimitOrders;
                    app.change_active_block(ActiveBlock::LimitOrders);
                    if let Some(network_txn) = &app.network_txn {
                        let _ = network_txn.send(NetworkEvent::FetchLimitOrders);
                    }
                }
                _ => {}
            },
            ActiveBlock::MyPositions => match key_code {
                KeyCode::Char('q') => {
                    cleanup_terminal();
                    std::process::exit(0);
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
                    app.stateful_table.previous();
                    let _ = request_redraw.try_send(());
                }
                KeyCode::Down => {
                    app.stateful_table.next();
                    let _ = request_redraw.try_send(());
                }
                KeyCode::Char('1') => {
                    app.chart_time_range = crate::app::ChartTimeRange::OneDay;
                    let _ = request_redraw.try_send(());
                }
                KeyCode::Char('2') => {
                    app.chart_time_range = crate::app::ChartTimeRange::OneWeek;
                    let _ = request_redraw.try_send(());
                }
                KeyCode::Char('3') => {
                    app.chart_time_range = crate::app::ChartTimeRange::OneMonth;
                    let _ = request_redraw.try_send(());
                }
                KeyCode::Char('4') => {
                    app.chart_time_range = crate::app::ChartTimeRange::ThreeMonths;
                    let _ = request_redraw.try_send(());
                }
                KeyCode::Char('5') => {
                    app.chart_time_range = crate::app::ChartTimeRange::SixMonths;
                    let _ = request_redraw.try_send(());
                }
                KeyCode::Char('6') => {
                    app.chart_time_range = crate::app::ChartTimeRange::OneYear;
                    let _ = request_redraw.try_send(());
                }
                KeyCode::Char('7') => {
                    app.chart_time_range = crate::app::ChartTimeRange::FiveYears;
                    let _ = request_redraw.try_send(());
                }
                KeyCode::Char('v') | KeyCode::Tab => {
                    app.chart_view = match app.chart_view {
                        crate::app::ChartView::Price => crate::app::ChartView::Volume,
                        crate::app::ChartView::Volume => crate::app::ChartView::Price,
                    };
                    let _ = request_redraw.try_send(());
                }
                KeyCode::Left => {
                    let idx = crate::app::ChartTimeRange::ALL
                        .iter()
                        .position(|r| *r == app.chart_time_range)
                        .unwrap_or(0);
                    let new_idx = if idx == 0 {
                        crate::app::ChartTimeRange::ALL.len() - 1
                    } else {
                        idx - 1
                    };
                    app.chart_time_range = crate::app::ChartTimeRange::ALL[new_idx];
                    let _ = request_redraw.try_send(());
                }
                KeyCode::Right => {
                    let idx = crate::app::ChartTimeRange::ALL
                        .iter()
                        .position(|r| *r == app.chart_time_range)
                        .unwrap_or(0);
                    let new_idx = if idx == crate::app::ChartTimeRange::ALL.len() - 1 {
                        0
                    } else {
                        idx + 1
                    };
                    app.chart_time_range = crate::app::ChartTimeRange::ALL[new_idx];
                    let _ = request_redraw.try_send(());
                }
                _ => {}
            },
            ActiveBlock::LimitOrders => match key_code {
                KeyCode::Char('q') => {
                    cleanup_terminal();
                    std::process::exit(0);
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
                KeyCode::Char('1') => {
                    app.change_active_block(ActiveBlock::Main);
                }
                KeyCode::Char('2') => {
                    app.change_active_block(ActiveBlock::MyPositions);
                }
                _ => {}
            },
        },
    }
}
