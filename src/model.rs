use async_graphql::EmptySubscription;
use async_graphql::{Context, Enum, Object, Schema};

use crate::file_management::{FileManager, ImageToReview};

pub(crate) type ServiceSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
pub(crate) struct QueryRoot {
    file_manager: FileManager,
}

pub(crate) fn new_query_root() -> QueryRoot {
    QueryRoot {
        file_manager: FileManager::new(),
    }
}

#[Object]
impl QueryRoot {
    /*
    {
      photosToReview{
       url
       album
      }
    }
    */
    async fn photos_to_review(&self, _ctx: &Context<'_>) -> Vec<ImageToReview> {
        match self.file_manager.get_photo_paths_to_review() {
            Ok(paths) => paths,
            Err(err) => {
                println!("Failed to retrieve photos_to_review: {}", err);
                vec![]
            }
        }
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
