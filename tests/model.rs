use anyhow::Result;
use async_graphql::value;
use std::path::{Path, PathBuf};

#[tokio::test]
async fn test_get_photos() -> Result<()> {
    let media_dir = init_env()?;
    write_image(&media_dir, "albumX", "123.jpg");
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

    let json = serde_json::to_string(&response.data)?;
    assert!(json.contains("/media/albumX/123.jpg"), "{}", json);
    assert_eq!(
        response.data,
        value!({
            "photosToReview": {
                "success": true,
                "output": {
                    "baseUrl": "http://integration-test",
                    "photos": [
                        {
                            "album": format!("{}/albumX", media_dir),
                            "url": "/media/albumX/123.jpg"
                        }
                    ]
                }
            }
        })
    );
    Ok(())
}

fn init_env() -> Result<String> {
    let tempdir = std::env::temp_dir().join("photomanager-tests");
    let path = photomanagerlib::fsops::get_unique_filepath(tempdir.to_str().unwrap())?;

    std::env::set_var("MEDIA_ROOT", &path);
    std::env::set_var("PUBLIC_URL", "http://integration-test");
    Ok(path)
}

fn write_image(folder: &str, album: &str, file_name: &str) {
    assert!(write_file(
        &PathBuf::from(folder).join(album).join(file_name),
        "image file contents"
    )
    .is_ok());
}

fn write_file(path: &Path, content: &str) -> Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, content)?;
    Ok(())
}
