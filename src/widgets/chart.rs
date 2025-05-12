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
    pub min_price: f64,
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
            min_price: 0.0,
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
        // Find min/max price for y-axis scaling
        let all_prices = self
            .token0_prices
            .iter()
            .chain(self.token1_prices.iter())
            .map(|(_, y)| *y)
            .collect::<Vec<_>>();
        let min_price = all_prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_price = all_prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        // If all prices are the same, add a small buffer
        let (min_price, max_price) = if min_price == max_price {
            (min_price * 0.95, max_price * 1.05 + 0.01)
        } else {
            (min_price, max_price)
        };
        self.max_price = max_price;
        self.min_price = min_price;
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
    let y_labels = if token_chart.max_price > 0.0 || token_chart.min_price < 0.0 {
        let min = token_chart.min_price;
        let max = token_chart.max_price;
        let mid = (min + max) / 2.0;
        vec![
            Span::styled(
                format!("{:.2}", min),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{:.2}", mid)),
            Span::styled(
                format!("{:.2}", max),
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
                .bounds([token_chart.min_price, token_chart.max_price]),
        )
}
