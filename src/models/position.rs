use juniper::GraphQLObject;

/** Query for active liquidity positions
 * {
  positions(where: {owner: "0x6ed31d002338349e486dad57939e1e4a4a7a0007", liquidity_gt: 0}) {
    token1 {
      symbol
      decimals
      name
    }
    transaction {
      timestamp
    }
    token0 {
      symbol
      decimals
      name
    }
    pool {
      token0Price
      token1Price
      volumeToken0
      volumeToken1
      feeGrowthGlobal0X128
      feeGrowthGlobal1X128
    }
    feeGrowthInside0LastX128
    feeGrowthInside1LastX128
    tickLower {feeGrowthOutside0X128 feeGrowthOutside1X128} #feeGrowthOutside0X128_lower
    tickUpper {feeGrowthOutside0X128 feeGrowthOutside1X128} #feeGrowthOutside0X128_upper
    withdrawnToken0
    withdrawnToken1
    depositedToken0
    depositedToken1
    liquidity
  }
}
 */

#[derive(Debug, Clone, GraphQLObject)]
#[graphql(description = "Information about a token")]
pub struct Token {
    /// The name of the token
    name: String,
    /// The symbol of the token
    symbol: String,
    /// The number of decimals of the token
    decimals: i32,
}

#[derive(Debug, Clone, GraphQLObject)]
#[graphql(description = "Information about a transaction")]
pub struct Transaction {
    /// The timestamp of the transaction
    timestamp: i32,
}

#[derive(Debug, Clone, GraphQLObject)]
#[graphql(description = "Information about a Uniswap pool")]
pub struct Pool {
    token_0_price: i32,
    token_1_price: i32,
    volume_token_0: i32,
    volume_token_1: i32,
    fee_growth_global_0_x128: String,
    fee_growth_global_1_x128: String,
}

#[derive(Debug, Clone, GraphQLObject)]
#[graphql(description = "Information about a Uniswap tick")]
pub struct Tick {
    fee_growth_outside_0_x128: String,
    fee_growth_outside_1_x128: String,
}
#[derive(Debug, Clone, GraphQLObject)]
#[graphql(description = "Information about a Uniswap position")]
pub struct Position {
    token_0: Token,
    token_1: Token,
    pool: Pool,
    fee_growth_inside_0_last_x128: String,
    fee_growth_inside_1_last_x128: String,
    tick_lower: Tick,
    tick_upper: Tick,
    withdrawn_token_0: i32,
    withdrawn_token_1: i32,
    deposited_token_0: i32,
    deposited_token_1: i32,
    liquidity: i32,
}
