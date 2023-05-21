use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::{env, fs};

use async_graphql::{Enum, SimpleObject};
use globwalk::GlobWalkerBuilder;

pub struct FileManager {
    root_dir: String,
}

#[derive(SimpleObject)]
pub struct PhotosToReview {
    pub base_url: String,
    pub photos: Vec<ImageToReview>,
}

#[derive(SimpleObject)]
pub struct ImageToReview {
    url: String,
    album: String,
}

#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq)]
pub enum ReviewScore {
    Best,
    Nah,
    Worst,
}

#[derive(Debug, Clone)]
pub struct PhotoReview {
    pub path: String,
    pub score: ReviewScore,
}

impl FileManager {
    pub fn new() -> Self {
        FileManager {
            root_dir: shellexpand::env(
                &env::var("MEDIA_ROOT").expect("'MEDIA_ROOT' environment variable is required"),
            )
            .unwrap()
            .to_string(),
        }
    }

    fn full_path(&self, relative_path: &str) -> String {
        format!(
            "{}{}",
            self.root_dir,
            relative_path
                .strip_prefix("/media")
                .unwrap_or(relative_path)
        )
    }

    pub fn review_photo(&self, review: &PhotoReview) -> Result<()> {
        println!("Reviewing photo: {:?}", review);
        let path = self.full_path(&review.path);
        if !PathBuf::from(&path).exists() {
            bail!("Photo not found: {}", path);
        }
        let new_folder = PathBuf::from(&path)
            .parent()
            .unwrap()
            .join(match review.score {
                ReviewScore::Best => "best",
                ReviewScore::Nah => "nah",
                ReviewScore::Worst => "worst",
            });
        fs::create_dir_all(&new_folder).with_context(|| {
            format!(
                "Failed to create media target folder '{}'",
                new_folder.display()
            )
        })?;
        let new_path = new_folder.join(PathBuf::from(&path).file_name().unwrap());
        println!("Moving photo from {} to {}", path, new_path.display());
        fs::rename(&path, &new_path).with_context(|| {
            format!(
                "Failed to move photo from {} to {}",
                path,
                new_path.display()
            )
        })?;

        Ok(())
    }
}

impl FileManager {
    pub fn get_photo_paths_to_review(&self) -> Result<PhotosToReview> {
        let image_files = self
            .find_image_files()
            .with_context(|| "failed to find image files")?;
        let photos = image_files
            .iter()
            .map(|f| ImageToReview {
                url: f.replace(&self.root_dir, "/media"),
                album: {
                    if let Some(dir) = PathBuf::from(f).parent().unwrap().to_str() {
                        dir
                    } else {
                        "unknown"
                    }
                    .to_string()
                },
            })
            .collect::<Vec<ImageToReview>>();

        Ok(PhotosToReview {
            base_url: env::var("PUBLIC_URL")
                .expect("'PUBLIC_URL' environment variable is required"),
            photos,
        })
    }

    fn find_image_files(&self) -> Result<Vec<String>> {
        let image_files_pattern = "*.{png,jpg,jpeg,gif}";
        let folder_with_review_images =
            self.find_next_folder_path_with_images_to_review(image_files_pattern)?;

        let image_files = GlobWalkerBuilder::from_patterns(
            folder_with_review_images.as_str(),
            &[image_files_pattern, "!best/*", "!nah/*"],
        )
        .max_depth(1)
        .follow_links(true)
        .build()?
        .filter_map(Result::ok)
        .take(10)
        .map(|img| img.path().to_str().unwrap().to_string())
        .collect::<Vec<String>>();

        Ok(image_files)
    }

    fn find_next_folder_path_with_images_to_review(
        &self,
        images_files_pattern: &str,
    ) -> Result<String> {
        let folders_with_review_images = GlobWalkerBuilder::from_patterns(
            self.root_dir.as_str(),
            &[
                format!("**/{}", images_files_pattern),
                "!**/best/".to_string(),
                "!**/nah/".to_string(),
                "!**/worst/".to_string(),
            ],
        )
        .build()?
        .filter_map(Result::ok)
        .take(1)
        .map(|img| img.path().parent().unwrap().to_str().unwrap().to_string())
        .collect::<Vec<String>>();

        if folders_with_review_images.is_empty() {
            bail!(format!(
                "No folders with images to review found under root folder {}",
                self.root_dir
            ));
        }
        Ok(folders_with_review_images[0].clone())
    }
}
