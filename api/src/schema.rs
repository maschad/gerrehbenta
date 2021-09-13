use juniper::FieldResult;
use juniper::{EmptyMutation, EmptySubscription, RootNode};
use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};


#[derive(GraphQLObject)]
#[graphql(description = "An ETH Token")]
struct Token {
	id: String,
	name: String,

}

#[derive(GraphQLObject)]
#[graphql(description = "Daily Token Data")]
struct TokenDayDatas {
	priceUSD: f32,
	volumeUSD: f32,
	high: f32,
	low: f32,
}

pub struct QueryRoot;

#[juniper::graphql_object(args: TokenStream, input: TokenStream)]
impl QueryRoot {
	fn getTokens(first: i16, orderBy: String) -> FieldResult<Token> {
		Ok(TokenDayDatas {
			Token {
			id,
			name,

		}
			priceUSD,
			volumeUSD,
			high,
			low
		})
	}
}


pub type Schema = RootNode<'static, QueryRoot, EmptyMutation, EmptySubscription>;

pub fn create_schema() -> Schema {
	Schema::new(QueryRoot {}, EmptyMutation::new(), EmptySubscription::new())
}