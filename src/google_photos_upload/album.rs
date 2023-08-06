use anyhow::{anyhow, Context, Result};
use hyper::HeaderMap;
use serde::Deserialize;
use serde_json::json;
pub async fn get_album_id(title: &str, auth_headers: HeaderMap) -> Result<String> {
    Ok(create_album(title, auth_headers)
        .await
        .context(format!("failed to create google photos album {}", &title))?
        .id)
}

async fn create_album(title: &str, auth_headers: HeaderMap) -> Result<Album> {
    let res = reqwest::Client::new()
        .post("https://photoslibrary.googleapis.com/v1/albums")
        .headers(auth_headers)
        .json(&json!({
            "album": {
                "title": title
            }
        }))
        .send()
        .await?;

    let response_body = &res.text().await?;
    println!("Create Google Drive Album Response: {}", response_body);

    serde_json::from_str::<Album>(response_body).map_err(|e| {
        anyhow!(
            "Failed to create album in google photos. Response body: {}. Error: {}",
            response_body,
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
