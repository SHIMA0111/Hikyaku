use std::path::Path;
use std::sync::Arc;
use reqwest::Client;
use tokio::sync::Mutex;
use aws_sdk_s3::client::Client as S3Client;
use crate::errors::HikyakuResult;
use crate::utils::identifier::FileSystemIdentifier;

pub struct FileSystemBuilder<ID: FileSystemIdentifier> {
    file_path: String,
    file_system_identifier: ID,
}

impl<ID: FileSystemIdentifier> FileSystemBuilder<ID> {
    
}


pub enum FileSystemObject {
    AmazonS3 {
        client: Vec<Arc<S3Client>>,
        bucket: String,
        key: String,
        file_size: Mutex<u64>,
    },
    GoogleDrive {
        client: Vec<Arc<Client>>,
        file_or_parent_id: String,
        upload_filename: Option<String>,
        mime_type: String,
        file_size: Mutex<u64>,
    },
    Local {
        path: Box<dyn AsRef<Path>>,
    },
}

