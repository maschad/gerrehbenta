use juniper::GraphQLObject;
use serde::Deserialize;

#[derive(Debug, Clone, GraphQLObject, Deserialize)]
#[graphql(description = "Information about a token")]
pub struct Token {
    /// The name of the token
    pub name: String,
    /// The symbol of the token
    pub symbol: String,
    /// The number of decimals of the token
    pub decimals: String,
}

#[derive(Debug, Clone, GraphQLObject, Deserialize)]
#[graphql(description = "Information about a transaction")]
pub struct Transaction {
    /// The timestamp of the transaction
    pub timestamp: String,
}

#[derive(Debug, Clone, GraphQLObject, Deserialize)]
pub struct PoolDayData {
    pub date: f64,
    pub token0Price: String,
    pub token1Price: String,
}

#[derive(Debug, Clone, GraphQLObject, Deserialize)]
pub struct PoolHourData {
    #[serde(rename = "periodStartUnix")]
    pub period_start_unix: f64,
    #[serde(rename = "token0Price")]
    pub token0_price: Option<String>,
    #[serde(rename = "token1Price")]
    pub token1_price: Option<String>,
    #[serde(rename = "volumeUSD")]
    pub volume_usd: String,
}

#[derive(Debug, Clone, GraphQLObject, Deserialize)]
#[graphql(description = "Information about a Uniswap pool")]
pub struct Pool {
    #[serde(rename = "token0Price")]
    pub token0_price: String,
    #[serde(rename = "token1Price")]
    pub token1_price: String,
    #[serde(rename = "poolDayData", default)]
    pub pool_day_datas: Vec<PoolDayData>,
    #[serde(rename = "poolHourData", default)]
    pub pool_hour_data: Vec<PoolHourData>,
}

#[derive(Debug, Clone, GraphQLObject, Deserialize)]
#[graphql(description = "Information about a Uniswap position")]
pub struct Position {
    #[serde(rename = "token0")]
    pub token0: Token,
    #[serde(rename = "token1")]
    pub token1: Token,
    pub pool: Pool,
    #[serde(rename = "withdrawnToken0")]
    pub withdrawn_token0: String,
    #[serde(rename = "withdrawnToken1")]
    pub withdrawn_token1: String,
    #[serde(rename = "depositedToken0")]
    pub deposited_token0: String,
    #[serde(rename = "depositedToken1")]
    pub deposited_token1: String,
    pub liquidity: String,
    #[serde(default)]
    pub transaction: Option<Transaction>,
}
