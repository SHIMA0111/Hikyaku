use crate::utils::region::Region;

mod s3_identifier;
mod google_drive_identifier;

pub struct Credentials<R>
where
    R: Region
{
    pub(crate) access_token_or_key: String,
    pub(crate) secret_access_key: Option<String>,
    pub(crate) region: Option<R>
}

impl <R> Credentials<R> 
where 
    R: Region
{
    pub(crate) fn new(access_token_or_key: &str, secret_access_key: Option<&str>, region: Option<R>) -> Self {
        Self {
            access_token_or_key: access_token_or_key.to_string(),
            secret_access_key: secret_access_key.map(|s| s.to_string()),
            region
        }
    }
}

pub struct NoneRegion;

impl Region for NoneRegion {
    fn get_region(&self) -> &str {
        ""
    }
}

pub trait FileSystemIdentifier<R = NoneRegion> 
where 
    R: Region
{
    fn get_access_token(&self) -> Credentials<R>;

    fn get_refresh_token(&self) -> Option<&str> {
        None
    }
    
    fn is_expired(&self) -> bool {
        false
    }
}