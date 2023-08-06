use anyhow::{anyhow, Context, Result};
use hyper::HeaderMap;
use serde::Deserialize;
use serde_json::json;
use tracing::{event, Level};
pub async fn get_album_id(title: &str, auth_headers: HeaderMap) -> Result<String> {
    Ok(create_album(title, auth_headers)
        .await
        .with_context(|| format!("failed to create google photos album {}", &title))?
        .id)
}

async fn create_album(title: &str, auth_headers: HeaderMap) -> Result<Album> {
    let post_result = crate::reqwops::post_json(
        "https://photoslibrary.googleapis.com/v1/albums",
        auth_headers,
        &json!({
            "album": {
                "title": title
            }
        }),
    )
    .await?;

    event!(
        Level::DEBUG,
        "Create Google Drive Album Response: {}",
        post_result.response_body
    );

    serde_json::from_str::<Album>(&post_result.response_body).map_err(|e| {
        anyhow!(
            "Failed to create album in google photos. Response body: {}. Error: {}",
            post_result.response_body,
            e
        )
    })
}

#[derive(Deserialize, Debug)]
struct Album {
    id: String,
    // title: String,
    // product_url: String,
    // is_writeable: bool,
}
