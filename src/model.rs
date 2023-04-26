use async_graphql::EmptySubscription;
use async_graphql::{Context, Enum, Object, Schema};

use crate::file_management::get_photo_paths_to_review;

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

    async fn photos_to_review(&self, _ctx: &Context<'_>) -> Vec<String> {
        get_photo_paths_to_review().unwrap()
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
        let _review = PhotoReview { path, score };
        true
    }
}
