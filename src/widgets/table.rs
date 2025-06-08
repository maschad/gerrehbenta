use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::{
    app::{App, ChartView},
    routes::ActiveBlock,
    widgets::chart::{render_volume_chart, TokenChart},
};

use crate::app::ChartTimeRange;
use chrono::{Duration, NaiveDateTime, TimeZone, Utc};

pub struct StatefulTable {
    pub state: TableState,
    pub items: Vec<Vec<String>>,
    pub charts: Vec<TokenChart>,
    pub token_day_datas: Vec<Vec<(f64, f64)>>,
}

impl StatefulTable {
    pub fn new() -> StatefulTable {
        let mut state = TableState::default();
        state.select(Some(0));
        StatefulTable {
            state,
            items: Vec::new(),
            charts: Vec::new(),
            token_day_datas: Vec::new(),
        }
    }

    pub fn update_positions(
        &mut self,
        positions: &[crate::models::position::Position],
        token_day_datas: &[Vec<(f64, f64)>],
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
                            .and_then(|d| d.volume_usd.parse::<f64>().ok())
                            .unwrap_or(0.0)
                    ),
                    if pos.liquidity.parse::<f64>().unwrap_or(0.0) > 0.0 {
                        "✓".to_string()
                    } else {
                        "X".to_string()
                    },
                    age_str,
                    {
                        let deposited0 = pos.deposited_token0.parse::<f64>().unwrap_or(0.0);
                        let deposited1 = pos.deposited_token1.parse::<f64>().unwrap_or(0.0);
                        let withdrawn0 = pos.withdrawn_token0.parse::<f64>().unwrap_or(0.0);
                        let withdrawn1 = pos.withdrawn_token1.parse::<f64>().unwrap_or(0.0);

                        let last = pos.pool.pool_hour_data.last();
                        let token0_price = last
                            .and_then(|d| d.token0_price.as_ref())
                            .or_else(|| Some(&pos.pool.token0_price))
                            .and_then(|p| p.parse::<f64>().ok())
                            .unwrap_or(0.0);
                        let token1_price = last
                            .and_then(|d| d.token1_price.as_ref())
                            .or_else(|| Some(&pos.pool.token1_price))
                            .and_then(|p| p.parse::<f64>().ok())
                            .unwrap_or(0.0);

                        let fees0 = (withdrawn0 - deposited0).max(0.0) * token0_price;
                        let fees1 = (withdrawn1 - deposited1).max(0.0) * token1_price;
                        format!("${:.2}", fees0 + fees1)
                    },
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

        // Store tokenDayDatas for each position
        self.token_day_datas = token_day_datas.to_vec();
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
    table: &mut StatefulTable,
    area: Rect,
    positions: &'a Vec<crate::models::position::Position>,
    chart_time_range: ChartTimeRange,
    chart_view: ChartView,
) {
    // Split the area into table, chart, and tab bar sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(10),   // Table
                Constraint::Min(10),   // Chart
                Constraint::Length(3), // Tab bar
            ]
            .as_ref(),
        )
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
            Constraint::Length(20),
        ],
    )
    .header(
        Row::new(vec!["Pool", "Token0", "Token1", "Value", "Fees"])
            .style(Style::default().fg(Color::Yellow))
            .height(1)
            .bottom_margin(5),
    )
    .block(Block::default().title("My Positions").borders(Borders::ALL))
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(table_widget, chunks[0], &mut table.state);

    // Render the chart for the selected position, filtered by time range and chart view
    if let Some(selected) = table.state.selected() {
        if let Some(pos) = positions.get(selected) {
            let now = chrono::Utc::now().timestamp() as f64;
            let (token0_data, token1_data, volume_data, is_hourly, data_warning) =
                match chart_time_range {
                    ChartTimeRange::OneDay | ChartTimeRange::OneWeek => {
                        let (cutoff, n_points, warn_age) =
                            if chart_time_range == ChartTimeRange::OneDay {
                                (now - 60.0 * 60.0 * 24.0, 24, 60.0 * 60.0 * 24.0)
                            } else {
                                (
                                    now - 60.0 * 60.0 * 24.0 * 7.0,
                                    168,
                                    60.0 * 60.0 * 24.0 * 7.0,
                                )
                            };
                        let mut t0: Vec<_> = pos
                            .pool
                            .pool_hour_data
                            .iter()
                            .filter(|d| d.period_start_unix as f64 >= cutoff)
                            .map(|d| {
                                (
                                    d.period_start_unix as f64,
                                    d.token0_price
                                        .as_ref()
                                        .unwrap_or(&"0.0".to_string())
                                        .parse::<f64>()
                                        .unwrap_or(0.0),
                                )
                            })
                            .collect();
                        let mut t1: Vec<_> = pos
                            .pool
                            .pool_hour_data
                            .iter()
                            .filter(|d| d.period_start_unix as f64 >= cutoff)
                            .map(|d| {
                                (
                                    d.period_start_unix as f64,
                                    d.token1_price
                                        .as_ref()
                                        .unwrap_or(&"0.0".to_string())
                                        .parse::<f64>()
                                        .unwrap_or(0.0),
                                )
                            })
                            .collect();
                        let mut v: Vec<_> = pos
                            .pool
                            .pool_hour_data
                            .iter()
                            .filter(|d| d.period_start_unix as f64 >= cutoff)
                            .map(|d| {
                                (
                                    d.period_start_unix as f64,
                                    d.volume_usd.parse::<f64>().unwrap_or(0.0),
                                )
                            })
                            .collect();
                        // Fallback: if no data in cutoff, use most recent N points
                        let mut fallback = false;
                        if t0.is_empty() || t1.is_empty() {
                            fallback = true;
                            let all_t0: Vec<_> = pos
                                .pool
                                .pool_hour_data
                                .iter()
                                .rev()
                                .take(n_points)
                                .map(|d| {
                                    (
                                        d.period_start_unix as f64,
                                        d.token0_price
                                            .as_ref()
                                            .unwrap_or(&"0.0".to_string())
                                            .parse::<f64>()
                                            .unwrap_or(0.0),
                                    )
                                })
                                .collect();
                            let all_t1: Vec<_> = pos
                                .pool
                                .pool_hour_data
                                .iter()
                                .rev()
                                .take(n_points)
                                .map(|d| {
                                    (
                                        d.period_start_unix as f64,
                                        d.token1_price
                                            .as_ref()
                                            .unwrap_or(&"0.0".to_string())
                                            .parse::<f64>()
                                            .unwrap_or(0.0),
                                    )
                                })
                                .collect();
                            let all_v: Vec<_> = pos
                                .pool
                                .pool_hour_data
                                .iter()
                                .rev()
                                .take(n_points)
                                .map(|d| {
                                    (
                                        d.period_start_unix as f64,
                                        d.volume_usd.parse::<f64>().unwrap_or(0.0),
                                    )
                                })
                                .collect();
                            t0 = all_t0.into_iter().collect::<Vec<_>>();
                            t1 = all_t1.into_iter().collect::<Vec<_>>();
                            v = all_v.into_iter().collect::<Vec<_>>();
                            t0.reverse();
                            t1.reverse();
                            v.reverse();
                        }
                        // Show warning if most recent data is too old
                        let data_warning =
                            t0.last().map_or(true, |(ts, _)| now - *ts > warn_age) || fallback;
                        (t0, t1, v, true, data_warning)
                    }
                    _ => {
                        let cutoff = match chart_time_range {
                            ChartTimeRange::OneMonth => now - 60.0 * 60.0 * 24.0 * 30.0,
                            ChartTimeRange::ThreeMonths => now - 60.0 * 60.0 * 24.0 * 90.0,
                            ChartTimeRange::SixMonths => now - 60.0 * 60.0 * 24.0 * 180.0,
                            ChartTimeRange::OneYear => now - 60.0 * 60.0 * 24.0 * 365.0,
                            ChartTimeRange::FiveYears => now - 60.0 * 60.0 * 24.0 * 365.0 * 5.0,
                            _ => now,
                        };
                        let mut t0: Vec<_> = pos
                            .pool
                            .pool_day_datas
                            .iter()
                            .filter(|d| d.date as f64 >= cutoff)
                            .map(|d| (d.date as f64, d.token0Price.parse::<f64>().unwrap_or(0.0)))
                            .collect();
                        t0.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                        let mut t1: Vec<_> = pos
                            .pool
                            .pool_day_datas
                            .iter()
                            .filter(|d| d.date as f64 >= cutoff)
                            .map(|d| (d.date as f64, d.token1Price.parse::<f64>().unwrap_or(0.0)))
                            .collect();
                        t1.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                        let v = table
                            .token_day_datas
                            .get(selected)
                            .cloned()
                            .unwrap_or_default();
                        // Show warning if most recent data is too old (e.g., last point older than 7 days)
                        let data_warning = t0
                            .last()
                            .map_or(true, |(ts, _)| now - *ts > 60.0 * 60.0 * 24.0 * 7.0);
                        (t0, t1, v, false, data_warning)
                    }
                };

            let no_data = token0_data.is_empty() && token1_data.is_empty();
            if no_data {
                use ratatui::widgets::{Paragraph, Wrap};
                let warning = Paragraph::new(vec![
                    ratatui::text::Line::from(Span::raw("No chart data available for the selected range. Try another range or check your data source."))
                ])
                .block(Block::default().title("Warning").borders(Borders::ALL))
                .wrap(Wrap { trim: true });
                frame.render_widget(warning, chunks[1]);
            } else {
                let mut chart = TokenChart::new();
                chart.token0_ticker = pos.token0.symbol.clone();
                chart.token1_ticker = pos.token1.symbol.clone();
                chart.is_hourly = is_hourly;
                match chart_view {
                    crate::app::ChartView::Price => {
                        chart.update_with_price_data(&token0_data, &token1_data);
                        let mut price_chart = render_volume_chart(&chart);
                        if data_warning {
                            use ratatui::widgets::{Paragraph, Wrap};
                            let warning = Paragraph::new(vec![ratatui::text::Line::from(
                                Span::raw("Warning: Data may be outdated or not recent!"),
                            )])
                            .block(Block::default().title("Data Warning").borders(Borders::ALL))
                            .wrap(Wrap { trim: true });
                            frame.render_widget(warning, chunks[1]);
                        } else {
                            frame.render_widget(price_chart, chunks[1]);
                        }
                    }
                    crate::app::ChartView::Volume => {
                        chart.update_with_price_data(&volume_data, &[]);
                        let mut volume_chart = render_volume_chart(&chart);
                        volume_chart = volume_chart
                            .block(Block::default().title("Volume (USD)").borders(Borders::ALL));
                        frame.render_widget(volume_chart, chunks[1]);
                    }
                }
            }
        }
    }

    // Render the time range tab bar
    let tab_titles: Vec<Span> = ChartTimeRange::ALL
        .iter()
        .map(|range| {
            if *range == chart_time_range {
                Span::styled(
                    range.as_str(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )
            } else {
                Span::styled(range.as_str(), Style::default().fg(Color::Cyan))
            }
        })
        .collect();
    let toggle_hint = match chart_view {
        crate::app::ChartView::Price => "[v] Volume",
        crate::app::ChartView::Volume => "[v] Price",
    };
    let tabs = ratatui::widgets::Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Range | Toggle: {}", toggle_hint)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .select(
            ChartTimeRange::ALL
                .iter()
                .position(|r| *r == chart_time_range)
                .unwrap_or(0),
        );
    frame.render_widget(tabs, chunks[2]);
}
