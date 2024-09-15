use url::Url;
use crate::utils::errors::{HikyakuError, HikyakuResult};

pub(crate) fn extract_protocol_hostname(url: &str) -> HikyakuResult<(String, String)> {
    let parsed_url = Url::parse(url)
        .map_err(|e| HikyakuError::OAuth2Error(format!("URL parse failed: {:?}", e)))?;
    let protocol = parsed_url.scheme().to_string();
    Ok((protocol, parsed_url.host_str().unwrap_or("").to_string()))
}