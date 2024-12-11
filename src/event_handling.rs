use crossbeam_channel::Sender;
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};

use crate::{
    app::{self, Mode},
    cleanup_terminal,
    models::states::InputMode,
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
                KeyCode::Up => {}
                KeyCode::Down => {}
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
