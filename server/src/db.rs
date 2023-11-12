use miette::IntoDiagnostic;
use rand::seq::IteratorRandom;
use sqlx::SqlitePool;

use crate::apis::pexels::{get_my_first_collection_media, PexelsImage};

pub(crate) async fn next_image_for_moodboard(
    pexels_api_key: &str,
    moodboard_id: i64,
    pool: SqlitePool,
) -> miette::Result<Option<PexelsImage>> {
    let rated = sqlx::query!(
        "SELECT pexels_id from PictureRatings WHERE moodboard_id = ?",
        moodboard_id
    )
    .fetch_all(&pool)
    .await
    .into_diagnostic()?;

    let unrated_picture_urls = get_my_first_collection_media(pexels_api_key)
        .await?
        .into_iter()
        .filter(|media| !rated.iter().any(|rated| rated.pexels_id == media.id));

    Ok(unrated_picture_urls.choose(&mut rand::thread_rng()))
}
