use reqwest::{Client, Method};
use reqwest::header::AUTHORIZATION;
use crate::services::API;
use crate::utils::errors::{HikyakuError, HikyakuResult};
use crate::utils::errors::HikyakuError::GoogleDriveError;
use crate::utils::oauth2::services::get_google_oauth2_secret;
use crate::utils::types::google_drive::GoogleDriveResponse;

pub struct GoogleDrive(API);

impl GoogleDrive {
    pub fn new(client_id: &str, client_secret: &str, redirect_uri: Option<&str>) -> HikyakuResult<Self> {
        let secret =
            get_google_oauth2_secret(client_id, client_secret, redirect_uri)?;

        let api = API::new(secret, "https://www.googleapis.com");
        Ok(Self(api))
    }

    pub async fn get_drive_list(&self, page_size: Option<u32>,
                                page_token: Option<&str>) -> HikyakuResult<GoogleDriveResponse> {
        let client = Client::new();
        let mut request_url = self.0.get_request_url("/drive/v3/drives", GoogleDriveError)?;
        if let Some(page_token) = page_token {
            request_url.query_pairs_mut().append_pair("pageToken", page_token);
        }
        let page_size = page_size.unwrap_or(20).to_string();
        request_url.query_pairs_mut().append_pair("pageSize", page_size.as_str());

        let token = self.0.access_token(&["https://www.googleapis.com/auth/drive"]).await?;

        let result = client
            .request(Method::GET, request_url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await.unwrap();

        result.json::<GoogleDriveResponse>().await
            .map_err(|e| HikyakuError::GoogleDriveError(e.to_string()))
    }

    pub async fn get_file_list(&self, page_size: Option<u32>,
                               page_token: Option<&str>,
                               drive_id: Option<&str>) -> HikyakuResult<GoogleDriveResponse> {

        let client = Client::new();
        let mut request_url =
            self.0.get_request_url("/drive/v3/files", GoogleDriveError)?;
        if let Some(drive_id) = drive_id {
            request_url.query_pairs_mut().append_pair("driveId", drive_id);
            request_url.query_pairs_mut().append_pair("includeItemsFromAllDrives", "true");
            request_url.query_pairs_mut().append_pair("supportsAllDrives", "true");
            request_url.query_pairs_mut().append_pair("corpora", "drive");
        }
        let page_size = page_size.unwrap_or(20).to_string();
        request_url.query_pairs_mut().append_pair("pageSize", page_size.as_str());
        if let Some(page_token) = page_token {
            request_url.query_pairs_mut().append_pair("pageToken", page_token);
        }

        let token = self.0.access_token(&["https://www.googleapis.com/auth/drive"]).await?;

        let result = client
            .request(Method::GET, request_url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await.unwrap();
        result.json::<GoogleDriveResponse>().await
            .map_err(|e| HikyakuError::GoogleDriveError(e.to_string()))
    }
}
