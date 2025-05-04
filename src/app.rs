use ethers::types::NameOrAddress;
use std::sync::mpsc::Sender;

use crate::{
    models::states::AppSearchState,
    network::limit_orders::LimitOrder,
    network::network::NetworkEvent,
    routes::{ActiveBlock, Route},
};

// App holds the state of the application
pub struct App {
    /// Current mode
    pub mode: Mode,
    /// Previous mode
    pub previous_mode: Mode,
    /// Current input mode
    pub search_state: AppSearchState,
    /// Current ens address
    pub ens_address: Option<String>,
    /// whether to show help dialogue
    pub show_help: bool,
    /// Current route
    pub routes: Vec<Route>,
    /// The channel to send network events to
    pub network_txn: Option<Sender<NetworkEvent>>,
    /// Current limit orders
    pub limit_orders: Vec<LimitOrder>,
}
#[derive(PartialEq, Eq, Clone, Copy, Debug)]

pub enum Mode {
    Welcome,
    Main,
    Help,
    Search,
    LimitOrders,
    PoolInfo,
    MyPositions,
}

impl App {
    pub fn default() -> App {
        App {
            previous_mode: Mode::Welcome,
            mode: Mode::Welcome,
            search_state: AppSearchState::default(),
            ens_address: None,
            routes: vec![Route::default()],
            show_help: false,
            network_txn: None,
            limit_orders: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        // #TODO: Add update logic
    }

    pub fn update_limit_orders(&mut self, orders: Vec<LimitOrder>) {
        self.limit_orders = orders;
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
        self.search_state.is_searching = true;
        if let Some(network_txn) = &self.network_txn {
            if let Err(e) = network_txn.send(action) {
                self.search_state.is_searching = false;
                println!("Error from dispatch {}", e);
                //#TODO: handle network error
            };
        }
    }

    pub fn submit_search(&mut self) {
        if let Ok(name_or_address) = self
            .search_state
            .current_search_query
            .parse::<NameOrAddress>()
        {
            self.dispatch(NetworkEvent::GetENSAddressInfo { name_or_address })
        }

        self.reset_cursor();
    }
}
