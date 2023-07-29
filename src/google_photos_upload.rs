use crate::image::PhotoReview as ReviewedPhoto;
use crate::reviewscore::ReviewScore;
use anyhow::Result;

pub fn upload_best_photos(review: ReviewedPhoto) -> Result<()> {
    if review.score != ReviewScore::Best {
        println!("Uploading photo: {:?}", review);
        return Ok(());
    }
    Ok(())
}
