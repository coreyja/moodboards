use miette::{IntoDiagnostic, Result};
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

pub async fn get_collection_media(
    api_key: &str,
    collection_id: &str,
) -> miette::Result<Vec<Value>> {
    let client = reqwest::Client::new();
    let url = format!("https://api.pexels.com/v1/collections/{collection_id}");
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
    let collections = response["media"]
        .as_array()
        .ok_or_else(|| miette::miette!("Failed to parse collections"))?;

    Ok(collections.clone())
}

pub async fn get_my_first_collection_media(api_key: &str) -> Result<Vec<Value>> {
    let collections = get_my_collections(api_key).await?;
    let collection_id = &collections[0]["id"].as_str().ok_or_else(|| {
        miette::miette!("Failed to parse collection id from {:?}", collections[0])
    })?;
    let media = get_collection_media(api_key, collection_id).await?;

    Ok(media)
}
