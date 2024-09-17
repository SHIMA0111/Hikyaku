use std::fs::File;
use std::path::Path;
use log::error;
use serde::Deserialize;
use crate::utils::errors::{HikyakuError, HikyakuResult};
use crate::utils::oauth2::provider::Oauth2Provider::{Google, Box, Dropbox, Microsoft};
use crate::utils::oauth2::SecretData;
use crate::utils::oauth2::url_parser::parse_url;

#[derive(Deserialize)]
pub(crate) struct GoogleOauth2 {
    installed: Option<GoogleOauth2Secret>,
    web: Option<GoogleOauth2Secret>,
}

#[derive(Deserialize)]
pub(crate) struct GoogleOauth2Secret {
    client_id: String,
    client_secret: String,
    auth_uri: String,
    token_uri: String,
    redirect_uris: Vec<String>,
}


/// Loads the Google OAuth2 secret data from a JSON file specified by the path.
///
/// # Arguments
///
/// * `secret_json_path` - A path to the JSON file containing the Google OAuth2 secret data.
///
/// # Returns
///
/// A `HikyakuResult` which is either:
///
/// - `Ok(SecretData)` containing the loaded secret data.
/// - `Err(HikyakuError)` with a message describing the error that occurred.
pub fn load_google_oauth2_secret<SP: AsRef<Path>>(secret_json_path: SP) -> HikyakuResult<SecretData> {
    let secret_data = match File::open(&secret_json_path) {
        Ok(file) => match serde_json::from_reader::<_, GoogleOauth2>(&file) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to parse secret file: {:?}", e);
                return Err(HikyakuError::OAuth2Error(
                    format!("Invalid secret format in: {}",
                            secret_json_path.as_ref().to_string_lossy())));
            }
        },
        Err(e) => {
            error!("Failed to open secret file: {:?}", e);
            return Err(HikyakuError::OAuth2Error(
                format!("Cannot open secret file: {}",
                        secret_json_path.as_ref().to_string_lossy())));
        }
    };

    let generate_secret_data = |secret: GoogleOauth2Secret| -> HikyakuResult<SecretData> {
        if secret.redirect_uris.is_empty() || secret.redirect_uris.len() >= 2 {
            return Err(HikyakuError::OAuth2Error(
                format!("'redirect_uri' needs to be unique but found {} uri(s)",
                        secret.redirect_uris.len())))
        }
        let redirect_uri = &secret.redirect_uris[0];
        let (redirect_base_url, port) = parse_url(redirect_uri)?;
        Ok(SecretData::new(
            &secret.client_id,
            &secret.client_secret,
            &secret.auth_uri,
            &secret.token_uri,
            Some(redirect_base_url.as_str()),
            port,
            Google
        ))
    };

    let secret_data = if let Some(secret) = secret_data.installed {
        generate_secret_data(secret)?
    } else if let Some(secret) = secret_data.web {
        generate_secret_data(secret)?
    } else {
        return Err(HikyakuError::OAuth2Error("JSON format is invalid".to_string()));
    };

    Ok(secret_data)
}


///
/// Creates a `SecretData` instance for Google OAuth2 using provided client credentials and an optional redirect URI.
///
/// # Arguments
///
/// * `client_id` - A string slice that holds the client ID.
/// * `client_secret` - A string slice that holds the client secret.
/// * `redirect_uri` - An optional string slice that holds the redirect URI.
///
/// # Returns
///
/// A `HikyakuResult` which is either:
///
/// - `Ok(SecretData)` containing the created secret data.
/// - `Err(HikyakuError)` with a message describing the error that occurred.
///
/// # Errors
///
/// This function will return an error if the `redirect_uri` is invalid.
///
/// # Examples
///
/// ```
/// use hikyaku::utils::oauth2::services::get_google_oauth2_secret;
/// 
/// let client_id = "your-client-id";
/// let client_secret = "your-client-secret";
/// let redirect_uri = Some("https://your-redirect-uri");
/// match get_google_oauth2_secret(client_id, client_secret, redirect_uri) {
///     Ok(secret_data) => {
///         // Use the secret data
///     }
///     Err(e) => {
///         eprintln!("Error: {:?}", e);
///     }
/// }
/// ```
pub fn get_google_oauth2_secret(client_id: &str, client_secret: &str, redirect_uri: Option<&str>) -> HikyakuResult<SecretData> {
    let (redirect_base_uri, port) = match redirect_uri {
        Some(uri) => {
            let (base_uri, port) = parse_url(uri)?;
            (Some(base_uri), port)
        },
        None => (None, 80)
    };

    let secret_data = SecretData::new(
        client_id,
        client_secret,
        "https://accounts.google.com/o/oauth2/auth",
        "https://oauth2.googleapis.com/token",
        redirect_base_uri.as_deref(),
        port,
        Box,
    );

    Ok(secret_data)
}

/// Creates a `SecretData` instance for Box OAuth2 using provided client credentials and an optional redirect URI.
///
/// # Arguments
///
/// * `client_id` - A string slice that holds the client ID.
/// * `client_secret` - A string slice that holds the client secret.
/// * `redirect_uri` - An optional string slice that holds the redirect URI.
///
/// # Returns
///
/// A `HikyakuResult` which is either:
///
/// - `Ok(SecretData)` containing the created secret data.
/// - `Err(HikyakuError)` with a message describing the error that occurred.
///
/// # Errors
///
/// This function will return an error if the `redirect_uri` is invalid.
///
/// # Examples
///
/// ```
/// use hikyaku::utils::oauth2::services::get_box_oauth2_secret;
///
/// let client_id = "your-client-id";
/// let client_secret = "your-client-secret";
/// let redirect_uri = Some("https://your-redirect-uri");
/// match get_box_oauth2_secret(client_id, client_secret, redirect_uri) {
///     Ok(secret_data) => {
///         // Use the secret data
///     }
///     Err(e) => {
///         eprintln!("Error: {:?}", e);
///     }
/// }
/// ```
pub fn get_box_oauth2_secret(client_id: &str, client_secret: &str, redirect_uri: Option<&str>) -> HikyakuResult<SecretData> {
    let (redirect_base_uri, port) = match redirect_uri {
        Some(uri) => {
            let (base_uri, port) = parse_url(uri)?;
            (Some(base_uri), port)
        },
        None => (None, 80)
    };

    let secret_data = SecretData::new(
        client_id,
        client_secret,
        "https://account.box.com/api/oauth2/authorize",
        "https://api.box.com/oauth2/token",
        redirect_base_uri.as_deref(),
        port,
        Box,
    );

    Ok(secret_data)
}


/// Creates a `SecretData` instance for Dropbox OAuth2 using provided client credentials and an optional redirect URI.
/// 
/// # Arguments
// 
/// * `client_id` - A string slice that holds the client ID.
/// * `client_secret` - A string slice that holds the client secret.
/// * `redirect_uri` - An optional string slice that holds the redirect URI.
/// 
/// # Returns
/// 
/// A `HikyakuResult` which is either:
/// 
/// - `Ok(SecretData)` containing the created secret data.
/// - `Err(HikyakuError)` with a message describing the error that occurred.
/// 
/// # Errors
/// 
/// This function will return an error if the `redirect_uri` is invalid.
///
/// # Examples
///
/// ```
/// use hikyaku::utils::oauth2::services::get_dropbox_oauth2_secret;
/// 
/// let client_id = "your-client-id";
/// let client_secret = "your-client-secret";
/// let redirect_uri = Some("https://your-redirect-uri");
/// match get_dropbox_oauth2_secret(client_id, client_secret, redirect_uri) {
///     Ok(secret_data) => {
///         // Use the secret data
///     }
///     Err(e) => {
///         eprintln!("Error: {:?}", e);
///     }
/// }
/// ```
pub fn get_dropbox_oauth2_secret(client_id: &str, client_secret: &str, redirect_uri: Option<&str>) -> HikyakuResult<SecretData> {
    let (redirect_base_uri, port) = match redirect_uri {
        Some(uri) => {
            let (base_uri, port) = parse_url(uri)?;
            (Some(base_uri), port)
        },
        None => (None, 80)
    };

    let secret_data = SecretData::new(
        client_id,
        client_secret,
        "https://www.dropbox.com/oauth2/authorize",
        "https://api.dropboxapi.com/oauth2/token",
        redirect_base_uri.as_deref(),
        port,
        Dropbox,
    );

    Ok(secret_data)
}


/// Represents the type of Microsoft tenant when creating OAuth2 secret data.
///
/// # Variants
///
/// * `SingleTenant` - This variant is used for a single-tenant application. It requires a `tenant_id` which is a static string slice identifying the tenant.
/// * `MultiTenant` - This variant is used for multi-tenant applications where the application can sign in users from multiple organizations.
///
/// # Examples
///
/// ```
/// use hikyaku::utils::oauth2::services::MicrosoftTenantType;
///
/// let single_tenant = MicrosoftTenantType::SingleTenant { tenant_id: "your-tenant-id" };
/// let multi_tenant = MicrosoftTenantType::MultiTenant;
/// ```
pub enum MicrosoftTenantType {
    SingleTenant{
        tenant_id: &'static str
    },
    MultiTenant,
}


/// Creates a `SecretData` instance for Microsoft OAuth2 using provided client credentials, redirect URI, and tenant type.
///
/// # Arguments
///
/// * `client_id` - A string slice that holds the client ID.
/// * `client_secret` - A string slice that holds the client secret.
/// * `redirect_uri` - An optional string slice that holds the redirect URI.
/// * `tenant_type` - An instance of [`MicrosoftTenantType`] representing the type of Microsoft tenant.
///
/// # Returns
///
/// A `HikyakuResult` which is either:
///
/// - `Ok(SecretData)` containing the created secret data.
/// - `Err(HikyakuError)` with a message describing the error that occurred.
///
/// # Errors
///
/// This function will return an error if the `redirect_uri` is invalid.
///
/// # Examples
///
/// ```
/// use hikyaku::utils::oauth2::services::{get_microsoft_oauth2_secret, MicrosoftTenantType};
///
/// let client_id = "your-client-id";
/// let client_secret = "your-client-secret";
/// let redirect_uri = Some("https://your-redirect-uri");
/// let tenant_type = MicrosoftTenantType::SingleTenant { tenant_id: "your-tenant-id" };
/// match get_microsoft_oauth2_secret(client_id, client_secret, redirect_uri, tenant_type) {
///     Ok(secret_data) => {
///         // Use the secret data
///     }
///     Err(e) => {
///         eprintln!("Error: {:?}", e);
///     }
/// }
/// ```
pub fn get_microsoft_oauth2_secret(client_id: &str,
                                   client_secret: &str,
                                   redirect_uri: Option<&str>,
                                   tenant_type: MicrosoftTenantType) -> HikyakuResult<SecretData> {
    let (auth_uri, token_uri) = match tenant_type {
        MicrosoftTenantType::SingleTenant {tenant_id} => {
            let auth_uri =
                format!("https://login.microsoftonline.com/{}/oauth2/v2.0/authorize", tenant_id);
            let token_uri =
                format!("https://login.microsoftonline.com/{}/oauth2/v2.0/token", tenant_id);
            (auth_uri, token_uri)
        },
        MicrosoftTenantType::MultiTenant => {
            ("https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
             "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string())
        }
    };

    let (redirect_base_uri, port) = match redirect_uri {
        Some(uri) => {
            let (base_uri, port) = parse_url(uri)?;
            (Some(base_uri), port)
        },
        None => (None, 80)
    };


    let secret_data = SecretData::new(
        client_id,
        client_secret,
        &auth_uri,
        &token_uri,
        redirect_base_uri.as_deref(),
        port,
        Microsoft
    );

    Ok(secret_data)
}
