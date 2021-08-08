use tui::{
    layout::{Constraint},
    style::{Color, Modifier, Style},
    widgets::{Cell, Row, Table,TableState}

};


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
                vec!["ETH", "$2,400", "-0.08%"],
                vec!["USDC", "$1.01", "-0.02%"],
                vec!["BTC", "$40,000", "+20.54%"],
                vec!["UNI", "$21.30", "+10.00%"],
                vec!["DAI", "$1.00", "0.00%"],
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


pub fn render_table<'a>(table: &StatefulTable<'a>) -> Table<'a> {

    // Table Layout
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = ["Name", "Price", "Volume"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Red)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
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

    // Stateful Table for tokens
    let t = Table::new(rows)
        .header(header)
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Max(10),
        ]);

    t
}