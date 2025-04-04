use std::time::SystemTime;
use aws_config::meta::credentials::CredentialsProviderChain;
use aws_config::meta::region::{RegionProviderChain};
use aws_sdk_s3::config::{Credentials, ProvideCredentials};
use time::OffsetDateTime;
use crate::errors::HikyakuError::EnvCredentialError;
use crate::errors::HikyakuResult;
use crate::utils::credential::Credential;
use crate::utils::region::aws::AWSRegion;
use crate::utils::region::Region;

pub struct S3Credential<AR: Region = AWSRegion> {
    credential: Credentials,
    region: AR,
}

impl <AR: Region> S3Credential<AR> {
    pub fn new(access_key_id: &str, secret_access_key: &str, session_token: Option<&str>, expiration: Option<OffsetDateTime>, region: AR) -> Self {
        let expiration = expiration.map(SystemTime::from);
        let credential = Credentials::new(
            access_key_id, 
            secret_access_key, 
            session_token.map(|s| s.to_string()), 
            expiration, 
            "HikyakuCredential");
        
        Self {
            credential,
            region,
        }
    }
}

impl S3Credential {
    pub async fn from_env() -> HikyakuResult<Self> {
        let env_region = RegionProviderChain::default_provider()
            .region()
            .await
            // The environment setting file 
            .ok_or(EnvCredentialError("Failed to get region from environment".to_string()))?;
        let region = AWSRegion::try_from(env_region)?;

        let credential = CredentialsProviderChain::default_provider()
            .await
            .provide_credentials()
            .await
            .map_err(|e| EnvCredentialError(e.to_string()))?;

        Ok(S3Credential::<AWSRegion> {
            credential,
            region,
        })
    }
}

impl Credential for S3Credential {
    type CredentialType = Credentials;
    type RegionType = AWSRegion;

    fn get_credential(&self) -> Self::CredentialType {
        self.credential.clone()
    }

    fn get_region(&self) -> Self::RegionType {
        self.region
    }
}
