use async_graphql::EmptySubscription;
use async_graphql::{Context, Object, Schema};

use crate::file_management::{FileManager, ImageToReview, PhotoReview, ReviewScore};

pub(crate) type ServiceSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub(crate) fn new_schema() -> ServiceSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(FileManager::new())
    .finish()
}

#[derive(Copy, Default, Clone)]
pub(crate) struct QueryRoot {}

#[Object]
impl QueryRoot {
    /// {
    ///   photosToReview{
    ///    url
    ///    album
    ///   }
    /// }
    #[graphql(name = "photosToReview")]
    async fn photos_to_review(&self, _ctx: &Context<'_>) -> Vec<ImageToReview> {
        let fm = _ctx.data::<FileManager>().unwrap();
        match fm.get_photo_paths_to_review() {
            Ok(paths) => paths,
            Err(err) => {
                println!("Failed to retrieve photos_to_review: {}", err);
                vec![]
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct MutationRoot {}

#[Object]
impl MutationRoot {
    ///     mutation {
    ///       reviewPhoto(path:"/albumx/testphoto.jpg", score: WORST)
    ///     }
    #[graphql(name = "reviewPhoto")]
    async fn review_photo(&self, _ctx: &Context<'_>, path: String, score: ReviewScore) -> bool {
        let review = PhotoReview { path, score };
        _ctx.data::<FileManager>().unwrap().review_photo(&review);
        true
    }
}
