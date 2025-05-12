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

use chrono::{Duration, NaiveDateTime, TimeZone, Utc};

pub struct StatefulTable {
    pub state: TableState,
    pub items: Vec<Vec<String>>,
    pub charts: Vec<TokenChart>,
}

impl StatefulTable {
    pub fn new() -> StatefulTable {
        let mut state = TableState::default();
        state.select(Some(0));
        StatefulTable {
            state,
            items: Vec::new(),
            charts: Vec::new(),
        }
    }

    pub fn update_positions(
        &mut self,
        positions: &[crate::models::position::Position],
        _volume_data: &[(f64, f64)],
    ) {
        self.items = positions
            .iter()
            .map(|pos| {
                let age_str = pos.transaction.as_ref().map_or("N/A".to_string(), |t| {
                    if let Ok(ts) = t.timestamp.parse::<i64>() {
                        let dt = NaiveDateTime::from_timestamp_opt(ts, 0)
                            .unwrap_or_else(|| NaiveDateTime::from_timestamp_opt(0, 0).unwrap());
                        let now = Utc::now().naive_utc();
                        let duration = now - dt;
                        if duration.num_days() > 0 {
                            format!("{}d ago", duration.num_days())
                        } else if duration.num_hours() > 0 {
                            format!("{}h ago", duration.num_hours())
                        } else if duration.num_minutes() > 0 {
                            format!("{}m ago", duration.num_minutes())
                        } else {
                            "just now".to_string()
                        }
                    } else {
                        "N/A".to_string()
                    }
                });
                vec![
                    format!("{}/{}", pos.token0.symbol, pos.token1.symbol),
                    format!(
                        "${:.2}",
                        pos.pool
                            .pool_hour_data
                            .iter()
                            .last()
                            .unwrap()
                            .volume_usd
                            .parse::<f64>()
                            .unwrap_or(0.0)
                    ),
                    if pos.liquidity.parse::<f64>().unwrap_or(0.0) > 0.0 {
                        "✓".to_string()
                    } else {
                        "X".to_string()
                    },
                    age_str,
                ]
            })
            .collect();

        // Create charts for each position
        self.charts = positions
            .iter()
            .map(|pos| {
                let mut token0_data: Vec<(f64, f64)> = pos
                    .pool
                    .pool_hour_data
                    .iter()
                    .map(|d| {
                        (
                            d.period_start_unix,
                            d.token0_price
                                .as_ref()
                                .unwrap_or(&String::from("0.0"))
                                .parse::<f64>()
                                .unwrap_or(0.0),
                        )
                    })
                    .collect();
                token0_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                let mut token1_data: Vec<(f64, f64)> = pos
                    .pool
                    .pool_hour_data
                    .iter()
                    .map(|d| {
                        (
                            d.period_start_unix,
                            d.token1_price
                                .as_ref()
                                .unwrap_or(&String::from("0.0"))
                                .parse::<f64>()
                                .unwrap_or(0.0),
                        )
                    })
                    .collect();
                token1_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                // Set window using min and max
                let window = if let (Some(min), Some(max)) = (
                    token0_data.first().map(|x| x.0),
                    token0_data.last().map(|x| x.0),
                ) {
                    [min, max]
                } else {
                    [0.0, 0.0]
                };

                let mut chart = TokenChart::new();
                chart.token0_ticker = pos.token0.symbol.clone();
                chart.token1_ticker = pos.token1.symbol.clone();
                chart.window = window;
                chart.update_with_price_data(&token0_data, &token1_data);
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

pub fn render_table<'a>(frame: &mut Frame, table: &mut StatefulTable, area: Rect) {
    // Split the area into table and chart sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Min(10)].as_ref())
        .split(area);

    let table_widget = Table::new(
        table.items.iter().map(|item| {
            let height = item
                .iter()
                .map(|content| content.lines().count())
                .max()
                .unwrap_or(1);
            Row::new(item.clone()).height(height as u16)
        }),
        &[
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Length(20),
        ],
    )
    .header(
        Row::new(vec!["Pool", "Token0", "Token1", "Value"])
            .style(Style::default().fg(Color::Yellow))
            .height(1)
            .bottom_margin(5),
    )
    .block(Block::default().title("My Positions").borders(Borders::ALL))
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(table_widget, chunks[0], &mut table.state);

    // Render the volume chart for the selected position
    if let Some(selected) = table.state.selected() {
        if let Some(chart) = table.charts.get(selected) {
            let volume_chart = render_volume_chart(chart);
            frame.render_widget(volume_chart, chunks[1]);
        }
    }
}
