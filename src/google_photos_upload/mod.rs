mod album;
mod google_photos_client;

use anyhow::Result;
use std::sync::mpsc;
use tracing::{error, info, instrument};

use crate::image::PhotoReview as ReviewedPhoto;
use crate::reviewscore::ReviewScore;

use self::google_photos_client::{GooglePhotosClient, OauthSecrets};

struct UploadRequestContext {
    sender: mpsc::Sender<ReviewedPhoto>,
    enabled: bool,
}

// todo:
// query existing album instead of creating a new album every time
// get access token failed once (refresh token was expired or revoked?)
// refresh access token when it has expired
// retry google drive upload with new access token when it had to be refreshed
// add env vars for google drive uploads to k3s manifests
// make sure that google upload errors become visible
// perf: hashmap for known albums

pub fn upload_best_photos(review: ReviewedPhoto) -> Result<(), mpsc::SendError<ReviewedPhoto>> {
    if review.score != ReviewScore::Best {
        return Ok(());
    }
    UPLOAD_REQUESTER.with(|ctx| {
        if ctx.enabled {
            ctx.sender.send(review)
        } else {
            info!("Google photos upload is disabled because env vars are not set");
            Ok(())
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

    if !ctx.enabled {
        return ctx;
    }

    tokio::spawn(async move {
        let client = GooglePhotosClient::new(&oauth_secrets);
        for req in receiver {
            single_run_upload_photo(&req, &client).await;
        }
    });
    ctx
}

#[instrument]
async fn single_run_upload_photo(req: &ReviewedPhoto, client: &GooglePhotosClient) {
    if let Err(e) = client.upload_photo(req).await {
        error!("Failed to upload photo to Google Photos: {}", e);
    }
}
