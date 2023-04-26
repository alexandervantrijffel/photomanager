use async_graphql::{Context, Enum, InputType, Object, Schema};
use async_graphql::{EmptyMutation, EmptySubscription};

pub(crate) type ServiceSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
pub(crate) struct QueryRoot;

#[Object]
impl QueryRoot {
    /*
        {
          hello
        }
    */
    async fn hello(&self, _ctx: &Context<'_>) -> &'static str {
        "Hello world"
    }
}
pub(crate) struct MutationRoot;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ReviewScore {
    Best,
    Soso,
    Worst,
}

#[derive(Clone)]
pub struct PhotoReview {
    pub path: String,
    pub score: ReviewScore,
}

#[Object]
impl MutationRoot {
    async fn review_photo(&self, _ctx: &Context<'_>, path: String, score: ReviewScore) -> bool {
        let review = PhotoReview { path, score };
        true
    }
}
