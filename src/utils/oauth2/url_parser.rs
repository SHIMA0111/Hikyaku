use url::Url;
use crate::utils::errors::{HikyakuError, HikyakuResult};


/// Extracts the protocol and hostname from the given URL string.
///
/// This function takes a URL string, parses it, and returns a tuple containing
/// the protocol (e.g., "http" or "https") and the hostname. If the hostname
/// cannot be extracted, it returns an empty string for the hostname.
pub(crate) fn extract_protocol_hostname(url: &str) -> HikyakuResult<(String, String)> {
    let parsed_url = Url::parse(url)
        .map_err(|e| HikyakuError::OAuth2Error(format!("URL parse failed: {:?}", e)))?;
    let protocol = parsed_url.scheme().to_string();
    Ok((protocol, parsed_url.host_str().unwrap_or("").to_string()))
}


/// Parses the given URL and extracts the base URL and port number.
///
/// This function takes a URL string, parses it, and returns a tuple containing
/// the base URL (with the path stripped) and the port number. If the URL does not
/// contain an explicit port, it returns the default port for the scheme.
pub(crate) fn parse_url(url: &str) -> HikyakuResult<(String, u16)> {
    let mut parsed_url = Url::parse(url)
        .map_err(|e| HikyakuError::OAuth2Error(format!("URL parse failed: {:?}", e)))?;
    let port = parsed_url
        .port_or_known_default()
        .ok_or_else(|| HikyakuError::OAuth2Error("Cannot get the port from the url".to_string()))?;
    parsed_url.set_path("");
    let url = parsed_url.as_str().to_string();

    Ok((url, port))
}