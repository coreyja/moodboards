use miette::IntoDiagnostic;
use rand::seq::SliceRandom;
use sqlx::SqlitePool;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Img {
    pub(crate) pexels_id: i64,
    pub(crate) url: String,
}

pub(crate) async fn next_image_for_moodboard(
    moodboard_id: i64,
    pool: SqlitePool,
) -> miette::Result<Option<Img>> {
    let unrated = sqlx::query_as!(
        Img,
        "SELECT pexels_id, url from Pictures
        LEFT JOIN PictureRatings using (pexels_id)
        WHERE Pictures.moodboard_id = ? AND PictureRatings.pexels_id is null",
        moodboard_id
    )
    .fetch_all(&pool)
    .await
    .into_diagnostic()?;

    Ok(unrated.choose(&mut rand::thread_rng()).cloned())
}
