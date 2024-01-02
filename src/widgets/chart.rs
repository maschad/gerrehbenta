use crate::util::event::SinSignal;

use ratatui::{
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset},
};

pub struct TokenChart {
    pub signal: SinSignal,
    pub data: Vec<(f64, f64)>,
    pub window: [f64; 2],
}

impl TokenChart {
    pub fn new() -> TokenChart {
        // #TODO: Dynamically dictate signal or line depending on selected time range by tab
        let mut signal = SinSignal::new(0.1, 2.0, 20.0);

        let data = signal.by_ref().take(200).collect::<Vec<(f64, f64)>>();

        TokenChart {
            signal,
            data,
            window: [0.0, 20.0],
        }
    }

    pub fn update(&mut self) {
        for _ in 0..10 {
            self.data.remove(0);
        }
        self.data.extend(self.signal.by_ref().take(10));
        self.window[0] += 1.0;
        self.window[1] += 1.0;
    }
}

pub fn render_chart<'a>(token_chart: &TokenChart) -> Chart {
    // Line Graph for selected token
    let x_labels = vec![
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
    ];

    // Chart data
    let dataset = vec![Dataset::default()
        .name("data")
        .marker(symbols::Marker::Dot)
        .style(Style::default().fg(Color::Cyan))
        .data(&token_chart.data)];

    // Chart Styling
    let chart = Chart::new(dataset)
        .block(
            Block::default()
                .title(Span::styled(
                    "Price",
                    Style::default()
                        .fg(Color::LightCyan)
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
                .title("Price")
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::styled("1800", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw("0"),
                    Span::styled("2420", Style::default().add_modifier(Modifier::BOLD)),
                ])
                .bounds([-20.0, 20.0]),
        );
    chart
}
