use miette::IntoDiagnostic;
use rand::seq::SliceRandom;
use sqlx::SqlitePool;

use crate::apis::pexels::PexelsImage;

pub(crate) async fn next_image_for_moodboard(
    moodboard_id: i64,
    pool: SqlitePool,
) -> miette::Result<Option<PexelsImage>> {
    let unrated = sqlx::query!(
        "SELECT json from Pictures
        LEFT JOIN PictureRatings using (pexels_id)
        WHERE Pictures.moodboard_id = ? AND PictureRatings.pexels_id is null",
        moodboard_id
    )
    .fetch_all(&pool)
    .await
    .into_diagnostic()?;

    let chosen = unrated.choose(&mut rand::thread_rng());

    if let Some(chosen) = chosen {
        let image: PexelsImage = serde_json::from_str(&chosen.json).into_diagnostic()?;
        Ok(Some(image))
    } else {
        Ok(None)
    }
}
