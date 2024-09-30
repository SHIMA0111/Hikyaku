use std::path::{Path, PathBuf};
use log::error;
use url::Url;
use crate::utils::errors::{HikyakuError, HikyakuResult};
use crate::utils::oauth2::SecretData;

pub mod s3;
pub mod google_drive;
pub mod r#box;
pub mod one_drive;

pub trait Service {
    fn new(client_id: &str, client_secret: &str, redirect_uri: Option<&str>) -> HikyakuResult<Self> where Self: Sized;
}

pub(crate) struct API {
    secret_data: SecretData,
    api_base_uri: String,
}

impl API {
    pub(crate) fn new(secret_data: SecretData, api_base_uri: &str) -> Self {
        let api_base_uri = if api_base_uri.ends_with("/") {
            let (base_uri, _) = api_base_uri.rsplit_once("/").unwrap();
            base_uri.to_string()
        } else {
            api_base_uri.to_string()
        };

        Self {
            secret_data,
            api_base_uri,
        }
    }

    pub(crate) async fn access_token(&self, scopes: &[&str]) -> HikyakuResult<String> {
        let home_dir = get_home_dir();
        match self.secret_data.get_access_token(scopes, home_dir).await {
            Some(access_token) => Ok(access_token),
            None => Err(HikyakuError::OAuth2Error("Faile to get access token".to_string())),
        }
    }

    pub(crate) fn generate_endpoint(&self, endpoint: &str) -> String {
        let endpoint = if endpoint.starts_with("/") {
            endpoint.to_string()
        } else {
            format!("/{}", endpoint)
        };

        format!("{}{}", self.api_base_uri, endpoint)
    }

    pub(crate) fn get_request_url<EF>(&self, endpoint: &str, error_fn: EF) -> HikyakuResult<Url>
    where
        EF: Fn(String) -> HikyakuError,
    {
        let api_endpoint = self.generate_endpoint(endpoint);
        match Url::parse(&api_endpoint) {
            Ok(uri) => Ok(uri),
            Err(e) => {
                error!("Failed to parse endpoint: {:?}", e);
                Err(error_fn("Failed to parse endpoint for get_drive_list".to_string()))
            }
        }
    }
}

fn get_home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or(PathBuf::from("."))
}
