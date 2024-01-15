use crate::network::ethers::types::AddressInfo;
use ethers::types::U256;

#[derive(Debug, Clone)]
pub struct Token {
    name: String,
    symbol: String,
    address: AddressInfo,
}

#[derive(Debug, Clone)]
pub struct Position {
    token_1: Token,
    token_2: Token,
    token_1_balance: U256,
    token_2_balance: U256,
    token_1_fees: U256,
    token_2_fees: U256,
}
