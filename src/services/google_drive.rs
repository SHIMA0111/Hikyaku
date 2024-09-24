use log::error;
use reqwest::{Client, Method};
use reqwest::header::AUTHORIZATION;
use serde::__private::from_utf8_lossy;
use crate::services::API;
use crate::utils::errors::HikyakuError::GoogleDriveError;
use crate::utils::errors::HikyakuResult;
use crate::utils::oauth2::services::get_google_oauth2_secret;
use crate::utils::types::GoogleDriveResponse;

pub struct GoogleDrive(API);

impl GoogleDrive {
    pub fn new(client_id: &str, client_secret: &str, redirect_uri: Option<&str>) -> HikyakuResult<Self> {
        let secret =
            get_google_oauth2_secret(client_id, client_secret, redirect_uri)?;

        let api = API::new(secret, "https://www.googleapis.com");
        Ok(Self(api))
    }

    pub async fn get_drive_list(&self) -> HikyakuResult<()> {
        let client = Client::new();
        let api_endpoint = self.0.generate_endpoint("/drive/v3/drives");
        let request = match reqwest::Url::parse(&api_endpoint) {
            Ok(uri) => uri,
            Err(e) => {
                error!("Failed to parse endpoint: {:?}", e);
                return Err(GoogleDriveError("Failed to parse endpoint for get_drive_list".to_string()));
            }
        };
        let token = self.0.access_token(&["https://www.googleapis.com/auth/drive"]).await?;

        let result = client
            .request(Method::GET, request)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await.unwrap();

        let response_body = result.json::<GoogleDriveResponse>().await;
        if let Err(e) = response_body {
            eprintln!("{:?}", e);
        } else {
            println!("{:?}", response_body);
        }
        Ok(())
    }
}
