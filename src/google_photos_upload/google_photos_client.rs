use crate::google_photos_upload::album::get_album_id;
use crate::image::PhotoReview as ReviewedPhoto;
use crate::reqwops;
use anyhow::{Context, Result, bail};
// use oauth2::basic::BasicClient;
// use oauth2::reqwest;
// use oauth2::RefreshToken;
// use oauth2::{AuthUrl, ClientId, ClientSecret, TokenUrl};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fs};
use tracing::{debug, info};

#[derive(Debug)]
pub struct GooglePhotosClient {
    access_token: Result<String>,
    reqwest_client: Arc<reqwest::Client>,
}

impl GooglePhotosClient {
    pub fn new(oauth_secrets: &OauthSecrets) -> Self {
        Self {
            access_token: Self::get_access_token(oauth_secrets),
            reqwest_client: Arc::new(reqwest::Client::new()),
        }
    }
    pub async fn upload_photo(&self, req: &ReviewedPhoto) -> Result<()> {
        info!("Uploading photo to Google Photos: {:?}", req);

        if let Err(e) = &self.access_token {
            bail!(
                "Failed to get google photos access token. Upload to google photos is disabled. Error: {}",
                e
            );
        }

        let album_name = format!("001-best-{}", req.image.album_name);
        let album_id = get_album_id(
            &album_name,
            self.get_auth_headers()?,
            Arc::clone(&self.reqwest_client),
        )
        .await?;
        let upload_token = self
            .upload_image_bytes(&req.image.full_path)
            .await
            .with_context(|| "Failed to upload image to google photos")?;
        self.batch_create_media(&upload_token, album_id.as_str())
            .await
    }
    async fn upload_image_bytes(&self, image_path: &str) -> Result<String> {
        let img_bytes = fs::read(image_path)?;

        let mut headers = self.get_auth_headers()?;
        headers.insert(CONTENT_TYPE, "application/octet-stream".parse()?);
        headers.insert("X-Goog-Upload-Protocol", "raw".parse()?); //
        //
        let mime_type = match PathBuf::from(image_path)
            .extension()
            .context("Failed to get file extension of image_path")?
            .to_str()
            .context("Failed to get file extension of image_path")?
        {
            "jpg" | "jpeg" => Ok("image/jpeg"),
            "png" => Ok("image/png"),
            "gif" => Ok("image/gif"),
            "heic" => Ok("image/heic"),
            "tiff" => Ok("image/tiff"),
            "webp" => Ok("image/webp"),
            other_ext => Err(format!(
                "Mime type of exstension [{other_ext}] is not supported",
            )),
        }
        .map_err(|e| anyhow::anyhow!(e))?;

        headers.insert("X-Goog-Upload-Content-Type", mime_type.parse().unwrap()); //

        let response = Arc::clone(&self.reqwest_client)
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
        info!(
            "Upload completed with status: {}. Text: {}",
            status, response_body
        );

        Ok(response_body.to_owned())
    }

    async fn batch_create_media(&self, upload_token: &str, album_id: &str) -> Result<()> {
        let post_result = reqwops::post_json(
            "https://photoslibrary.googleapis.com/v1/mediaItems:batchCreate",
            self.get_auth_headers()?,
            Arc::clone(&self.reqwest_client),
            &json!({
                "albumId": album_id,
                "newMediaItems": [
                    {
                        "description": "test",
                        "simpleMediaItem": {
                            "uploadToken": upload_token
                        }
                    }
                ]
            }),
        )
        .await?;
        if !post_result.status.is_success() {
            return Err(anyhow::anyhow!(
                "Failed to batch create media in google photos. Status: {}. Response body: {}",
                post_result.status,
                post_result.response_body
            ));
        }

        debug!(
            "Batch create media completed with status: {}. Text: {}",
            post_result.status, post_result.response_body
        );
        Ok(())
    }

    fn get_access_token(oauth_secrets: &OauthSecrets) -> Result<String> {
        // this is needed to prevent the panic of a blocking reqwest call:
        // Cannot drop a runtime in a context where blocking is not allowed" panic in the blocking Client
        // see https://github.com/seanmonstar/reqwest/issues/1017
        tokio::task::block_in_place(|| {
            debug!("Getting Google Photos client access token");

            // this might not work!!
            // try the official example from
            // https://docs.rs/oauth2/latest/oauth2/#example-synchronous-blocking-api

            // Manually create and execute the OAuth2 token refresh request
            let http_client = reqwest::blocking::Client::new();

            // OAuth2 token endpoint
            let token_url = "https://oauth2.googleapis.com/token";

            // Prepare the OAuth2 token request parameters
            let params = [
                ("client_id", oauth_secrets.client_id.as_str()),
                ("client_secret", oauth_secrets.client_secret.as_str()),
                ("refresh_token", oauth_secrets.refresh_token.as_str()),
                ("grant_type", "refresh_token"),
            ];

            // Execute the request to exchange the refresh token for an access token
            let response = http_client
                .post(token_url)
                .form(&params)
                .header("Accept", "application/json")
                .send()?;

            if !response.status().is_success() {
                bail!("Token refresh failed: {}", response.text()?);
            }

            let token_response: TokenResponse = response.json()?;
            Ok(token_response.access_token)

            // let client = BasicClient::new(ClientId::new(oauth_secrets.client_id.clone()))
            //     .set_client_secret(ClientSecret::new(oauth_secrets.client_secret.clone()))
            //     .set_auth_uri(AuthUrl::new(
            //         "https://accounts.google.com/o/oauth2/auth".into(),
            //     )?)
            //     .set_token_uri(TokenUrl::new("https://oauth2.googleapis.com/token".into())?);

            // Set the URL the user will be redirected to after the authorization process (not
            // used)
            // .set_redirect_uri(RedirectUrl::new(
            //     "http://localhost:3000/auth/google/callback".to_string(),
            // )

            // let http_client = reqwest::blocking::ClientBuilder::new()
            //     // Following redirects opens the client up to SSRF vulnerabilities.
            //     .redirect(reqwest::redirect::Policy::none())
            //     .build()
            //     .expect("Client should build");

            // Unwrapping token_result will either produce a Token or a RequestTokenError.
            // Ok(client
            //     .exchange_refresh_token(&RefreshToken::new(oauth_secrets.refresh_token.clone()))
            //     .request(&http_client)?
            //     .access_token()
            //     .secret()
            //     .clone())
        })
    }

    fn get_auth_headers(&self) -> Result<hyper::HeaderMap> {
        let mut headers = hyper::HeaderMap::new();
        match &self.access_token {
            Ok(token) => {
                headers.insert(AUTHORIZATION, format!("Bearer {}", &token).parse().unwrap());
            }
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "cannot upload image to google because no access token is available"
                        .to_string()
                ));
            }
        }

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
        let mut oauth_secrets = Self {
            client_id: Self::string_from_env_or_default("GOOGLE_CLIENT_ID"),
            client_secret: Self::string_from_env_or_default("GOOGLE_CLIENT_SECRET"),
            refresh_token: Self::string_from_env_or_default("GOOGLE_REFRESH_TOKEN"),
            is_valid: false,
        };
        oauth_secrets.is_valid = !oauth_secrets.client_id.is_empty()
            && !oauth_secrets.client_secret.is_empty()
            && !oauth_secrets.refresh_token.is_empty();

        oauth_secrets
    }

    fn string_from_env_or_default(env_var_name: &str) -> String {
        env::var(env_var_name).unwrap_or_else(|_| String::new())
    }
}

// Parse the response to get the access token
#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    expires_in: u64,
    #[allow(dead_code)]
    token_type: String,
}
