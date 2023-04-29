use std::error::Error;

use globwalk::GlobWalkerBuilder;

pub(crate) struct FileManager {
    pub root_dir: String,
}

impl FileManager {
    pub fn new() -> Self {
        FileManager {
            root_dir: shellexpand::env("$HOME/pictures/photomanager-test/albumx")
                .unwrap()
                .to_string(),
        }
    }
    pub fn get_photo_paths_to_review(&self) -> Result<Vec<String>, Box<dyn Error>> {
        self.find_image_files()
    }

    fn find_image_files(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let image_files = GlobWalkerBuilder::from_patterns(
            self.root_dir.as_str(),
            &["*.{png,jpg,jpeg,gif}", "!best/*", "!soso/*"],
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
}
