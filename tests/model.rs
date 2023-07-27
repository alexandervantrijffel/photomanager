use anyhow::Result;
use async_graphql::value;
use std::path::{Path, PathBuf};

// subscription example test
// https://github.com/async-graphql/async-graphql/blob/bdbd1f8a9040edd7c45aee7275b6feba2e696052/tests/raw_ident.rs#L59
//
#[tokio::test]
async fn test_get_photos() -> Result<()> {
    let media_dir = init_env()?;
    write_reviewed_image(
        &media_dir,
        photomanagerlib::reviewscore::ReviewScore::Best,
        "albumX",
        "best-photo.jpg",
        "i",
    );
    // should not be listed because it has been reviewed before
    write_image(&media_dir, "albumX", "best-photo.jpg", "i");

    write_image(&media_dir, "albumX", "123.jpg", "i");

    let schema = photomanagerlib::model::new_schema();
    let data = schema
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
        .await
        .into_result()
        .unwrap()
        .data;

    assert_eq!(
        data,
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

    assert!(
        !PathBuf::from(&media_dir)
            .join("albumX")
            .join("best-photo.jpg")
            .exists(),
        "best-photo should have been removed because it has been reviewed already"
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

fn write_reviewed_image(
    folder: &str,
    score: photomanagerlib::reviewscore::ReviewScore,
    album: &str,
    file_name: &str,
    contents: &str,
) {
    assert!(write_file(
        &PathBuf::from(folder)
            .join(score.as_str())
            .join(album)
            .join(file_name),
        contents
    )
    .is_ok());
}

fn write_image(folder: &str, album: &str, file_name: &str, contents: &str) {
    assert!(write_file(&PathBuf::from(folder).join(album).join(file_name), contents).is_ok());
}

fn write_file(path: &Path, content: &str) -> Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, content)?;
    Ok(())
}
