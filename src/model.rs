use async_graphql::EmptySubscription;
use async_graphql::{Context, Object, Schema};

use crate::file_management::{FileManager, PhotoReview, PhotosToReview, ReviewScore};

pub type ServiceSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn new_schema() -> ServiceSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(FileManager::new())
    .finish()
}

#[derive(Copy, Default, Clone)]
pub struct QueryRoot {}

#[Object]
impl QueryRoot {
    /// {
    ///   photosToReview{
    ///    url
    ///    album
    ///   }
    /// }
    #[graphql(name = "photosToReview")]
    async fn photos_to_review(&self, _ctx: &Context<'_>) -> PhotosToReview {
        let fm = _ctx.data::<FileManager>().unwrap();
        match fm.get_photo_paths_to_review() {
            Ok(paths) => paths,
            Err(err) => {
                println!("Failed to retrieve photos_to_review: {}", err);
                PhotosToReview {
                    base_url: "".to_string(),
                    photos: vec![],
                }
            }
        }
    }
}

#[derive(Default)]
pub struct MutationRoot {}

#[Object]
impl MutationRoot {
    ///     mutation {
    ///       reviewPhoto(path:"/albumx/testphoto.jpg", score: WORST)
    ///     }
    #[graphql(name = "reviewPhoto")]
    async fn review_photo(&self, _ctx: &Context<'_>, path: String, score: ReviewScore) -> bool {
        let review = PhotoReview {
            path: path.clone(),
            score,
        };
        match _ctx.data::<FileManager>().unwrap().review_photo(&review) {
            Ok(_) => true,
            Err(err) => {
                println!("Failed to review photo '{}': {}", path, err);
                false
            }
        }
    }
}
