
use tui::widgets::TableState;

pub enum Event<I> {
	Input(I),
	Tick,
}


pub struct SinSignal {
    x: f64,
    interval: f64,
    period: f64,
    scale: f64,
}

impl SinSignal {
    pub fn new(interval: f64, period: f64, scale: f64) -> SinSignal {
        SinSignal {
            x: 0.0,
            interval,
            period,
            scale,
        }
    }
}


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


impl Iterator for SinSignal {
    type Item = (f64, f64);
    fn next(&mut self) -> Option<Self::Item> {
        let point = (self.x, (self.x * 1.0 / self.period).sin() * self.scale);
        self.x += self.interval;
        Some(point)
    }
}

pub struct StatefulTable<'a> {
    pub state: TableState,
    pub items: Vec<Vec<&'a str>>,
}

impl<'a> StatefulTable<'a> {
    pub fn new() -> StatefulTable<'a> {
        StatefulTable {
            state: TableState::default(),
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


pub struct TokenChart {
    pub signal: SinSignal,
    pub data: Vec<(f64, f64)>,
    pub window: [f64; 2],
}


impl TokenChart {

    pub fn new() -> TokenChart {
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