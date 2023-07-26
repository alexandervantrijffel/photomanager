use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::{env, fs};

use globwalk::GlobWalkerBuilder;

use crate::fsops::{can_safely_overwrite, get_unique_filepath, have_equal_contents, safe_rename};
use crate::image::{Image, ImageToReview, PhotoReview, PhotosToReview};
use crate::reviewscore::{get_review_scores, get_review_scores_as_str, ReviewScore};

pub struct FileManager {
    root_dir: String,
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
    pub fn new_image(&self, relative_path: &str) -> Image {
        Image::new(relative_path, &self.root_dir)
    }
}

impl FileManager {
    pub fn review_photo(&self, review: &PhotoReview) -> Result<()> {
        println!("Reviewing photo: {:?}", review);
        if !PathBuf::from(&review.image.full_path).exists() {
            bail!("Photo not found: {}", review.image.full_path)
        }

        let destination_path = review.get_destination_path()?;

        println!(
            "Moving photo from {} to {}",
            review.image.full_path, &destination_path
        );

        self.move_file_prevent_overwrite_different_contents(
            &review.image.full_path,
            &destination_path,
        )
    }

    fn move_file_prevent_overwrite_different_contents(
        &self,
        source_file: &str,
        destination_file: &str,
    ) -> Result<()> {
        let mut final_destination_file = destination_file.to_string();
        if !can_safely_overwrite(source_file, destination_file)? {
            final_destination_file = get_unique_filepath(destination_file)?;
            println!(
                "Destination file already exists, but contents are different. Moving to {}",
                final_destination_file
            );
        }
        safe_rename(source_file, &final_destination_file)
    }

    pub fn undo(&self, review: &PhotoReview) -> Result<()> {
        println!("undoing review: {:?}", review);
        let destination_file = review.get_destination_path()?;
        if !PathBuf::from(&destination_file).exists() {
            bail!("Cannot undo, photo at [{}] not found", &destination_file)
        }
        println!(
            "Moving photo from {} to {}",
            destination_file, review.image.full_path
        );
        safe_rename(&destination_file, &review.image.full_path)
    }
}

// get_photos_to_review implementation
impl FileManager {
    pub fn get_photos_to_review(&self) -> Result<PhotosToReview> {
        let (folder_image_count, image_files) = self
            .find_image_files()
            .with_context(|| "failed to find image files")?;

        let folder_name = image_files
            .iter()
            .find(|p| !p.album_name.is_empty())
            .map(|p| p.album_name.clone())
            .unwrap_or("unknown".to_string());

        let photos = image_files
            .iter()
            .map(|f| ImageToReview {
                url: "/media/".to_string() + &f.relative_path,
                album: PathBuf::from(&f.full_path)
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap_or("unknown")
                    .to_string(),
            })
            .collect::<Vec<ImageToReview>>();

        Ok(PhotosToReview {
            base_url: env::var("PUBLIC_URL")
                .expect("'PUBLIC_URL' environment variable is required"),
            photos,
            folder_image_count,
            folder_name,
        })
    }

    fn find_image_files(&self) -> Result<(usize, Vec<Image>)> {
        let folder_with_review_images = self.find_next_folder_path_with_images_to_review()?;

        let mut image_files = fs::read_dir(folder_with_review_images)?
            .filter_map(Result::ok)
            .filter(|entry| {
                let path = entry.path();
                path.is_file()
                    && path.extension().map_or(false, |ext| {
                        ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "gif"
                    })
            })
            .map(|entry| entry.path().to_str().unwrap().to_string())
            .collect::<Vec<String>>();

        image_files.sort();

        let folder_image_count = image_files.len();

        let image_files = image_files
            .into_iter()
            .map(|path| Image::from_full_path(&path, &self.root_dir))
            // exclude all images that have already been reviewed
            .filter(|img| {
                !get_review_scores().iter().any(|score| {
                    if have_equal_contents(
                        &img.full_path,
                        &img.get_destination_path(score)
                            .expect("failed to get destination path"),
                    )
                    .unwrap_or(false)
                    {
                        // move images that are already reviewed to the already_reviewed bucket
                        safe_rename(
                            &img.full_path,
                            &img.get_destination_path(&ReviewScore::AlreadyReviewed)
                                .expect("failed to get destination path"),
                        )
                        // if the image was moved successfully, it shouldn't be reviewed
                        // anymore
                        .is_ok()
                    } else {
                        false
                    }
                })
            })
            .take(20)
            .collect();

        Ok((folder_image_count, image_files))
    }

    fn find_next_folder_path_with_images_to_review(&self) -> Result<String> {
        let mut excludes: Vec<String> = get_review_scores_as_str()
            .iter()
            .map(|f| format!("!**/{}/", f))
            .collect();

        excludes.insert(0, format!("**/{}", "*.{png,jpg,jpeg,gif}"));

        let folders_with_review_images =
            GlobWalkerBuilder::from_patterns(self.root_dir.as_str(), &excludes)
                .build()?
                .filter_map(Result::ok)
                .find_map(|img| {
                    img.path()
                        .parent()
                        .and_then(|p| p.to_str().map(|s| s.to_string()))
                });

        match folders_with_review_images {
            Some(folder) => Ok(folder),
            None => bail!(
                "No folders with images to review found under root folder {}",
                self.root_dir
            ),
        }
    }
}
