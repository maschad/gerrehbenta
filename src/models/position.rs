use juniper::GraphQLObject;

#[derive(Debug, Clone, GraphQLObject)]
#[graphql(description = "Information about a token")]
pub struct Token {
    name: String,
    symbol: String,
}

#[derive(Debug, Clone, GraphQLObject)]
#[graphql(description = "Information about a Uniswap position")]
pub struct Position {
    token_1: Token,
    token_2: Token,
    token_1_balance: String,
    token_2_balance: String,
    token_1_fees: String,
    token_2_fees: String,
}
