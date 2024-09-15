mod web_server;
pub(crate) mod drop_control;
pub mod provider;
mod url_parser;
mod stores;

use std::fmt::{Display, Formatter};
use std::path::Path;
use std::time::Duration;
use log::{debug, error, info, warn};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, RefreshToken, TokenResponse, TokenUrl};
use oauth2::basic::BasicClient;
use oauth2::reqwest::{async_http_client};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::utils::oauth2::provider::Oauth2Provider;
use crate::utils::oauth2::stores::{load_token, save_token};
use crate::utils::oauth2::url_parser::extract_protocol_hostname;
use crate::utils::oauth2::web_server::{spawn_webserver};

/// Application Client Secret data.
///
/// Work as container of the secret from oauth2.0 provider
pub struct SecretData {
    client_id: String,
    client_secret: String,
    auth_uri: String,
    token_uri: String,
    protocol: String,
    redirect_hostname: String,
    port: u16,
    provider: Oauth2Provider,
}

/// Token object generated from [`SecretData`]
///
/// Token has parts of token like scopes and access token, refresh token
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Token {
    scopes: Vec<String>,
    access_token: String,
    refresh_token: Option<String>,
    expires_at: OffsetDateTime,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token\nscope: [{}]\naccess_token: *****\nrefresh_token: *****\nexpires_at: '{}'",
               self.scopes.join(", "), self.expires_at.unix_timestamp())
    }
}

impl SecretData {
    pub fn new(client_id: &str,
               client_secret: &str,
               auth_uri: &str,
               token_uri: &str,
               server_base_uri: Option<&str>,
               port: u16,
               provider: Oauth2Provider) -> Self {
        let (protocol, hostname) = extract_protocol_hostname(server_base_uri.unwrap_or("localhost"))
            .unwrap_or_else(|e| {
                error!("Failed to extract server base uri: {}", e);
                warn!("Using default server base uri: localhost");
                ("http".to_string(), "localhost".to_string())
            });

        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            auth_uri: auth_uri.to_string(),
            token_uri: token_uri.to_string(),
            protocol,
            redirect_hostname: hostname,
            port,
            provider,
        }
    }

    /// Return access token as [`Some(String)`] if it exists.
    ///
    /// If a user doesn't authenticate this app, returns [`None`]
    pub async fn get_access_token(&self, scopes: &[&str], token_path: &Path) -> Option<String> {
        let token_info = match load_token(self.provider, token_path) {
            Some(token_info) => {
                if token_info.expires_at > OffsetDateTime::now_utc() && scopes == token_info.scopes {
                    debug!("Token found: {}", token_info);
                    return Some(token_info.access_token)
                }
                else if scopes != token_info.scopes {
                    warn!("Token scopes mismatch. Re-authentication required.");
                    None
                }
                else {
                    warn!("Token was expired. Try to refresh token");
                    Some(token_info)
                }
            }
            None => None
        };

        let redirect_uri =
            format!("{}://{}:{}/auth/callback", self.protocol, self.redirect_hostname, self.port);

        let client = BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            AuthUrl::new(self.auth_uri.clone()).unwrap(),
            Some(TokenUrl::new(self.token_uri.clone()).unwrap()),
        )
            .set_auth_type(self.provider.auth_type())
            .set_redirect_uri(RedirectUrl::new(redirect_uri).unwrap());

        if let Some(Some(refresh_token)) = token_info.map(|token| token.refresh_token) {
            debug!("Refresh token is found");
            if let Some(new_token) = token_refresh(&client, &refresh_token, scopes).await {
                if new_token.scopes == scopes {
                    info!("Refresh the access token completed normally:\n{}", new_token);
                    if let Err(e) = save_token(self.provider, &new_token, token_path) {
                        error!("Failed to save token. This token isn't stored. (error: {:?})", e);
                    }
                    return Some(new_token.access_token);
                }
            };
        }

        let (sender, mut receiver) =
            tokio::sync::mpsc::channel::<Token>(1);
        spawn_webserver(
            &client,
            scopes,
            self.protocol.as_str(),
            self.redirect_hostname.as_str(),
            self.port,
            sender).await;

        match receiver.recv().await {
            Some(token_data) => {
                debug!("Get token:\n{}", token_data);
                save_token(self.provider, &token_data, token_path).unwrap();
                Some(token_data.access_token.to_string())
            }
            None => None
        }
    }
}

/// Token refresh using refresh token from the generated token object.
///
/// This function is used by automatically in [`SecretData::get_access_token`] so
/// this function is in private.
async fn token_refresh(client: &BasicClient,
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
