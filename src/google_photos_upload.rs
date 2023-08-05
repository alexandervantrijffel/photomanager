use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::sync::mpsc;
use std::{env, fs};

use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, RefreshToken, TokenResponse, TokenUrl};

use crate::image::PhotoReview as ReviewedPhoto;
use crate::reviewscore::ReviewScore;

use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

struct UploadRequestContext {
    sender: mpsc::Sender<ReviewedPhoto>,
    enabled: bool,
}

pub fn upload_best_photos(review: ReviewedPhoto) -> Result<(), mpsc::SendError<ReviewedPhoto>> {
    if review.score != ReviewScore::Best {
        return Ok(());
    }
    UPLOAD_REQUESTER.with(|ctx| {
        if !ctx.enabled {
            println!("Google photos upload is disabled because env vars are not set");
            Ok(())
        } else {
            ctx.sender.send(review)
        }
    })
}

thread_local! {
    static UPLOAD_REQUESTER: UploadRequestContext = init_upload_requester();
}

fn init_upload_requester() -> UploadRequestContext {
    let (sender, receiver) = mpsc::channel::<ReviewedPhoto>();

    let oauth_secrets = OauthSecrets::from_env();

    let ctx = UploadRequestContext {
        enabled: oauth_secrets.is_valid,
        sender,
    };

    if ctx.enabled {
        tokio::spawn(async move {
            println!("Starting Google Photos upload thread");

            let client = GooglePhotosClient::new(&oauth_secrets);
            for req in receiver {
                if let Err(e) = client.upload_photo(req).await {
                    println!("Failed to upload photo to Google Photos: {}", e);
                };
            }
        });
    } else {
        println!("Google photos upload is disabled because env vars are not set");
    }
    ctx
}

struct GooglePhotosClient {
    access_token: Result<String>,
}

impl GooglePhotosClient {
    fn new(oauth_secrets: &OauthSecrets) -> GooglePhotosClient {
        GooglePhotosClient {
            access_token: Self::get_access_token(oauth_secrets),
        }
    }
    async fn upload_photo(&self, req: ReviewedPhoto) -> Result<()> {
        println!("Uploading photo to Google Photos: {:?}", req);

        if let Err(e) = &self.access_token {
            bail!("Failed to get google photos access token. Upload to google photos is disabled. Error: {}", e);
        }

        let album_name = format!("001-best-{}", req.image.album_name);
        self.create_album(&album_name).await.context(format!(
            "failed to create google photos album {}",
            &album_name
        ))?;

        println!("Created google photos album {}", &album_name);
        let upload_token = self
            .upload_image_bytes(&req.image.full_path)
            .await
            .context("Failed to upload image to google photos: {:?}")?;
        Ok(())
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

    fn get_access_token(oauth_secrets: &OauthSecrets) -> Result<String> {
        // this is needed to prevent the panic of a blocking reqwest call:
        // Cannot drop a runtime in a context where blocking is not allowed" panic in the blocking Client
        // see https://github.com/seanmonstar/reqwest/issues/1017
        tokio::task::block_in_place(|| {
            println!("Getting Google Photos client access token");
            // Create an OAuth2 client by specifying the client ID, client secret, authorization URL and
            // token URL.
            //
            let client = BasicClient::new(
                ClientId::new(oauth_secrets.client_id.clone()),
                Some(ClientSecret::new(oauth_secrets.client_secret.clone())),
                AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string())?,
                Some(TokenUrl::new(
                    "https://oauth2.googleapis.com/token".to_string(),
                )?),
            )
            // Set the URL the user will be redirected to after the authorization process.
            .set_redirect_uri(RedirectUrl::new(
                "http://localhost:3000/auth/google/callback".to_string(),
            )?);

            // Unwrapping token_result will either produce a Token or a RequestTokenError.
            let token_result = client
                .exchange_refresh_token(&RefreshToken::new(oauth_secrets.refresh_token.clone()))
                .request(http_client)?;

            println!(
                "Google Photos client access token: {}",
                token_result.access_token().secret()
            );

            Ok(token_result.access_token().secret().clone())
        })
    }

    async fn create_album(&self, album_name: &str) -> Result<()> {
        let url = "https://photoslibrary.googleapis.com/v1/albums";

        let body = json!({
            "album": {
                "title": album_name
            }
        });

        let client = reqwest::Client::new();
        let res = client
            .post(url)
            .headers(self.get_auth_headers()?)
            .json(&body)
            .send()
            .await?;
        println!("Response: {:?}", res.text().await?);

        Ok(())
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

struct OauthSecrets {
    client_id: String,
    client_secret: String,
    refresh_token: String,
    is_valid: bool,
}

impl OauthSecrets {
    fn from_env() -> Self {
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
