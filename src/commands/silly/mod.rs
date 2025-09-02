mod capybara;
mod cat;
mod dog;
mod monkey;
mod panda;

pub use capybara::capybara;
pub use cat::cat;
pub use dog::dog;
pub use monkey::monkey;
pub use panda::panda;

/// Fetches an image URL from a given API endpoint and JSON path
async fn fetch_image_url(url: &str, json_path: &str) -> anyhow::Result<String> {
    let resp = reqwest::get(url).await?;
    let json = resp.json::<serde_json::Value>().await?;

    json.pointer(&format!("/{}", json_path.replace('.', "/")))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Image URL not found in response"))
}
