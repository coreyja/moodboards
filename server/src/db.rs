use miette::IntoDiagnostic;
use rand::seq::IteratorRandom;
use sqlx::SqlitePool;

use crate::images_urls;

pub async fn next_image_for_moodboard(
    moodboard_id: i64,
    pool: SqlitePool,
) -> miette::Result<Option<&'static str>> {
    let rated_picture_urls = sqlx::query!(
        "SELECT url from PictureRatings WHERE moodboard_id = ?",
        moodboard_id
    )
    .fetch_all(&pool)
    .await
    .into_diagnostic()?;

    let unrated_picture_urls = images_urls().into_iter().filter(|url| {
        !rated_picture_urls
            .iter()
            .any(|rated_url| rated_url.url == **url)
    });

    Ok(unrated_picture_urls.choose(&mut rand::thread_rng()))
}
