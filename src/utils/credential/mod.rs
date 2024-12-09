use crate::utils::region::{NoneRegion, Region};

pub mod s3_credential;
pub mod google_drive_credential;

pub trait Credential {
    type CredentialType;
    type RegionType: Region;
    
    fn get_credential(&self) -> Self::CredentialType;
    fn get_region(&self) -> Self::RegionType;
}

pub(crate) struct NoCredential;

impl Credential for NoCredential {
    type CredentialType = ();
    type RegionType = NoneRegion;

    fn get_credential(&self) -> Self::CredentialType {
        ()
    }

    fn get_region(&self) -> Self::RegionType {
        NoneRegion
    }
}
