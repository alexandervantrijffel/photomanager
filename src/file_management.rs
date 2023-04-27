use globwalk::{GlobError, GlobWalkerBuilder};

pub(crate) fn get_photo_paths_to_review() -> Result<Vec<String>, GlobError> {
    let root_dir = "/data/github.com/alexandervantrijffel/photomanager";
    let image_files =
        GlobWalkerBuilder::from_patterns(root_dir, &["*.{png,jpg,jpeg,gif}", "!best/*", "!soso/*"])
            .max_depth(10)
            .follow_links(true)
            .build()?
            .filter_map(Result::ok)
            .map(|img| img.path().to_str().unwrap().to_string())
            .take(10)
            .collect::<Vec<String>>();

    Ok(image_files)
}
