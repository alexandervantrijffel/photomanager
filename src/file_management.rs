use std::error::Error;

use globwalk::GlobWalkerBuilder;

pub(crate) struct FileManager {
    pub root_dir: String,
}

impl FileManager {
    pub fn new() -> Self {
        FileManager {
            root_dir: shellexpand::env("$HOME/pictures/photomanager-test")
                .unwrap()
                .to_string(),
        }
    }
    pub fn get_photo_paths_to_review(&self) -> Result<Vec<String>, Box<dyn Error>> {
        self.find_image_files()
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
