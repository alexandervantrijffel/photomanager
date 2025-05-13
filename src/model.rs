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
    async fn photos_to_review(&self, ctx: &Context<'_>) -> Response<PhotosToReview> {
        match ctx.data::<FileManager>().unwrap().get_photos_to_review() {
            Ok(paths) => Response::succeeded(paths),
            Err(err) => {
                error!("Failed to retrieve photos_to_review: {:#}", err);
                Response {
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
    ) -> Response<String> {
        let file_manager = ctx.data::<FileManager>().unwrap();
        match file_manager
            .new_image(&path)
            .and_then(|image| file_manager.review_photo(&PhotoReview { image, score }))
            .and_then(upload_best_photos)
        {
            Ok(()) => Response::succeeded(String::new()),
            Err(err) => {
                error!("Failed to review photo '{}': {:#}", path, err);
                Response {
                    success: false,
                    output: err.to_string(),
                }
            }
        }
    }

    #[graphql(name = "undo")]
    async fn undo(&self, ctx: &Context<'_>, path: String, score: ReviewScore) -> Response<String> {
        let file_manager = ctx.data::<FileManager>().unwrap();
        match file_manager
            .new_image(&path)
            .and_then(|image| FileManager::undo(&PhotoReview { image, score }))
        {
            Ok(()) => Response::succeeded(String::new()),
            Err(err) => {
                error!("Failed to undo review photo '{}': {:#}", path, err);
                Response {
                    success: false,
                    output: err.to_string(),
                }
            }
        }
    }
}

#[derive(SimpleObject)]
#[graphql(concrete(name = "MutationReponseString", params(String)))]
#[graphql(concrete(name = "MutationResponsePhotosToReview", params(PhotosToReview)))]
pub struct Response<T: OutputType> {
    success: bool,
    output: T,
}

impl<T> Response<T>
where
    T: OutputType,
{
    #[must_use]
    pub const fn succeeded(t: T) -> Self {
        Self {
            success: true,
            output: t,
        }
    }
}
