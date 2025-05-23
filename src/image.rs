use crate::reviewscore::ReviewScore;
use anyhow::{Context, Result};
use async_graphql::SimpleObject;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PhotoReview {
    pub image: Image,
    pub score: ReviewScore,
}

impl PhotoReview {
    pub fn get_destination_path(&self) -> String {
        self.image.get_destination_path(self.score)
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
    pub album_name: String,
}
impl Image {
    pub fn try_new(relative_path: &str, root_dir: &str) -> Result<Self> {
        Ok(Self {
            relative_path: relative_path.into(),
            root_dir: root_dir.into(),
            full_path: format!(
                "{}{}",
                root_dir,
                relative_path
                    .strip_prefix("/media")
                    .context("failed to strip prefix")?
            ),
            album_name: PathBuf::from(relative_path)
                .parent()
                .context("Failed to get parent directory")?
                .file_name()
                .context("Failed to get file name")?
                .to_str()
                .context("Failed to convert to str")?
                .into(),
        })
    }
    pub fn from_full_path(full_path: &str, root_dir: &str) -> Self {
        Self {
            relative_path: Path::new(full_path)
                .strip_prefix(Path::new(root_dir))
                .unwrap()
                .to_str()
                .unwrap()
                .into(),
            root_dir: root_dir.into(),
            full_path: full_path.into(),
            album_name: PathBuf::from(full_path)
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .into(),
        }
    }
    // Returns <root_dir>/score/album/filename
    pub fn get_destination_path(&self, score: ReviewScore) -> String {
        PathBuf::from(&self.root_dir)
            .join(score.as_str())
            .join(&self.album_name)
            .join(PathBuf::from(&self.full_path).file_name().unwrap())
            .to_str()
            .unwrap()
            .into()
    }
}
