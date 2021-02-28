
#[derive(GraphQLObject)]
#[grapql(description = "Contains data on a specific token. This token specific data is aggregated across all pairs, and is updated whenever there is a transaction involving that token.")]
struct Token {
    pub id: i64,
    pub symbol: String,
    pub totalSupply: i128,
    pub tradeVolumeUSD: i128,
    pub totalLiquidity: i128
}