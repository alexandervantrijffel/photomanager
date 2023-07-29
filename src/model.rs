use std::env;

use async_graphql::{Context, EmptySubscription, Object, Schema};

use async_graphql::{OutputType, SimpleObject};

use crate::file_management::FileManager;
use crate::google_photos_upload::upload_best_photos;
use crate::image::{PhotoReview, PhotosToReview};
use crate::reviewscore::ReviewScore;

pub type ServiceSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn new_schema(media_path: Option<&str>) -> ServiceSchema {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(FileManager::new(
        &media_path.map(|p| p.to_string()).unwrap_or_else(|| {
            shellexpand::env(
                &env::var("MEDIA_ROOT").expect("'MEDIA_ROOT' environment variable is required"),
            )
            .unwrap()
            .to_string()
        }),
    ))
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
    async fn photos_to_review(&self, _ctx: &Context<'_>) -> MutationResponse<PhotosToReview> {
        _ctx.data::<FileManager>()
            .unwrap()
            .get_photos_to_review()
            .map(|paths| MutationResponse {
                success: true,
                output: paths,
            })
            .unwrap_or_else(|err| {
                println!("Failed to retrieve photos_to_review: {:#}", err);
                MutationResponse {
                    success: false,
                    output: PhotosToReview {
                        base_url: "".to_string(),
                        photos: vec![],
                        folder_name: "".to_string(),
                        folder_image_count: 0,
                    },
                }
            })
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
    pub fn succeeded<T>(_: T) -> Self {
        Self {
            success: true,
            output: "".to_string(),
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
        file_manager
            .review_photo(&PhotoReview {
                image: file_manager.new_image(&path),
                score,
            })
            .map(upload_best_photos)
            .map(MutationResponse::succeeded)
            .unwrap_or_else(|err| {
                println!("Failed to review photo '{}': {:#}", path, err);
                MutationResponse {
                    success: false,
                    output: err.to_string(),
                }
            })
    }

    #[graphql(name = "undo")]
    async fn undo(
        &self,
        ctx: &Context<'_>,
        path: String,
        score: ReviewScore,
    ) -> MutationResponse<String> {
        let file_manager = ctx.data::<FileManager>().unwrap();
        file_manager
            .undo(&PhotoReview {
                image: file_manager.new_image(&path),
                score,
            })
            .map(MutationResponse::succeeded)
            .unwrap_or_else(|err| {
                println!("Failed to undo review photo '{}': {:#}", path, err);
                MutationResponse {
                    success: false,
                    output: err.to_string(),
                }
            })
    }
}
