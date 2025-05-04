use ratatui::{
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
};

use crate::models::event_handling::SinSignal;
use chrono::{NaiveDateTime, TimeZone, Utc};

pub struct TokenChart {
    pub token0_prices: Vec<(f64, f64)>,
    pub token1_prices: Vec<(f64, f64)>,
    pub token0_ticker: String,
    pub token1_ticker: String,
    pub window: [f64; 2],
    pub max_price: f64,
}

impl TokenChart {
    pub fn new() -> TokenChart {
        TokenChart {
            token0_prices: Vec::new(),
            token1_prices: Vec::new(),
            token0_ticker: String::new(),
            token1_ticker: String::new(),
            window: [0.0, 0.0],
            max_price: 0.0,
        }
    }

    pub fn update_with_price_data(
        &mut self,
        token0_data: &[(f64, f64)],
        token1_data: &[(f64, f64)],
    ) {
        self.token0_prices = token0_data.to_vec();
        self.token1_prices = token1_data.to_vec();
        // Calculate window bounds
        if !self.token0_prices.is_empty() {
            self.window = [
                self.token0_prices.first().map(|(x, _)| *x).unwrap_or(0.0),
                self.token0_prices.last().map(|(x, _)| *x).unwrap_or(0.0),
            ];
        }
        // Find max price for y-axis scaling
        let max0 = self
            .token0_prices
            .iter()
            .map(|(_, y)| *y)
            .fold(0.0, f64::max);
        let max1 = self
            .token1_prices
            .iter()
            .map(|(_, y)| *y)
            .fold(0.0, f64::max);
        self.max_price = max0.max(max1);
    }
}

pub fn render_volume_chart<'a>(token_chart: &TokenChart) -> Chart {
    let x_labels = if !token_chart.token0_prices.is_empty() {
        let start = token_chart.window[0] as i64;
        let mid = ((token_chart.window[0] + token_chart.window[1]) / 2.0) as i64;
        let end = token_chart.window[1] as i64;
        vec![
            Span::styled(
                format!(
                    "{}",
                    Utc.timestamp_opt(start, 0)
                        .single()
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| start.to_string())
                ),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(
                Utc.timestamp_opt(mid, 0)
                    .single()
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| mid.to_string()),
            ),
            Span::styled(
                format!(
                    "{}",
                    Utc.timestamp_opt(end, 0)
                        .single()
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| end.to_string())
                ),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![]
    };
    let y_labels = if token_chart.max_price > 0.0 {
        vec![
            Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:.1}K", token_chart.max_price / 2000.0)),
            Span::styled(
                format!("{:.1}K", token_chart.max_price / 1000.0),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![]
    };
    let datasets = vec![
        Dataset::default()
            .name(token_chart.token0_ticker.as_str())
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Green))
            .data(&token_chart.token0_prices),
        Dataset::default()
            .name(token_chart.token1_ticker.as_str())
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Yellow))
            .data(&token_chart.token1_prices),
    ];
    Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(
                    "Token Prices (USD)",
                    Style::default()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .labels(x_labels)
                .bounds(token_chart.window),
        )
        .y_axis(
            Axis::default()
                .title("Price (USD)")
                .style(Style::default().fg(Color::Gray))
                .labels(y_labels)
                .bounds([0.0, token_chart.max_price]),
        )
}
