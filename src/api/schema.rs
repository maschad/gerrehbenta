use juniper::FieldResult;
use juniper::{EmptySubscription, RootNode};
use juniper::{GraphQLEnum, GraphQLInputObject, GraphQLObject};


#[derive(GraphQLObject)]
#[graphql(description = "An ETH Token")]
struct Token {
	id: String,
	name: String,
	priceUSD: f32,
	volumeUSD: f32,
	high: f32,
	low: f32,
}

pub struct QueryRoot;

#[juniper::graphql_object(args: TokenStream, input: TokenStream)]
impl QueryRoot {
	fn token(first: i16, orderBy: String) -> FieldResult<Token> {

	}
}

#[juniper::graphql_object]
impl MutationRoot {

}


pub type Schema = RootNode<'static, QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema() -> Schema {
	Schema::new(QueryRoot {}, MutationRoot {}, EmptySubscription::new())
}