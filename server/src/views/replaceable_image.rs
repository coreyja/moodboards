use maud::{html, Render};
use miette::Result;

use crate::{images_urls, AppState};

pub struct ReplaceableImage {
    image_url: String,
    image_id: i64,
}

impl ReplaceableImage {
    pub fn from_url(image_url: impl Into<String>) -> Self {
        let image_url = image_url.into();
        // TODO: This is NOT how ids should work
        let image_id = images_urls()
            .iter()
            .position(|u| (&image_url) == u)
            .unwrap() as i64;

        Self {
            image_url,
            image_id,
        }
    }

    pub fn from_optional_url(image_url: Option<impl Into<String>>) -> Option<Self> {
        let image_url = image_url?;

        Some(Self::from_url(image_url))
    }

    pub async fn next(app_state: &AppState) -> Result<Option<Self>> {
        let next_image_url =
            crate::db::next_image_for_moodboard(app_state.moodboard_id, app_state.pool.clone())
                .await
                .unwrap();

        Ok(Self::from_optional_url(next_image_url))
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
