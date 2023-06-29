use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
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
    pub folder_image_count: usize,
    pub folder_name: String,
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
}

impl FileManager {
    pub fn review_photo(&self, review: &PhotoReview) -> Result<()> {
        println!("Reviewing photo: {:?}", review);
        let full_path = self.full_path(&review.path);
        if !PathBuf::from(&full_path).exists() {
            bail!("Photo not found: {full_path}");
        }

        let paths = self.source_and_destination_paths(review)?;
        fs::create_dir_all(&paths.destination_folder).with_context(|| {
            format!(
                "Failed to create media target folder '{}'",
                &paths.destination_folder.display()
            )
        })?;

        let destination_file = paths.destination_file.display().to_string();
        println!("Moving photo from {} to {}", full_path, destination_file);
        fs::rename(&full_path, &destination_file).with_context(|| {
            format!(
                "Failed to move photo from {} to {}",
                full_path, destination_file
            )
        })?;

        Ok(())
    }

    pub fn undo(&self, review: &PhotoReview) -> Result<()> {
        println!("undoing review: {:?}", review);
        let paths = self.source_and_destination_paths(review)?;
        if !PathBuf::from(&paths.full_path).exists() {
            bail!("Photo not found: {}", paths.full_path);
        }
        println!(
            "Moving photo from {} to {}",
            paths.full_path,
            paths.destination_file.display()
        );
        fs::rename(&paths.full_path, &paths.destination_file).with_context(|| {
            format!(
                "Failed to move photo from {} to {}",
                paths.full_path,
                paths.destination_file.display()
            )
        })?;
        Ok(())
    }
}

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

        if image_files.len() > 10 {
            image_files.resize(10, String::new());
        }

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
        let mut excludes = self
            .get_exclude_folder_names()
            .iter()
            .map(|f| format!("!**/{}/", f))
            .collect::<Vec<String>>();

        excludes.insert(0, format!("**/{}", "*.{png,jpg,jpeg,gif}"));

        let folders_with_review_images =
            GlobWalkerBuilder::from_patterns(self.root_dir.as_str(), &excludes)
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

impl FileManager {
    fn full_path(&self, relative_path: &str) -> String {
        format!(
            "{}{}",
            self.root_dir,
            relative_path
                .strip_prefix("/media")
                .unwrap_or(relative_path)
        )
    }

    fn source_and_destination_paths(&self, review: &PhotoReview) -> Result<DiskPaths> {
        let full_path = self.full_path(&review.path);

        let source_folder = match PathBuf::from(&full_path).parent() {
            Some(parent) => parent.to_path_buf(),
            None => bail!("Parent folder not found for path: {}", full_path),
        };

        let destination_folder = source_folder.join(match review.score {
            ReviewScore::Best => "best",
            ReviewScore::Nah => "nah",
            ReviewScore::Worst => "worst",
        });

        let destination_file =
            destination_folder.join(PathBuf::from(&full_path).file_name().unwrap());
        let paths = DiskPaths {
            full_path,
            destination_folder,
            destination_file,
        };
        Ok(paths)
    }
}

#[derive(Debug)]
struct DiskPaths {
    full_path: String,
    destination_folder: PathBuf,
    destination_file: PathBuf,
}
