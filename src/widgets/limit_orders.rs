use crate::network::limit_orders::LimitOrder;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

pub struct LimitOrdersWidget {
    orders: Vec<LimitOrder>,
}

impl LimitOrdersWidget {
    pub fn new() -> Self {
        Self { orders: Vec::new() }
    }

    pub fn update(&mut self, orders: Vec<LimitOrder>) {
        self.orders = orders;
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default().title("Limit Orders").borders(Borders::ALL);

        let header = Row::new(vec![
            Cell::from("Token"),
            Cell::from("Deadline"),
            Cell::from("Start Amount"),
            Cell::from("End Amount"),
            Cell::from("Price USD"),
            Cell::from("Value USD"),
            Cell::from("Market Cap"),
            Cell::from("24h Volume"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));

        let rows: Vec<Row> = self
            .orders
            .iter()
            .map(|order| {
                Row::new(vec![
                    Cell::from(order.token.clone()),
                    Cell::from(order.deadline.clone()),
                    Cell::from(order.start_amount.clone()),
                    Cell::from(order.end_amount.clone()),
                    Cell::from(order.price_usd.clone().unwrap_or_else(|| "N/A".to_string())),
                    Cell::from(order.value_usd.clone()),
                    Cell::from(order.market_cap_usd.clone()),
                    Cell::from(order.volume_24h.clone()),
                ])
            })
            .collect();

        let widths = [
            Constraint::Length(10),
            Constraint::Length(20),
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(15),
            Constraint::Length(15),
        ];

        let table = Table::new(rows, widths).header(header).block(block);

        f.render_widget(table, area);
    }
}
