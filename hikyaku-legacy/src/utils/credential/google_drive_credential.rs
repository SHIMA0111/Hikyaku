use time::OffsetDateTime;
use crate::utils::credential::Credential;
use crate::utils::region::NoneRegion;

#[derive(Debug, Clone)]
pub struct GoogleDriveTokens {
    access_token: String,
    refresh_token: Option<String>,
    expires_at: OffsetDateTime,
}

impl GoogleDriveTokens {
    pub(crate) fn get_access_token(&self) -> &str {
        &self.access_token
    }
}

pub struct GoogleDriveCredential {
    credential: GoogleDriveTokens,
}

impl GoogleDriveCredential {
    pub fn new(access_token: &str, refresh_token: &str, expires_at: OffsetDateTime) -> Self {
        let credential = GoogleDriveTokens {
            access_token: access_token.to_string(),
            refresh_token: Some(refresh_token.to_string()),
            expires_at,
        };
        
        Self {
           credential, 
        }
    }
}

impl Credential for GoogleDriveCredential {
    type CredentialType = GoogleDriveTokens;
    type RegionType = NoneRegion;

    fn get_credential(&self) -> Self::CredentialType {
        self.credential.clone()
    }
    
    fn get_region(&self) -> Self::RegionType {
        NoneRegion
    }
}
