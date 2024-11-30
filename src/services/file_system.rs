use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;
use reqwest::Client;
use aws_sdk_s3::client::Client as S3Client;
use crate::utils::credential::google_drive_credential::GoogleDriveTokens;

#[derive(Clone)]
pub enum FileSystemObject {
    AmazonS3 {
        clients: Vec<Arc<S3Client>>,
        bucket: String,
        key: String,
        file_size: Option<u64>,
    },
    GoogleDrive {
        clients: Vec<Arc<Client>>,
        google_drive_token: Arc<GoogleDriveTokens>,
        queryable_file_or_parent_id: String,
        not_exist_file_paths: Vec<String>,
        upload_filename: Option<String>,
        mime_type: String,
        file_size: Option<u64>,
    },
    Local {
        path: PathBuf,
        file_size: Option<u64>
    },
}

impl Display for FileSystemObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AmazonS3 {bucket, key, file_size, ..} => {
                write!(f, "AmazonS3: bucket: {}, key: {}, file_size: {:?}", bucket, key, file_size)
            },
            Self::GoogleDrive {
                queryable_file_or_parent_id,
                not_exist_file_paths,
                upload_filename,
                mime_type,
                file_size, ..} => {
                write!(f, "GoogleDrive: queryable_file_or_parent_id: {}, not_exist_file_paths: {:?}, upload_filename: {:?}, mime_type: {}, file_size: {:?}", queryable_file_or_parent_id, not_exist_file_paths, upload_filename, mime_type, file_size)
            },
            Self::Local {path, file_size} => {
                write!(f, "Local: path: {}, file_size: {:?}", path.display(), file_size)
            }
        }
    }
}
