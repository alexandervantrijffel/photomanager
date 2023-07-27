use anyhow::Result;
use std::path::{Path, PathBuf};

#[tokio::test]
async fn test_get_photos() {
    let tempdir = init_env();
    write_image(&tempdir, "albumX", "123.jpg");
    let schema = photomanagerlib::model::new_schema();
    let response = schema
        .execute(
            "
{
  photosToReview {
    success
    output {
      baseUrl
      photos {
        album
        url
      }
    }
  }
}
",
        )
        .await;

    let json = serde_json::to_string(&response.data).unwrap();
    assert!(json.contains("/media/albumX/123.jpg"), "{}", json);
}

fn init_env() -> PathBuf {
    let tempdir = std::env::temp_dir();
    std::env::set_var("MEDIA_ROOT", tempdir.to_str().unwrap());
    std::env::set_var("PUBLIC_URL", "http://integration-test");
    tempdir
}

fn write_image(folder: &Path, album: &str, file_name: &str) {
    assert!(write_file(&folder.join(album).join(file_name), "123").is_ok());
}

fn write_file(path: &Path, content: &str) -> Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, content)?;
    Ok(())
}
