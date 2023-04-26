use walkdir::{DirEntry, WalkDir};

fn file_is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn is_directory(entry: &DirEntry) -> bool {
    let result = entry.file_type().is_dir();
    println!("file_name {:?} is_directory {}", entry.file_name(), result);
    result
}

pub(crate) fn get_photo_paths_to_review() -> Vec<&'static str> {
    let root_dir = "/data/github.com/alexandervantrijffel/photomanager";
    for entry in WalkDir::new(root_dir)
        .into_iter()
        .filter_entry(|e| {
            !file_is_hidden(e)
                && e.path().to_str().is_some()
                && (e.path().to_str() == Some(root_dir) || !is_directory(e))
        })
        .filter_map(|e| e.ok())
    {
        // let entry = entry.unwrap();
        println!("{}", entry.path().display());
    }
    vec!["/photos/1.jpg", "/photos/2.jpg"]
}
