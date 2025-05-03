use std::collections::HashMap;

use ethers::types::Address;
use juniper::{graphql_object, Context};

use crate::network::ethers::types::AddressInfo;
use crate::widgets::enter_ens::EnterEnsState;

use super::position::Position;

pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Clone, Default)]
pub struct Database {
    pub positions: HashMap<String, Position>,
}

impl Context for Database {}

impl Database {
    pub fn default() -> Database {
        let mut positions = HashMap::<String, Position>::new();

        Database { positions }
    }

    pub fn get_position(&self, address: &str) -> Option<&Position> {
        self.positions.get(address)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Query;

#[graphql_object(context = Database)]
impl Query {
    fn position(
        #[graphql(context)] database: &Database,
        #[graphql(description = "Address of a position")] address: String,
    ) -> Option<&Position> {
        database.get_position(&address)
    }
}

pub struct AppSearchState {
    pub current_search_query: String,
    pub cursor_position: usize,
    pub input_mode: InputMode,
    pub is_invalid_search: bool,
    pub is_blank_search: bool,
    pub query: Option<Query>,
    pub is_searching: bool,
    pub ens_state: EnterEnsState,
}

impl Default for AppSearchState {
    fn default() -> Self {
        AppSearchState {
            current_search_query: "".to_owned(),
            cursor_position: 0,
            input_mode: InputMode::Normal,
            is_invalid_search: false,
            is_blank_search: false,
            is_searching: false,
            query: None,
            ens_state: EnterEnsState::new(),
        }
    }
}

impl AppSearchState {
    /// Resets the [`AppSearchState`] to its default state, albeit still enabled.
    pub fn reset(&mut self) {
        *self = AppSearchState {
            ..AppSearchState::default()
        }
    }

    /// Returns whether the [`AppSearchState`] has an invalid or blank search.
    pub fn is_invalid_or_blank_search(&self) -> bool {
        self.is_blank_search || self.is_invalid_search
    }
}
