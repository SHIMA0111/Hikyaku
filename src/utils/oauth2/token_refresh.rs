use std::time::Duration;
use log::error;
use oauth2::basic::BasicClient;
use oauth2::{RefreshToken, TokenResponse};
use oauth2::reqwest::async_http_client;
use time::OffsetDateTime;
use crate::utils::oauth2::{Token};

/// Refreshes an OAuth2 token using a refresh token.
///
/// This function is used automatically in [`SecretData::get_access_token`], 
/// hence it is marked as private.
///
/// # Arguments
///
/// * `client` - A reference to the `BasicClient` used for the OAuth2 flow.
/// * `refresh_token` - The refresh token string to be used to acquire a new access token.
/// * `scopes` - A slice of scope strings for which the token is requested.
///
/// # Returns
///
/// Optionally returns a new `Token` object if the refresh is successful, otherwise `None`.
pub(crate) async fn token_refresh(client: &BasicClient,
                                  refresh_token: &str,
                                  scopes: &[&str]) -> Option<Token> {
    let oauth2_refresh_token = RefreshToken::new(refresh_token.to_string());
    let token = client
        .exchange_refresh_token(&oauth2_refresh_token)
        .request_async(async_http_client)
        .await;

    match token {
        Ok(token) => {
            let expires_in = token.expires_in().unwrap_or(Duration::from_secs(3600));
            let expires_at = OffsetDateTime::now_utc() + expires_in;

            let token_result = Token {
                application_id: client.client_id().to_string(),
                scopes: scopes.to_vec().iter().map(|scope| scope.to_string()).collect(),
                access_token: token.access_token().secret().to_owned(),
                refresh_token: Some(refresh_token.to_string()),
                expires_at
            };
            Some(token_result)
        },
        Err(e) => {
            error!("Token refresh failed: {:?}", e);
            None
        }
    }
}