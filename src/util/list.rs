use ratatui::widgets::ListState;

#[derive(Clone)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
    pub header_size: usize,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
            header_size: 2,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 + self.header_size {
                    self.header_size
                } else {
                    i + 1
                }
            }
            None => self.header_size,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i <= self.header_size {
                    self.items.len() - 1 + self.header_size
                } else {
                    i - 1
                }
            }
            None => self.header_size,
        };
        self.state.select(Some(i));
    }

    pub fn get_selected_item_index(&self) -> Option<usize> {
        self.state.selected().map(|state| state - self.header_size)
    }

    /*
    fn unselect(&mut self) {
        self.state.select(None);
    }
    */
}
