use std::error::Error;
use std::path::PathBuf;

use async_graphql::{Enum, SimpleObject};
use globwalk::GlobWalkerBuilder;

pub(crate) struct FileManager {
    root_dir: String,
}

#[derive(SimpleObject)]
pub(crate) struct ImageToReview {
    url: String,
    album: String,
}

#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq)]
pub enum ReviewScore {
    Best,
    Soso,
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
            root_dir: shellexpand::env("$HOME/pictures/photomanager-test")
                .unwrap()
                .to_string(),
        }
        // TODO create root_dir, best, soso dirs, archived folder
    }

    fn full_path(&self, relative_path: &str) -> String {
        format!("{}{}", self.root_dir, relative_path)
    }

    pub fn review_photo(&self, review: &PhotoReview) {
        println!("Reviewing photo: {:?}", review);
        let path = self.full_path(review.path.as_str());
        let dir = PathBuf::from(&path).parent().unwrap();
        let new_path = match review.score {
            ReviewScore::Best => dir.join("best"),
            ReviewScore::Soso => dir.join("soso"),
            // TODO archive worst photos
            ReviewScore::Worst => dir.join("worst"),
        }
        .join(PathBuf::from(path).file_name().unwrap())
        .to_str()
        .unwrap();
        println!("Moving photo from {} to {}", path, new_path);
    }
}

impl FileManager {
    pub fn get_photo_paths_to_review(&self) -> Result<Vec<ImageToReview>, Box<dyn Error>> {
        let image_files = self.find_image_files()?;
        Ok(image_files
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
            .collect::<Vec<ImageToReview>>())
    }

    fn find_image_files(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let image_files_pattern = "*.{png,jpg,jpeg,gif}";
        let folder_with_review_images =
            self.find_next_folder_path_with_images_to_review(image_files_pattern)?;

        let image_files = GlobWalkerBuilder::from_patterns(
            folder_with_review_images.as_str(),
            &[image_files_pattern, "!best/*", "!soso/*"],
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
    ) -> Result<String, Box<dyn Error>> {
        let folders_with_review_images = GlobWalkerBuilder::from_patterns(
            self.root_dir.as_str(),
            &[
                format!("**/{}", images_files_pattern),
                "!**/best/".to_string(),
                "!**/soso/".to_string(),
            ],
        )
        .build()?
        .filter_map(Result::ok)
        .take(1)
        .map(|img| img.path().parent().unwrap().to_str().unwrap().to_string())
        .collect::<Vec<String>>();

        if folders_with_review_images.is_empty() {
            return Err(format!(
                "No folders with images to review found under root folder {}",
                self.root_dir
            )
            .into());
        }
        Ok(folders_with_review_images[0].clone())
    }
}
