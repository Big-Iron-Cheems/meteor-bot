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

/// Fetch an image URL from a JSON API response.
///
/// `json_pointer` must be a valid JSON Pointer (RFC 6901): a slash-delimited path,
///  e.g. `"/data/url"` or `"/link"`. The leading slash is required.
pub(super) async fn fetch_image_url(
    client: &reqwest::Client,
    url: url::Url,
    json_pointer: &str,
) -> anyhow::Result<String> {
    let json = client.get(url).send().await?.json::<serde_json::Value>().await?;

    json.pointer(json_pointer)
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
        .ok_or_else(|| anyhow::anyhow!("Image URL not found at pointer '{json_pointer}'"))
}
