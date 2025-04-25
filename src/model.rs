use crate::file_management::FileManager;
use crate::google_photos_upload::upload_best_photos;
use crate::image::{PhotoReview, PhotosToReview};
use crate::reviewscore::ReviewScore;
use async_graphql::{Context, EmptySubscription, Object, Schema};
use async_graphql::{OutputType, SimpleObject};
use std::env;
use tracing::error;

pub type ServiceSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[must_use]
pub fn new_schema(media_path: Option<&str>) -> ServiceSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(FileManager::new(media_path.map_or_else(
        || env::var("MEDIA_ROOT").expect("'MEDIA_ROOT' environment variable is required"),
        std::string::ToString::to_string,
    )))
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
    ///        folderName
    ///        folderImageCount
    ///    }
    ///  }
    ///}
    #[graphql(name = "photosToReview")]
    async fn photos_to_review(&self, ctx: &Context<'_>) -> MutationResponse<PhotosToReview> {
        match ctx.data::<FileManager>().unwrap().get_photos_to_review() {
            Ok(paths) => MutationResponse {
                success: true,
                output: paths,
            },
            Err(err) => {
                error!("Failed to retrieve photos_to_review: {:#}", err);
                MutationResponse {
                    success: false,
                    output: PhotosToReview {
                        base_url: String::new(),
                        photos: vec![],
                        folder_name: String::new(),
                        folder_image_count: 0,
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

impl MutationResponse<String> {
    #[must_use]
    pub fn succeeded<T>(_: T) -> Self {
        Self {
            success: true,
            output: String::new(),
        }
    }
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
        ctx: &Context<'_>,
        path: String,
        score: ReviewScore,
    ) -> MutationResponse<String> {
        let file_manager = ctx.data::<FileManager>().unwrap();
        match file_manager
            .review_photo(&PhotoReview {
                image: file_manager.new_image(&path),
                score,
            })
            .map(upload_best_photos)
        {
            Ok(_) => MutationResponse::succeeded(String::new()),
            Err(err) => {
                error!("Failed to review photo '{}': {:#}", path, err);
                MutationResponse {
                    success: false,
                    output: err.to_string(),
                }
            }
        }
    }

    #[graphql(name = "undo")]
    async fn undo(
        &self,
        ctx: &Context<'_>,
        path: String,
        score: ReviewScore,
    ) -> MutationResponse<String> {
        let file_manager = ctx.data::<FileManager>().unwrap();
        match FileManager::undo(&PhotoReview {
            image: file_manager.new_image(&path),
            score,
        }) {
            Ok(()) => MutationResponse::succeeded(String::new()),
            Err(err) => {
                error!("Failed to undo review photo '{}': {:#}", path, err);
                MutationResponse {
                    success: false,
                    output: err.to_string(),
                }
            }
        }
    }
}
