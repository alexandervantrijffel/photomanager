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

// use walkdir::{DirEntry, WalkDir};
// fn file_is_hidden(entry: &DirEntry) -> bool {
//     entry
//         .file_name()
//         .to_str()
//         .map(|s| s.starts_with('.'))
//         .unwrap_or(false)
// }

// fn is_directory(entry: &DirEntry) -> bool {
//     let result = entry.file_type().is_dir();
//     println!("file_name {:?} is_directory {}", entry.file_name(), result);
//     result
// }

// fn is_image(entry: &DirEntry) -> bool {
//     let result = entry
//         .path()
//         .extension()
//         .map(|s| s == "jpg" || s == "jpeg" || s == "png")
//         .unwrap_or(false);
//     println!("file_name {:?} is_image {}", entry.file_name(), result);
//     result
// }

// pub(crate) fn get_photo_paths_to_review_walkdir() -> Vec<String> {
//     let root_dir = "/data/github.com/alexandervantrijffel/photomanager";
//     // let images = vec!["/photos/1.jpg", "/photos/2.jpg"];
//     let mut images = Vec::new();
//     for entry in WalkDir::new(root_dir)
//         .into_iter()
//         .filter_entry(|e| {
//             !file_is_hidden(e)
//                 && e.path().to_str().is_some()
//                 && (e.path().to_str() == Some(root_dir) || (!is_directory(e) && is_image(e)))
//         })
//         .filter_map(|e| e.ok())
//     {
//         if is_directory(&entry) {
//             continue;
//         }

//         println!("{}", entry.path().display());
//         images.push(entry.path().to_str().unwrap().to_string());
//     }
//     images
// }
