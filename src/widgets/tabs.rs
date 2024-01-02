use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Tabs},
};

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}
pub fn render_tab_blocks<'a>(tabs: &TabsState<'a>) -> Tabs<'a> {
    // Tabs
    let titles = tabs
        .titles
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            vec![
                Span::styled(first, Style::default().fg(Color::Yellow)),
                Span::styled(rest, Style::default().fg(Color::Green)),
            ]
        })
        .collect();

    let tab_blocks = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Timeline"))
        .select(tabs.index)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    tab_blocks
}

pub fn render_tab_titles<'a>(tabs: &TabsState<'a>) -> Block<'a> {
    let inner = match tabs.index {
        0 => Block::default().title("Minutes ").borders(Borders::ALL),
        1 => Block::default().title("Last Hour").borders(Borders::ALL),
        2 => Block::default().title("Last Day").borders(Borders::ALL),
        3 => Block::default().title("Last Month").borders(Borders::ALL),
        4 => Block::default()
            .title("Last Three Months")
            .borders(Borders::ALL),
        5 => Block::default()
            .title("Last Six Months")
            .borders(Borders::ALL),
        6 => Block::default().title("Last Year").borders(Borders::ALL),
        _ => unreachable!(),
    };
    inner
}
