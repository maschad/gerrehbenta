use crate::models::position::Position;
use ethers::core::types::{Address, NameOrAddress};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Etherscan {
    api_key: Option<String>,
}

#[derive(Clone)]
pub struct Network<'a> {
    endpoint: &'a str,
    etherscan: &'a Option<Etherscan>,
}

pub enum NetworkEvent {
    GetENSAddressInfo {
        name_or_address: NameOrAddress,
        is_searching: bool,
    },
    GetAddressInfo {
        address: Address,
        positions: Option<Vec<Position>>,
    },
}
