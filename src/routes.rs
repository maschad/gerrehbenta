use crate::{models::position::Position, network::ethers::types::AddressInfo};

#[derive(Clone)]
pub enum RouteId {
    Welcome,
    MyPositions(Option<AddressInfo>),
    PositionInfo(Option<Position>),
    LimitOrders,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActiveBlock {
    SearchBar,
    Main,
    MyPositions,
    LimitOrders,
}

#[derive(Clone)]
pub struct Route {
    id: RouteId,
    active_block: ActiveBlock,
}

impl Route {
    pub fn new(id: RouteId, active_block: ActiveBlock) -> Self {
        Self { id, active_block }
    }

    pub fn get_active_block(&self) -> ActiveBlock {
        self.active_block
    }

    pub fn get_id(&self) -> RouteId {
        self.id.to_owned()
    }
}

impl Default for Route {
    fn default() -> Self {
        Self {
            id: RouteId::Welcome,
            active_block: ActiveBlock::SearchBar,
        }
    }
}
