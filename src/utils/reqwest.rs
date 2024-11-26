use std::fmt::{Display, Formatter};
use axum::http::HeaderValue;
use reqwest::{header, Client};
use reqwest::header::AUTHORIZATION;
use crate::errors::HikyakuError::{BuilderError, ParseError};
use crate::errors::HikyakuResult;

#[derive(Debug, Clone, Copy)]
pub(crate) enum AuthType {
    Bearer,
}

impl Display for AuthType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bearer => write!(f, "Bearer"),
        }
    }
}


pub(crate) fn get_client_with_token(token: &str, auth_type: AuthType) -> HikyakuResult<Client> {
    let mut header_value = HeaderValue::from_str(&format!("{} {}", auth_type, token))
        .map_err(|e| ParseError(format!("Failed to parse header value: {:#?}", e)))?;
    header_value.set_sensitive(true);
    let mut headers = header::HeaderMap::new();
    headers.insert(AUTHORIZATION, header_value);

    let client = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| BuilderError(format!("Failed to build client: {:#?}", e)))?;

    Ok(client)
}