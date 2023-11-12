use maud::{html, Render};
use miette::Result;

use crate::{apis::pexels::PexelsImage, AppState};

pub struct ReplaceableImage {
    pexels_img: PexelsImage,
}

impl ReplaceableImage {
    pub(crate) fn from_optional_img(media: Option<PexelsImage>) -> Option<Self> {
        let media = media?;

        Some(Self { pexels_img: media })
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
            figure {
                img src=(self.pexels_img.src.large) {}
                figcaption {
                    "This "
                    a href=(format!("https://www.pexels.com/photo/{}", self.pexels_img.id)) { "Photo" }
                    " was taken by "
                    a href=(self.pexels_img.photographer_url) { (self.pexels_img.photographer) }
                    " on Pexels."
                }
            }

            button cja-click={"/images/" (self.pexels_img.id) "/upvote/"} cja-method="POST" cja-replace-id="replaceable-image" {
                "Upvote Image"
            }
            button cja-click={"/images/" (self.pexels_img.id) "/downvote/"} cja-method="POST" cja-replace-id="replaceable-image" {
                "Downvote Image"
            }
          }
        }
    }
}
