use juniper::GraphQLObject;
use serde::Deserialize;

#[derive(Debug, Clone, GraphQLObject, Deserialize)]
#[graphql(description = "Information about a token")]
pub struct Token {
    /// The ID of the token
    pub id: String,
    /// The name of the token
    pub name: String,
    /// The symbol of the token
    pub symbol: String,
    /// The number of decimals of the token
    pub decimals: i32,
}

#[derive(Debug, Clone, GraphQLObject, Deserialize)]
#[graphql(description = "Information about a transaction")]
pub struct Transaction {
    /// The ID of the transaction
    pub id: String,
    /// The timestamp of the transaction
    pub timestamp: i32,
}

#[derive(Debug, Clone, GraphQLObject, Deserialize)]
#[graphql(description = "Information about a Uniswap pool")]
pub struct Pool {
    /// The ID of the pool
    pub id: String,
    pub token_0_price: i32,
    pub token_1_price: i32,
    pub volume_token_0: i32,
    pub volume_token_1: i32,
}

#[derive(Debug, Clone, GraphQLObject, Deserialize)]
#[graphql(description = "Information about a Uniswap position")]
pub struct Position {
    /// The ID of the position
    pub id: String,
    pub token_0: Token,
    pub token_1: Token,
    pub pool: Pool,
    pub withdrawn_token_0: i32,
    pub withdrawn_token_1: i32,
    pub deposited_token_0: i32,
    pub deposited_token_1: i32,
    pub liquidity: i32,
    pub transaction: Transaction,
}
