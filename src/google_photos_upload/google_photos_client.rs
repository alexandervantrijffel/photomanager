use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::{env, fs};

use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{AuthUrl, ClientId, ClientSecret, RefreshToken, TokenResponse, TokenUrl};

use crate::google_photos_upload::album::get_album_id;
use crate::image::PhotoReview as ReviewedPhoto;

use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

pub struct GooglePhotosClient {
    access_token: Result<String>,
}

impl GooglePhotosClient {
    pub fn new(oauth_secrets: &OauthSecrets) -> GooglePhotosClient {
        GooglePhotosClient {
            access_token: Self::get_access_token(oauth_secrets),
        }
    }
    pub async fn upload_photo(&self, req: ReviewedPhoto) -> Result<()> {
        println!("Uploading photo to Google Photos: {:?}", req);

        if let Err(e) = &self.access_token {
            bail!("Failed to get google photos access token. Upload to google photos is disabled. Error: {}", e);
        }

        let album_name = format!("001-best-{}", req.image.album_name);
        let album_id = get_album_id(&album_name, self.get_auth_headers()?).await?;
        let upload_token = self
            .upload_image_bytes(&req.image.full_path)
            .await
            .context("Failed to upload image to google photos: {:?}")?;
        self.batch_create_media(&upload_token, album_id.as_str())
            .await
    }
    async fn upload_image_bytes(&self, image_path: &str) -> Result<String> {
        let img_bytes = fs::read(image_path)?;

        let mut headers = self.get_auth_headers()?;
        headers.insert(CONTENT_TYPE, "application/octet-stream".parse().unwrap());
        headers.insert("X-Goog-Upload-Protocol", "raw".parse().unwrap()); //
                                                                          //
        let mime_type = match PathBuf::from(image_path)
            .extension()
            .unwrap()
            .to_str()
            .expect("Failed to get file extension of image_path")
        {
            "jpg" => Ok("image/jpeg"),
            "jpeg" => Ok("image/jpeg"),
            "png" => Ok("image/png"),
            "gif" => Ok("image/gif"),
            "heic" => Ok("image/heic"),
            "tiff" => Ok("image/tiff"),
            "webp" => Ok("image/webp"),
            other_ext => Err(format!(
                "Mime type of exstension [{}] is not supported",
                other_ext
            )),
        }
        .expect("Failed to get mime type of extension");

        headers.insert("X-Goog-Upload-Content-Type", mime_type.parse().unwrap()); //
                                                                                  // accepted file types BMP, GIF, HEIC, ICO, JPG, PNG, TIFF, WEBP, some RAW files.

        let client = reqwest::Client::new();
        let response = client
            .post("https://photoslibrary.googleapis.com/v1/uploads")
            .headers(headers)
            .body(img_bytes)
            .send()
            .await?;

        let status = &response.status();
        let response_body = &response.text().await?;
        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Failed to upload image to google photos. Status: {}. Response body: {}",
                status,
                response_body
            ));
        }
        println!(
            "Upload completed with status: {}. Text: {}",
            status, response_body
        );

        Ok(response_body.to_owned())
    }

    async fn batch_create_media(&self, upload_token: &str, album_id: &str) -> Result<()> {
        let client = reqwest::Client::new();
        let response = client
            .post("https://photoslibrary.googleapis.com/v1/mediaItems:batchCreate")
            .headers(self.get_auth_headers()?)
            .json(&json!({
                "albumId": album_id,
                "newMediaItems": [
                    {
                        "description": "test",
                        "simpleMediaItem": {
                            "uploadToken": upload_token
                        }
                    }
                ]
            }))
            .send()
            .await?;

        let status = &response.status();
        let response_body = &response.text().await?;
        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Failed to batch create media in google photos. Status: {}. Response body: {}",
                status,
                response_body
            ));
        }

        Ok(println!(
            "Batch create media completed with status: {}. Text: {}",
            status, response_body
        ))
    }
    fn get_access_token(oauth_secrets: &OauthSecrets) -> Result<String> {
        // this is needed to prevent the panic of a blocking reqwest call:
        // Cannot drop a runtime in a context where blocking is not allowed" panic in the blocking Client
        // see https://github.com/seanmonstar/reqwest/issues/1017
        tokio::task::block_in_place(|| {
            println!("Getting Google Photos client access token");
            let client = BasicClient::new(
                ClientId::new(oauth_secrets.client_id.clone()),
                Some(ClientSecret::new(oauth_secrets.client_secret.clone())),
                AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string())?,
                Some(TokenUrl::new(
                    "https://oauth2.googleapis.com/token".to_string(),
                )?),
            );
            // Set the URL the user will be redirected to after the authorization process (not
            // used)
            // .set_redirect_uri(RedirectUrl::new(
            //     "http://localhost:3000/auth/google/callback".to_string(),
            // )

            // Unwrapping token_result will either produce a Token or a RequestTokenError.
            Ok(client
                .exchange_refresh_token(&RefreshToken::new(oauth_secrets.refresh_token.clone()))
                .request(http_client)?
                .access_token()
                .secret()
                .clone())
        })
    }

    fn get_auth_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        match &self.access_token {
            Ok(token) => {
                headers.insert(AUTHORIZATION, format!("Bearer {}", &token).parse().unwrap());
            }
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "cannot upload image to google because no access token is available"
                        .to_string()
                ))
            }
        };

        Ok(headers)
    }
}

pub struct OauthSecrets {
    client_id: String,
    client_secret: String,
    refresh_token: String,
    pub is_valid: bool,
}

impl OauthSecrets {
    pub fn from_env() -> Self {
        let mut oauth_secrets = OauthSecrets {
            client_id: OauthSecrets::string_from_env_or_default("GOOGLE_CLIENT_ID"),
            client_secret: OauthSecrets::string_from_env_or_default("GOOGLE_CLIENT_SECRET"),
            refresh_token: OauthSecrets::string_from_env_or_default("GOOGLE_REFRESH_TOKEN"),
            is_valid: false,
        };
        oauth_secrets.is_valid = !oauth_secrets.client_id.is_empty()
            && !oauth_secrets.client_secret.is_empty()
            && !oauth_secrets.refresh_token.is_empty();

        oauth_secrets
    }

    fn string_from_env_or_default(env_var_name: &str) -> String {
        env::var(env_var_name).unwrap_or("".to_string())
    }
}
