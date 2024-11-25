use crate::utils::region::Region;

pub mod s3_credential;
pub mod google_drive_credential;

pub trait Credential {
    type CredentialType;
    type RegionType: Region;
    
    fn get_credential(&self) -> Self::CredentialType;
    fn get_region(&self) -> Self::RegionType;
}