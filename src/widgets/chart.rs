use ratatui::{
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset},
};

use crate::models::event_handling::SinSignal;

pub struct TokenChart {
    pub data: Vec<(f64, f64)>,
    pub window: [f64; 2],
    pub max_volume: f64,
}

impl TokenChart {
    pub fn new() -> TokenChart {
        TokenChart {
            data: Vec::new(),
            window: [0.0, 0.0],
            max_volume: 0.0,
        }
    }

    pub fn update_with_volume_data(&mut self, volume_data: &[(f64, f64)]) {
        self.data = volume_data.to_vec();

        // Calculate window bounds
        if !self.data.is_empty() {
            self.window = [
                self.data.first().map(|(x, _)| *x).unwrap_or(0.0),
                self.data.last().map(|(x, _)| *x).unwrap_or(0.0),
            ];

            // Find max volume for y-axis scaling
            self.max_volume = self.data.iter().map(|(_, y)| *y).fold(0.0, f64::max);
        }
    }
}

pub fn render_volume_chart<'a>(token_chart: &TokenChart) -> Chart {
    // X-axis labels (time)
    let x_labels = if !token_chart.data.is_empty() {
        vec![
            Span::styled(
                format!("{}", token_chart.window[0]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!(
                "{}",
                (token_chart.window[0] + token_chart.window[1]) / 2.0
            )),
            Span::styled(
                format!("{}", token_chart.window[1]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![]
    };

    // Y-axis labels (volume)
    let y_labels = if token_chart.max_volume > 0.0 {
        vec![
            Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{:.1}K", token_chart.max_volume / 2000.0)),
            Span::styled(
                format!("{:.1}K", token_chart.max_volume / 1000.0),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![]
    };

    // Chart data
    let dataset = vec![Dataset::default()
        .name("Volume")
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(Color::Green))
        .data(&token_chart.data)];

    // Chart Styling
    let chart = Chart::new(dataset)
        .block(
            Block::default()
                .title(Span::styled(
                    "24h Volume",
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
                .title("Volume (USD)")
                .style(Style::default().fg(Color::Gray))
                .labels(y_labels)
                .bounds([0.0, token_chart.max_volume]),
        );
    chart
}
