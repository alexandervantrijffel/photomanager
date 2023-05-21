use async_graphql::EmptySubscription;
use async_graphql::{Context, Object, Schema};

use crate::file_management::{FileManager, PhotoReview, PhotosToReview, ReviewScore};
use async_graphql::{OutputType, SimpleObject};

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
    ///{
    ///  photosToReview{
    ///     output {
    ///        baseUrl
    ///        photos{
    ///          album
    ///          url
    ///        }
    ///    }
    ///  }
    ///}
    #[graphql(name = "photosToReview")]
    async fn photos_to_review(&self, _ctx: &Context<'_>) -> MutationResponse<PhotosToReview> {
        let fm = _ctx.data::<FileManager>().unwrap();
        match fm.get_photo_paths_to_review() {
            Ok(paths) => MutationResponse {
                success: true,
                output: paths,
            },
            Err(err) => {
                println!("Failed to retrieve photos_to_review: {}", err);
                MutationResponse {
                    success: false,
                    output: PhotosToReview {
                        base_url: "".to_string(),
                        photos: vec![],
                    },
                }
            }
        }
    }
}

#[derive(Default)]
pub struct MutationRoot {}

#[derive(SimpleObject)]
#[graphql(concrete(name = "MutationReponseString", params(String)))]
#[graphql(concrete(name = "MutationResponsePhotosToReview", params(PhotosToReview)))]
pub struct MutationResponse<T: OutputType> {
    success: bool,
    output: T,
}

#[Object]
impl MutationRoot {
    ///     mutation {
    ///       reviewPhoto(path:"/albumx/testphoto.jpg", score: WORST) {
    ///          success
    ///          output
    ///       }
    ///     }
    #[graphql(name = "reviewPhoto")]
    async fn review_photo(
        &self,
        _ctx: &Context<'_>,
        path: String,
        score: ReviewScore,
    ) -> MutationResponse<String> {
        let review = PhotoReview {
            path: path.clone(),
            score,
        };
        match _ctx.data::<FileManager>().unwrap().review_photo(&review) {
            Ok(_) => MutationResponse {
                success: true,
                output: "".to_string(),
            },
            Err(err) => {
                println!("Failed to review photo '{}': {}", path, err);
                MutationResponse {
                    success: false,
                    output: err.to_string(),
                }
            }
        }
    }
}
