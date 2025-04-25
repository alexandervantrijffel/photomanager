use anyhow::{Context, Result, anyhow};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tracing::info;

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

pub fn rename_with_create_dir_all(source: &str, destination: &str, mode: u32) -> Result<()> {
    let destination_folder = Path::new(destination)
        .parent()
        .ok_or_else(|| anyhow!("Failed to get parent dir"))?;
    fs::create_dir_all(destination_folder).with_context(|| {
        format!(
            "Failed to create media target folder '{}'",
            &destination_folder.display()
        )
    })?;
    chmod(destination_folder.to_str().unwrap(), mode)?;

    info!("Moving photo from {} to {}", source, destination);
    fs::rename(source, destination)
        .with_context(|| format!("Failed to move photo from {source} to {destination}"))
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
        .map_or_else(String::new, |s| String::from(".") + s);

    (1..=1000)
        .find_map(|i| {
            let last_path_buf = dir.join(format!("{title}-{i}{ext}"));
            if Path::new(&last_path_buf).exists() {
                None
            } else {
                Some(last_path_buf.to_str().unwrap().into())
            }
        })
        .ok_or_else(|| anyhow!("Failed to find unique file path for: {}", file_path))
}

pub fn chmod(file_path: &str, mode: u32) -> Result<()> {
    let mut perms = fs::metadata(file_path)?.permissions();
    perms.set_mode(mode);
    fs::set_permissions(file_path, perms).map_err(std::convert::Into::into)
}
