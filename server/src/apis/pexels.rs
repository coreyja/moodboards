use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub async fn get_my_collections(api_key: &str) -> Result<Vec<Value>> {
    let client = reqwest::Client::new();
    let url = "https://api.pexels.com/v1/collections";
    let response = client
        .get(url)
        .header("Authorization", api_key)
        .send()
        .await
        .into_diagnostic()?;
    let response = response
        .json::<serde_json::Value>()
        .await
        .into_diagnostic()?;
    let collections = response["collections"]
        .as_array()
        .ok_or_else(|| miette::miette!("Failed to parse collections"))?;

    Ok(collections.clone())
}

#[derive(Serialize, Deserialize)]
struct CollectionResponse {
    id: String,
    media: Vec<PexelsMedia>,
}
impl CollectionResponse {
    fn images(self) -> Vec<PexelsImage> {
        self.media
            .into_iter()
            .filter_map(|media| match media {
                PexelsMedia::Photo(image) => Some(image),
                _ => None,
            })
            .collect()
    }
}

pub(crate) async fn get_collection_media(
    api_key: &str,
    collection_id: &str,
) -> miette::Result<Vec<PexelsImage>> {
    let client = reqwest::Client::new();
    let url = format!("https://api.pexels.com/v1/collections/{collection_id}");
    let response = client
        .get(url)
        .header("Authorization", api_key)
        .send()
        .await
        .into_diagnostic()?;
    // let response = response
    //     .json::<CollectionResponse>()
    //     .await
    //     .into_diagnostic()?;
    let response = response
        .json::<serde_json::Value>()
        .await
        .into_diagnostic()?;
    dbg!(response.clone());

    let response = serde_json::from_value::<CollectionResponse>(response).into_diagnostic()?;

    Ok(response.images())
}

pub(crate) async fn get_my_first_collection_media(api_key: &str) -> Result<Vec<PexelsImage>> {
    let collections = get_my_collections(api_key).await?;
    let collection_id = &collections[0]["id"].as_str().ok_or_else(|| {
        miette::miette!("Failed to parse collection id from {:?}", collections[0])
    })?;
    let media = get_collection_media(api_key, collection_id).await?;

    Ok(media)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct ImageSrcSizes {
    pub(crate) landscape: String,
    pub(crate) large: String,
    pub(crate) large2x: String,
    pub(crate) medium: String,
    pub(crate) original: String,
    pub(crate) portrait: String,
    pub(crate) small: String,
    pub(crate) tiny: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum PexelsMedia {
    Photo(PexelsImage),
    Video(PexelsVideo),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct PexelsImage {
    pub(crate) avg_color: String,
    pub(crate) height: i64,
    pub(crate) id: i64,
    pub(crate) liked: bool,

    pub(crate) photographer: String,
    pub(crate) photographer_id: i64,
    pub(crate) photographer_url: String,
    pub(crate) src: ImageSrcSizes,
    pub(crate) url: String,
    pub(crate) width: i64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PexelsUser {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) url: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PexelsVideo {
    pub(crate) duration: i64,
    pub(crate) height: i64,
    pub(crate) id: i64,
    pub(crate) image: String,
    pub(crate) url: String,
    pub(crate) user: PexelsUser,
    pub(crate) width: i64,
}
