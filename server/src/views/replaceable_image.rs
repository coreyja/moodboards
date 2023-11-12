use maud::{html, Render};
use miette::Result;

use crate::{db::Img, AppState};

pub struct ReplaceableImage {
    image_url: String,
    image_id: i64,
}

impl ReplaceableImage {
    pub(crate) fn from_optional_img(media: Option<Img>) -> Option<Self> {
        let media = media?;

        Some(Self {
            image_url: media.url,
            image_id: media.pexels_id,
        })
    }

    pub async fn next(app_state: &AppState) -> Result<Option<Self>> {
        let next_image =
            crate::db::next_image_for_moodboard(app_state.moodboard_id, app_state.pool.clone())
                .await?;

        Ok(Self::from_optional_img(next_image))
    }
}

impl Render for ReplaceableImage {
    fn render(&self) -> maud::Markup {
        html! {
          div id="replaceable-image" {
            img src=(self.image_url) {}

            button cja-click={"/images/" (self.image_id) "/upvote/"} cja-method="POST" cja-replace-id="replaceable-image" {
                "Upvote Image"
            }
            button cja-click={"/images/" (self.image_id) "/downvote/"} cja-method="POST" cja-replace-id="replaceable-image" {
                "Downvote Image"
            }
          }
        }
    }
}
