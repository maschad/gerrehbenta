use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget, Wrap},
};

pub struct EnterEnsState {
    search_ens_string: String,
    has_user_input: bool,
    error_msg: Option<String>,
    pub is_searching: bool,
}

impl EnterEnsState {
    pub fn new() -> EnterEnsState {
        EnterEnsState {
            search_ens_string: String::new(),
            has_user_input: false,
            error_msg: None,
            is_searching: false,
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

    pub fn set_error(&mut self, error: String) {
        self.error_msg = Some(error);
        self.has_user_input = false;
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

pub struct EnterENS {}

impl StatefulWidget for EnterENS {
    type State = EnterEnsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let loading_spinner = if state.is_searching { "⏳" } else { "" };

        let input_style = if state.is_searching {
            Style::default().fg(Color::DarkGray)
        } else if state.error_msg.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Cyan)
        };

        let spans = if !state.has_user_input && state.error_msg.is_some() {
            Line::from(vec![Span::styled(
                state.error_msg.as_ref().unwrap(),
                Style::default().fg(Color::Red),
            )])
        } else {
            Line::from(vec![Span::styled(
                format!("{} {}", state.search_ens_string, loading_spinner),
                input_style,
            )])
        };

        let block = if state.is_searching {
            Block::new()
                .title(" Searching... ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::DarkGray))
        } else if state.error_msg.is_some() {
            Block::new()
                .title(" Enter ENS ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Red))
        } else {
            Block::new()
                .title(" Enter ENS ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Green))
        };

        Paragraph::new(spans)
            .block(block)
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
