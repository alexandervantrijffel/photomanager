use anyhow::{anyhow, bail, Context, Result};
use std::path::{Path, PathBuf};
use std::{env, fs};

use globwalk::GlobWalkerBuilder;

use crate::image::{Image, ImageToReview, PhotoReview, PhotosToReview};

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

        let paths = review
            .image
            .source_and_destination_paths(review)?
            .ensure_paths()?;

        let destination_file = paths.destination_file.display().to_string();
        println!(
            "Moving photo from {} to {}",
            review.image.full_path, destination_file
        );

        self.move_file_prevent_overwrite_different_contents(
            &review.image.full_path,
            &destination_file,
        )
    }

    fn move_file_prevent_overwrite_different_contents(
        &self,
        source_file: &str,
        destination_file: &str,
    ) -> Result<()> {
        let mut final_destination_file = destination_file.to_string();
        if Path::new(destination_file).exists() {
            let source_file_contents = fs::read(source_file).unwrap();
            let destination_file_contents = fs::read(destination_file).unwrap();
            if source_file_contents != destination_file_contents {
                final_destination_file = self.get_unique_filepath(destination_file)?; // format!("{}.new", &destination_file);
                println!(
                    "Destination file already exists, but contents are different. Moving to {}",
                    final_destination_file
                );
            }
        }
        fs::rename(source_file, &final_destination_file).with_context(|| {
            format!(
                "Failed to move photo from {} to {}",
                source_file, final_destination_file
            )
        })
    }

    fn get_unique_filepath(&self, file_path: &str) -> Result<String> {
        let path = Path::new(file_path);
        let dir = path
            .parent()
            .ok_or_else(|| anyhow!("Failed to get parent dir"))?;
        let title = path
            .file_stem()
            .and_then(|p| p.to_str())
            .ok_or_else(|| anyhow!("no file title"))?;
        let ext = path.extension().and_then(|p| p.to_str()).ok_or_else(|| {
            anyhow!(
                "Failed to get file extension for: {}",
                path.to_str().unwrap()
            )
        })?;

        let mut last_path_buf: PathBuf = PathBuf::new();

        let found = (1..=20).find(|i| {
            last_path_buf = dir.join(format!("{}-{}.{}", title, i, ext));
            !Path::new(&last_path_buf).exists()
        });

        match found {
            Some(_) => Ok(last_path_buf.to_str().unwrap().to_string()),
            None => bail!(
                "Failed to find unique file path for: {}, last path: {}",
                file_path,
                last_path_buf.to_str().unwrap()
            ),
        }
    }

    pub fn undo(&self, review: &PhotoReview) -> Result<()> {
        println!("undoing review: {:?}", review);
        let paths = review.image.source_and_destination_paths(review)?;
        if !PathBuf::from(&review.image.full_path).exists() {
            bail!("Photo not found: {}", &review.image.full_path)
        }
        println!(
            "Moving photo from {} to {}",
            review.image.full_path,
            paths.destination_file.display()
        );
        fs::rename(&review.image.full_path, &paths.destination_file).with_context(|| {
            format!(
                "Failed to move photo from {} to {}",
                review.image.full_path,
                paths.destination_file.display()
            )
        })?;
        Ok(())
    }
}

// get_photos_to_review implementation
impl FileManager {
    pub fn get_photos_to_review(&self) -> Result<PhotosToReview> {
        let (folder_image_count, folder_path, image_files) = self
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

        let folder_name = Path::new(&folder_path).file_name().unwrap().to_str();

        Ok(PhotosToReview {
            base_url: env::var("PUBLIC_URL")
                .expect("'PUBLIC_URL' environment variable is required"),
            photos,
            folder_image_count,
            folder_name: folder_name.unwrap().to_string(),
        })
    }

    fn find_image_files(&self) -> Result<(usize, String, Vec<String>)> {
        let folder_with_review_images = self.find_next_folder_path_with_images_to_review()?;

        let mut image_files = fs::read_dir(&folder_with_review_images)?
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

        image_files.truncate(20);

        Ok((folder_image_count, folder_with_review_images, image_files))
    }

    fn get_exclude_folder_names(&self) -> Vec<String> {
        vec![
            "best".to_string(),
            "nah".to_string(),
            "worst".to_string(),
            ".recycle".to_string(),
        ]
    }
    fn find_next_folder_path_with_images_to_review(&self) -> Result<String> {
        let mut excludes: Vec<String> = self
            .get_exclude_folder_names()
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
