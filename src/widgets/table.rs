use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::{
    app::App,
    routes::ActiveBlock,
    widgets::chart::{render_volume_chart, TokenChart},
};

pub struct StatefulTable {
    pub state: TableState,
    pub items: Vec<Vec<String>>,
    pub charts: Vec<TokenChart>,
}

impl StatefulTable {
    pub fn new() -> StatefulTable {
        StatefulTable {
            state: TableState::default(),
            items: Vec::new(),
            charts: Vec::new(),
        }
    }

    pub fn update_positions(
        &mut self,
        positions: &[crate::models::position::Position],
        volume_data: &[(f64, f64)],
    ) {
        self.items = positions
            .iter()
            .map(|pos| {
                vec![
                    format!("{}/{}", pos.token0.symbol, pos.token1.symbol),
                    format!(
                        "${:.2}",
                        pos.pool.volume_token0.parse::<f64>().unwrap_or(0.0)
                            * pos.pool.token0_price.parse::<f64>().unwrap_or(0.0)
                    ),
                    if pos.liquidity.parse::<f64>().unwrap_or(0.0) > 0.0 {
                        "✓".to_string()
                    } else {
                        "X".to_string()
                    },
                    format!(
                        "{}",
                        pos.transaction
                            .as_ref()
                            .map_or("N/A".to_string(), |t| t.timestamp.clone())
                    ),
                ]
            })
            .collect();

        // Create charts for each position
        self.charts = positions
            .iter()
            .map(|_| {
                let mut chart = TokenChart::new();
                chart.update_with_volume_data(volume_data);
                chart
            })
            .collect();
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

pub fn render_table<'a>(
    frame: &mut Frame,
    stateful_table: &StatefulTable,
    app: &'a mut App,
    area: Rect,
) {
    // Split the area into table and chart sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(0)])
        .split(area);

    // Render the table
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::LightBlue);
    let header_cells = ["Name", "Fees", "In-Range", "Age"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(5);
    let rows = stateful_table.items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(c.as_str()));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });

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

    frame.render_stateful_widget(table, chunks[0], &mut stateful_table.state.clone());

    // Render the volume chart for the selected position
    if let Some(selected) = stateful_table.state.selected() {
        if let Some(chart) = stateful_table.charts.get(selected) {
            let volume_chart = render_volume_chart(chart);
            frame.render_widget(volume_chart, chunks[1]);
        }
    }
}
