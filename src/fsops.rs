use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};

pub fn can_safely_overwrite(source: &str, destination: &str) -> Result<bool> {
    if !PathBuf::from(destination).exists() {
        return Ok(true);
    }
    let source_file_contents = fs::read(source)?;
    let destination_file_contents = fs::read(destination)?;
    Ok(destination_file_contents == source_file_contents)
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
