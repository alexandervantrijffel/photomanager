use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

pub fn can_safely_overwrite(source: &str, destination: &str) -> Result<bool> {
    if !PathBuf::from(destination).exists() {
        return Ok(true);
    }
    Ok(fs::read(source)? == fs::read(destination)?)
}

pub fn have_equal_contents(source: &str, destination: &str) -> Result<bool> {
    if !PathBuf::from(destination).exists() {
        return Ok(false);
    }
    Ok(fs::read(source)? == fs::read(destination)?)
}

pub fn safe_rename(source: &str, destination: &str) -> Result<()> {
    let destination_folder = Path::new(destination)
        .parent()
        .ok_or_else(|| anyhow!("Failed to get parent dir"))?;
    fs::create_dir_all(destination_folder).with_context(|| {
        format!(
            "Failed to create media target folder '{}'",
            &destination_folder.display()
        )
    })?;
    println!("Moving photo from {} to {}", source, destination);
    fs::rename(source, destination)
        .with_context(|| format!("Failed to move photo from {} to {}", source, destination))
}

pub fn get_unique_filepath(file_path: &str) -> Result<String> {
    let path = Path::new(file_path);
    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("Failed to get parent dir"))?;
    let title = path
        .file_stem()
        .and_then(|p| p.to_str())
        .ok_or_else(|| anyhow!("no file title"))?;

    let ext = path
        .extension()
        .and_then(|p| p.to_str())
        .map(|s| ".".to_owned() + s)
        .unwrap_or_else(|| "".to_owned());

    (1..=20)
        .find_map(|i| {
            let last_path_buf = dir.join(format!("{}-{}{}", title, i, ext));
            if !Path::new(&last_path_buf).exists() {
                Some(last_path_buf.to_str().unwrap().to_string())
            } else {
                None
            }
        })
        .ok_or(anyhow!(
            "Failed to find unique file path for: {}",
            file_path
        ))
}
