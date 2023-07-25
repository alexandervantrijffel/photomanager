use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

use async_graphql::{Enum, SimpleObject};

#[derive(Debug, Clone)]
pub struct Image {
    pub relative_path: String,
    pub root_dir: String,
    pub full_path: String,
}
#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq)]
pub enum ReviewScore {
    Best,
    Nah,
    Worst,
}

#[derive(Debug)]
pub struct DiskPaths {
    pub destination_folder: PathBuf,
    pub destination_file: PathBuf,
}

impl DiskPaths {
    pub fn ensure_paths(self) -> Result<Self> {
        // std::fs::create_dir_all(&self.destination_folder).unwrap();
        fs::create_dir_all(&self.destination_folder).with_context(|| {
            format!(
                "Failed to create media target folder '{}'",
                &self.destination_folder.display()
            )
        })?;
        Ok(self)
    }
}

#[derive(Debug, Clone)]
pub struct PhotoReview {
    pub image: Image,
    pub score: ReviewScore,
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
    pub fn source_and_destination_paths(&self, review: &PhotoReview) -> Result<DiskPaths> {
        let source_folder = match PathBuf::from(&review.image.full_path).parent() {
            Some(parent) => parent.to_path_buf(),
            None => bail!(
                "Parent folder not found for path: {}",
                review.image.full_path
            ),
        };

        let destination_folder = source_folder.join(match review.score {
            ReviewScore::Best => "best",
            ReviewScore::Nah => "nah",
            ReviewScore::Worst => "worst",
        });

        let destination_file =
            destination_folder.join(PathBuf::from(&review.image.full_path).file_name().unwrap());
        Ok(DiskPaths {
            destination_folder,
            destination_file,
        })
    }
}
