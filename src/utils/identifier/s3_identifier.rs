use crate::utils::identifier::{Credentials, FileSystemIdentifier};
use crate::utils::region::aws::AWSRegion;

pub struct S3Identifier {
    access_token: String,
    secret_token: String,
    region: AWSRegion,
}

impl S3Identifier {
    pub fn new(access_token: &str, secret_token: &str, region: AWSRegion) -> Self {
        Self {
            access_token: access_token.to_string(),
            secret_token: secret_token.to_string(),
            region,
        }
    }
}

impl FileSystemIdentifier<AWSRegion> for S3Identifier {
    fn get_access_token(&self) -> Credentials<AWSRegion> {
        Credentials::new(
            &self.access_token,
            Some(&self.secret_token),
            Some(self.region)
        )
    }
}
