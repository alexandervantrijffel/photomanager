use anyhow::Result;
use async_graphql::value;
use rand::Rng;
use std::path::{Path, PathBuf};

// graphql subscription example test
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
    )?;
    // should not be listed because it has been reviewed before
    let unreviewed_best_photo_path = write_image(&media_dir, "albumX", "best-photo.jpg", "i")?;

    write_image(&media_dir, "albumX", "123.jpg", "i")?;

    let data = photomanagerlib::model::new_schema(Some(&media_dir))
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
        !unreviewed_best_photo_path.exists(),
        "best-photo should have been removed because it has been reviewed already"
    );
    Ok(())
}

#[tokio::test]
async fn test_undo() -> Result<()> {
    let media_dir = init_env()?;
    let best_photo_reviewed_path = write_reviewed_image(
        &media_dir,
        photomanagerlib::reviewscore::ReviewScore::Best,
        "albumX",
        "best-photo.jpg",
        "i",
    )?;
    let data = photomanagerlib::model::new_schema(Some(&media_dir))
        .execute(
            "
mutation {
  undo(path: \"/media/albumX/best-photo.jpg\", score: BEST) {
    success
    output   
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
            "undo": {
                "success": true,
                "output": ""
            }
        })
    );

    assert!(
        !best_photo_reviewed_path.exists(),
        "best-photo should have been removed because it was undone"
    );
    Ok(())
}

#[tokio::test]
async fn test_review_photo() -> Result<()> {
    let media_dir = init_env()?;
    let good_photo_path = write_image(&media_dir, "albumX", "good-photo.jpg", "i")?;
    let data = photomanagerlib::model::new_schema(Some(&media_dir))
        .execute(
            "
mutation {
  reviewPhoto(path: \"/media/albumX/good-photo.jpg\", score: GOOD) {
    success
    output   
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
            "reviewPhoto": {
                "success": true,
                "output": ""
            }
        })
    );

    assert!(PathBuf::from(&media_dir)
        .join(photomanagerlib::reviewscore::ReviewScore::Good.as_str())
        .join("albumX")
        .join("good-photo.jpg")
        .exists());

    assert!(
        !good_photo_path.exists(),
        "good-photo should have been removed because it was reviewed"
    );
    Ok(())
}

fn init_env() -> Result<String> {
    let tempdir = std::env::temp_dir().join("photomanager-tests");

    let mut rng = rand::thread_rng();
    let random_suffix: u32 = rng.gen_range(1..10000);

    let unique_filepath = photomanagerlib::fsops::get_unique_filepath(tempdir.to_str().unwrap())?;
    let path = format!("{}--{}", unique_filepath, random_suffix);

    std::env::set_var("PUBLIC_URL", "http://integration-test");
    Ok(path)
}

fn write_reviewed_image(
    folder: &str,
    score: photomanagerlib::reviewscore::ReviewScore,
    album: &str,
    file_name: &str,
    contents: &str,
) -> Result<PathBuf> {
    let path = PathBuf::from(folder)
        .join(score.as_str())
        .join(album)
        .join(file_name);
    write_file(&path, contents)?;
    Ok(path)
}

fn write_image(folder: &str, album: &str, file_name: &str, contents: &str) -> Result<PathBuf> {
    let path = PathBuf::from(folder).join(album).join(file_name);
    write_file(&path, contents)?;
    Ok(path)
}

fn write_file(path: &Path, content: &str) -> Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, content)?;
    Ok(())
}
