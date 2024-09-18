mod web_server;
pub(crate) mod drop_control;
pub mod provider;
mod url_parser;
mod stores;
mod token_refresh;
pub mod services;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::Path;
use log::{debug, error, info, warn};
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use oauth2::basic::BasicClient;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use crate::utils::oauth2::provider::Oauth2Provider;
use crate::utils::oauth2::stores::{load_token, save_token};
use crate::utils::oauth2::token_refresh::token_refresh;
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
    extra_args: HashMap<String, String>,
    protocol: String,
    redirect_hostname: String,
    port: u16,
    init_path: String,
    redirect_path: String,
    provider: Oauth2Provider,
}

/// Token object generated from [`SecretData`]
///
/// Token has parts of token like scopes and access token, refresh token
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Token {
    scopes: Vec<String>,
    application_id: String,
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
    pub(crate) fn new(client_id: &str,
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
            extra_args: HashMap::new(),
            protocol,
            redirect_hostname: hostname,
            port,
            init_path: "/auth/init".to_string(),
            redirect_path: "/auth/callback".to_string(),
            provider,
        }
    }

    /// Set the path for the initialization endpoint.
    ///
    /// This path will be used by the OAuth2 process to start the authentication process.
    ///
    /// # Arguments
    ///
    /// * `init_path` - A string slice that holds the path for the initialization endpoint.
    ///
    /// # Example
    ///
    /// ```
    /// use hikyaku::utils::oauth2::services::get_google_oauth2_secret;
    ///
    /// let mut secret_data = get_google_oauth2_secret(
    ///     "client_id",
    ///     "client_secret",
    ///     Some("https://example.com"),
    /// ).unwrap();
    ///
    /// secret_data.set_init_path("/new_init_path");
    /// ```
    pub fn set_init_path(&mut self, init_path: &str) {
        self.init_path = if init_path.starts_with("/") {
            init_path.to_string()
        } else {
            format!("/{}", init_path)
        }
    }

    /// Set the path for the redirect endpoint.
    ///
    /// This path will be used by the OAuth2 process to handle the callback after authentication.
    ///
    /// # Arguments
    ///
    /// * `callback_path` - A string slice that holds the path for the redirect endpoint.
    ///
    /// # Example
    ///
    /// ```
    /// use hikyaku::utils::oauth2::services::get_google_oauth2_secret;
    ///
    /// let mut secret_data = get_google_oauth2_secret(
    ///     "client_id",
    ///     "client_secret",
    ///     Some("https://example.com"),
    /// ).unwrap();
    ///
    /// secret_data.set_redirect_path("/new_callback_path");
    /// ```
    pub fn set_redirect_path(&mut self, callback_path: &str) {
        self.redirect_path = if callback_path.starts_with("/") {
            callback_path.to_string()
        } else {
            format!("/{}", callback_path)
        }
    }

    /// Add extra arguments for the authentication URL.
    ///
    /// These extra arguments will be appended to the authentication URL
    /// when generating the URL for OAuth2 authentication.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the name of the URL parameter.
    /// * `value` - A string slice that holds the value of the URL parameter.
    ///
    /// # Example
    ///
    /// ```
    /// use hikyaku::utils::oauth2::services::get_google_oauth2_secret;
    ///
    /// let mut secret_data = get_google_oauth2_secret(
    ///     "client_id",
    ///     "client_secret",
    ///     Some("https://example.com"),
    /// ).unwrap();
    ///
    /// secret_data.add_extra_args_for_auth_url("include_granted_scopes", "true");
    /// ```
    pub fn add_extra_args_for_auth_url(&mut self, key: &str, value: &str) {
        self.extra_args.insert(key.to_string(), value.to_string());
    }

    /// Return access token as [`Some(String)`] if it exists.
    ///
    /// # Arguments
    ///
    /// * `scopes` - A slice containing the scopes required for the access token.
    /// * `token_path` - The path where the token is stored.
    ///
    /// # Returns
    ///
    /// Returns [`Some(String)`] containing the access token if it exists and is valid.
    /// Returns [`None`] if the user doesn't authenticate the app.
    ///
    /// This function tries to load the token from the provided path. If the token is found and valid,
    /// it returns the access token. If the token is expired, it attempts to refresh the token using the
    /// refresh token. If the token scopes don't match, it requires re-authentication. If there is no token,
    /// it starts the OAuth2 flow to get a new token.
    pub async fn get_access_token<TP: AsRef<Path>>(&self, scopes: &[&str],
                                                   token_path: TP) -> Option<String> {
        let token_info = match load_token(self.provider.clone(), token_path.as_ref()) {
            Some(token_info) => {
                if token_info.expires_at > OffsetDateTime::now_utc() && scopes == token_info.scopes {
                    debug!("Token found: {}", token_info);
                    return Some(token_info.access_token)
                }
                else if scopes != token_info.scopes {
                    warn!("Token scopes mismatch. Re-authentication required.");
                    None
                }
                else if self.client_id != token_info.application_id {
                    warn!("Token application id mismatch. Re-authentication required.");
                    None
                }
                else {
                    warn!("Token was expired. Try to refresh token");
                    Some(token_info)
                }
            }
            None => None
        };

        let redirect_uri = if [443, 80].contains(&self.port) {
            format!("{}://{}{}", self.protocol, self.redirect_hostname, self.redirect_path)
        } else {
            format!("{}://{}:{}{}", self.protocol, self.redirect_hostname, self.port, self.redirect_path)
        };

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
                    if let Err(e) = save_token(self.provider.clone(), &new_token, token_path.as_ref()) {
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
            self.init_path.as_str(),
            self.redirect_path.as_str(),
            &self.extra_args,
            sender).await;

        match receiver.recv().await {
            Some(token_data) => {
                debug!("Get token:\n{}", token_data);
                save_token(self.provider.clone(), &token_data, token_path.as_ref()).unwrap();
                Some(token_data.access_token.to_string())
            }
            None => None
        }
    }
}
