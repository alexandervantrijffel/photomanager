use anyhow::{bail, Result};
use std::path::PathBuf;

use async_graphql::SimpleObject;

use crate::fsops::have_equal_contents;
use crate::reviewscore::{get_review_scores, get_review_scores_as_str, ReviewScore};

#[derive(Debug, Clone)]
pub struct PhotoReview {
    pub image: Image,
    pub score: ReviewScore,
}

impl PhotoReview {
    pub fn get_destination_path(&self) -> Result<String> {
        let source_folder = match PathBuf::from(&self.image.full_path).parent() {
            Some(parent) => parent.to_path_buf(),
            None => bail!(
                "Parent folder not found for path: {}",
                &self.image.full_path
            ),
        };

        let destination_folder = source_folder.join(self.score.as_str());

        let destination_file =
            destination_folder.join(PathBuf::from(&self.image.full_path).file_name().unwrap());
        Ok(destination_file.to_str().unwrap().to_string())
    }
}

#[derive(SimpleObject)]
pub struct PhotosToReview {
    pub base_url: String,
    pub photos: Vec<ImageToReview>,
    pub folder_image_count: usize,
    pub folder_name: String,
}
#[derive(SimpleObject)]
pub struct ImageToReview {
    pub url: String,
    pub album: String,
}

#[derive(Debug, Clone)]
pub struct Image {
    pub relative_path: String,
    pub root_dir: String,
    pub full_path: String,
}
impl Image {
    pub fn new(relative_path: &str, root_dir: &str) -> Self {
        Image {
            relative_path: relative_path.to_string(),
            root_dir: root_dir.to_string(),
            full_path: format!(
                "{}{}",
                root_dir,
                relative_path.strip_prefix("/media").unwrap()
            ),
        }
    }
}
