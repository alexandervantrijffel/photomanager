use crate::fsops::{
    can_safely_overwrite, chmod, get_unique_filepath, have_equal_contents,
    rename_with_create_dir_all,
};
use crate::image::{
    Image, ImageToReview, PhotoReview, PhotoReview as ReviewedPhoto, PhotosToReview,
};
use crate::reviewscore::{ReviewScore, get_review_scores, get_review_scores_as_str};
use anyhow::{Context, Result, anyhow, bail};
use globwalk::GlobWalkerBuilder;
use std::path::PathBuf;
use std::{env, fs};
use tracing::info;

pub struct FileManager {
    root_dir: String,
}

impl FileManager {
    pub const fn new(media_path: String) -> Self {
        Self {
            root_dir: media_path,
        }
    }

    pub fn new_image(&self, relative_path: &str) -> Result<Image> {
        Image::try_new(relative_path, &self.root_dir)
    }
}

impl FileManager {
    pub fn review_photo(&self, review: &PhotoReview) -> Result<ReviewedPhoto> {
        info!("Reviewing photo: {:?}", review);
        if !PathBuf::from(&review.image.full_path).exists() {
            bail!("Photo not found: {}", review.image.full_path)
        }

        let destination_path = review.get_destination_path();

        Self::move_file_prevent_overwrite_different_contents(
            &review.image.full_path,
            &destination_path,
        )
        .map(|()| ReviewedPhoto {
            image: Image::from_full_path(&destination_path, &self.root_dir),
            score: review.score,
        })
    }

    fn move_file_prevent_overwrite_different_contents(
        source_file: &str,
        destination_file: &str,
    ) -> Result<()> {
        let mut final_destination_file = destination_file.into();
        if !can_safely_overwrite(source_file, destination_file)? {
            final_destination_file = get_unique_filepath(destination_file)?;
            info!(
                "Destination file already exists, but contents are different. Moving to {}",
                final_destination_file
            );
        }
        rename_with_create_dir_all(source_file, &final_destination_file, 0o775)?;
        chmod(&final_destination_file, 0o775)
    }

    pub fn undo(review: &PhotoReview) -> Result<()> {
        info!("undoing review: {:?}", review);
        let destination_file = review.get_destination_path();
        if !PathBuf::from(&destination_file).exists() {
            bail!("Cannot undo, photo at [{}] not found", &destination_file)
        }
        rename_with_create_dir_all(&destination_file, &review.image.full_path, 0o775)
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
            .map_or_else(|| "unknown".into(), |p| p.album_name.clone());

        let photos = image_files
            .iter()
            .map(|f| {
                Ok(ImageToReview {
                    url: "/media/".to_string() + &f.relative_path,
                    album: PathBuf::from(&f.full_path)
                        .parent()
                        .context("Failed to get parent directory")?
                        .to_str()
                        .context("to_str failed")?
                        .into(),
                })
            })
            .collect::<Result<Vec<ImageToReview>>>()?;

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
                    && path.extension().map_or_else(
                        || false,
                        |ext| ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "gif",
                    )
            })
            .map(|entry| entry.path().to_str().unwrap().into())
            .collect::<Vec<String>>();

        image_files.sort();

        let folder_image_count = image_files.len();

        let image_files = image_files
            .into_iter()
            .map(|path| Image::from_full_path(&path, &self.root_dir))
            // exclude all images that have already been reviewed
            .filter(|img| {
                !get_review_scores().iter().any(|score| {
                    if have_equal_contents(&img.full_path, &img.get_destination_path(*score))
                        .unwrap_or(false)
                    {
                        // move images that are already reviewed to the already_reviewed bucket
                        return rename_with_create_dir_all(
                            &img.full_path,
                            &img.get_destination_path(ReviewScore::AlreadyReviewed),
                            0o775,
                        )
                        // if the image was moved successfully, it shouldn't be reviewed
                        // anymore
                        .is_ok();
                    }
                    false
                })
            })
            .take(20)
            .collect();

        Ok((folder_image_count, image_files))
    }

    fn find_next_folder_path_with_images_to_review(&self) -> Result<String> {
        let mut excludes: Vec<String> = vec![format!("**/{}", "*.{png,jpg,jpeg,gif}")];
        excludes.extend(
            get_review_scores_as_str()
                .iter()
                .map(|f| format!("!**/{f}/")),
        );

        GlobWalkerBuilder::from_patterns(self.root_dir.as_str(), &excludes)
            .build()?
            .filter_map(Result::ok)
            .find_map(|img| {
                img.path()
                    .parent()
                    .and_then(|p| p.to_str().map(std::convert::Into::into))
            })
            .ok_or_else(|| {
                anyhow!(
                    "No folders with images to review found under root folder {}",
                    self.root_dir
                )
            })
    }
}
