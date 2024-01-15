use ethers::types::NameOrAddress;
use std::sync::mpsc::Sender;

use crate::{
    models::states::AppSearchState,
    network::network::NetworkEvent,
    routes::{ActiveBlock, Route},
};

// App holds the state of the application
pub struct App {
    /// Current input mode
    pub search_state: AppSearchState,
    /// History of recorded messages
    pub messages: Vec<String>,
    /// whether to show help dialogue
    pub show_help: bool,
    /// Current route
    pub routes: Vec<Route>,
    /// Whether the app is loading
    pub is_loading: bool,
    /// The channel to send network events to
    pub network_txn: Option<Sender<NetworkEvent>>,
}

impl App {
    pub fn default() -> App {
        App {
            search_state: AppSearchState::default(),
            messages: Vec::new(),
            routes: vec![Route::default()],
            show_help: false,
            is_loading: false,
            network_txn: None,
        }
    }

    pub fn pop_current_route(&mut self) {
        if self.routes.len() > 1 {
            self.routes.pop();
        }
    }

    pub fn get_current_route(&self) -> Route {
        self.routes
            .last()
            .map_or(Route::default(), |route| route.to_owned())
    }

    pub fn set_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    pub fn change_active_block(&mut self, active_block: ActiveBlock) {
        let current_route = self.get_current_route();
        self.routes.pop();
        self.routes
            .push(Route::new(current_route.get_id(), active_block));
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.search_state.cursor_position.saturating_sub(1);
        self.search_state.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.search_state.cursor_position.saturating_add(1);
        self.search_state.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        self.search_state
            .current_search_query
            .insert(self.search_state.cursor_position, new_char);

        self.move_cursor_right();
    }

    pub fn paste(&mut self, data: String) {
        self.search_state.current_search_query =
            format!("{}{}", self.search_state.current_search_query, data);
        for _ in 0..data.len() {
            self.move_cursor_right();
        }
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.search_state.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.search_state.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self
                .search_state
                .current_search_query
                .chars()
                .take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self
                .search_state
                .current_search_query
                .chars()
                .skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.search_state.current_search_query =
                before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.search_state.current_search_query.len())
    }

    pub fn reset_cursor(&mut self) {
        self.search_state.cursor_position = 0;
    }

    // Send a network event to the network thread
    pub fn dispatch(&mut self, action: NetworkEvent) {
        // `is_loading` will be set to false again after the async action has finished in network.rs
        self.is_loading = true;
        if let Some(network_txn) = &self.network_txn {
            if let Err(e) = network_txn.send(action) {
                self.is_loading = false;
                println!("Error from dispatch {}", e);
                //#TODO: handle network error
            };
        }
    }

    pub fn submit_search(&mut self) -> String {
        if let Ok(name_or_address) = self
            .search_state
            .current_search_query
            .parse::<NameOrAddress>()
        {
            self.dispatch(NetworkEvent::GetENSAddressInfo {
                name_or_address,
                is_searching: true,
            })
        }

        let message = self.search_state.current_search_query.to_owned();

        self.search_state.current_search_query.clear();
        self.reset_cursor();
        message
    }
}
