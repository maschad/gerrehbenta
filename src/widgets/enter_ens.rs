use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, StatefulWidget, Widget, Wrap};

use super::utils::block;

pub struct EnterEnsState {
    search_ens_string: String,
    has_user_input: bool,
    error_msg: Option<String>,
}

impl EnterEnsState {
    pub fn new() -> EnterEnsState {
        EnterEnsState {
            search_ens_string: String::new(),
            has_user_input: false,
            error_msg: Some(String::new()),
        }
    }

    pub fn add_char(&mut self, c: char) {
        self.search_ens_string.push(c);
        self.has_user_input = true;
    }

    pub fn del_char(&mut self) {
        self.search_ens_string.pop();
    }

    pub fn reset(&mut self) {
        self.search_ens_string.drain(..);
        self.has_user_input = false;
        self.error_msg = None;
    }

    pub fn enter(&mut self) -> EnterEnsState {
        let mut new_state = EnterEnsState::new();
        new_state.search_ens_string = self.search_ens_string.clone().to_ascii_uppercase();
        new_state
    }

    pub fn get_search_ens_string(&self) -> &str {
        &self.search_ens_string
    }
}

pub struct EnterEnsWidget {}

impl StatefulWidget for EnterEnsWidget {
    type State = EnterEnsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let spans = if !state.has_user_input && state.error_msg.is_some() {
            Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::White)),
                Span::styled(
                    state.error_msg.as_ref().unwrap(),
                    Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::White)),
                Span::styled(
                    &state.search_ens_string,
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Cyan),
                ),
            ])
        };

        Paragraph::new(spans)
            .block(block::new(" Enter ENS "))
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
