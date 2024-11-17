use time::OffsetDateTime;
use crate::utils::identifier::{Credentials, FileSystemIdentifier, NoneRegion};

pub struct GoogleDriveIdentifier {
    access_token: String,
    refresh_token: Option<String>,
    expires_at: u64,
}

impl GoogleDriveIdentifier {
    fn new(access_token: &str, refresh_token: Option<&str>, expires_at: u64) -> Self {
        Self {
            access_token: access_token.to_string(),
            refresh_token: refresh_token.map(|s| s.to_string()),
            expires_at,
        }
    }
}

impl FileSystemIdentifier for GoogleDriveIdentifier {
    fn get_access_token(&self) -> Credentials<NoneRegion> {
        Credentials::new(
            &self.access_token,
            None,
            None
        )
    }

    fn get_refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }

    fn is_expired(&self) -> bool {
        // SAFETY: now_utc() returns the current UTC timestamp 
        // so it is always a positive value. Please caution if you live before 1970-01-01T00:00. 
        self.expires_at < OffsetDateTime::now_utc().unix_timestamp() as u64
    }
}