use anyhow::Result;
use std::sync::mpsc;
use std::{env, thread};

use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, RefreshToken, TokenResponse, TokenUrl};

use crate::image::PhotoReview as ReviewedPhoto;
use crate::reviewscore::ReviewScore;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
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
            Ok(())
        } else {
            ctx.sender.send(review)
        }
    })
}

thread_local! {
    static UPLOAD_REQUESTER: UploadRequestContext = {
        let (sender, receiver) = mpsc::channel::<ReviewedPhoto>();

        let oauth_secrets = OauthSecrets::from_env();

        let ctx = UploadRequestContext{enabled: oauth_secrets.is_valid, sender};

        if ctx.enabled {
            thread::spawn( || async move  {
                println!("Starting Google Photos upload thread");
                let client = GooglePhotosClient::new(&oauth_secrets);

                for req in receiver {
                    client.upload_photo(req).await;
                }
            });
        } else {
            println!("Google photos upload is disabled because env vars are not set");
        }
        ctx
    };
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
    async fn upload_photo(&self, req: ReviewedPhoto) {
        println!("Uploading photo to Google Photos: {:?}", req);

        if !&self.access_token.is_ok() {
            println!(
                "Failed to get google photos access token. Upload to google photos is disabled."
            );
            return;
        }

        let album_name = format!("001-best-{}", req.image.album_name);
        match create_album(self.access_token.as_ref().unwrap(), &album_name).await {
            Ok(_) => println!("Created google photos album {}", &album_name),
            Err(e) => println!(
                "Failed to create google photos album {}: {:?}",
                &album_name, e
            ),
        }
    }
    fn get_access_token(oauth_secrets: &OauthSecrets) -> Result<String> {
        println!("Getting Google Photos client access token");
        // Create an OAuth2 client by specifying the client ID, client secret, authorization URL and
        // token URL.
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

async fn create_album(access_token: &str, album_name: &str) -> Result<()> {
    let url = "https://photoslibrary.googleapis.com/v1/albums";

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(format!("Bearer {}", access_token).as_str())?,
    );

    let body = json!({
        "album": {
            "title": album_name
        }
    });

    let client = reqwest::Client::new();
    let res = client.post(url).headers(headers).json(&body).send().await?;
    println!("Response: {:?}", res.text().await?);

    Ok(())
}
