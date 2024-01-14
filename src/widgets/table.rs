use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
};

use crate::{app::App, routes::ActiveBlock};

pub struct StatefulTable<'a> {
    pub state: TableState,
    pub items: Vec<Vec<&'a str>>,
}

impl<'a> StatefulTable<'a> {
    pub fn new() -> StatefulTable<'a> {
        StatefulTable {
            state: TableState::default(),
            // #TODO: Pull token prices and populate here
            items: vec![
                vec!["USDC/ETH", "$2,400", "✓", "2d"],
                vec!["ETH/MATIC", "$100", "X", "10d"],
                vec!["USDC/TETH", "$2,000", "✓", "2w"],
            ],
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub fn render_table<'a>(table: &StatefulTable<'a>, app: &'a mut App) -> Table<'a> {
    // Table Layout
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::LightBlue);
    let header_cells = ["Name", "Fees", "In-Range", "Age"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(5);
    let rows = table.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(*c));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });

    // Stateful Table for positions
    let widths = &[
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(
                    if let ActiveBlock::MyPositions = app.get_current_route().get_active_block() {
                        Color::Green
                    } else {
                        Color::White
                    },
                ))
                .title("My Positions"),
        );

    table
}
