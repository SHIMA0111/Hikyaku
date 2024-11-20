use std::path::{Path, PathBuf};
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
        file_size: Mutex<Option<u64>>,
    },
    GoogleDrive {
        client: Vec<Arc<Client>>,
        file_or_parent_id: String,
        upload_filename: Option<String>,
        mime_type: String,
        file_size: Mutex<Option<u64>>,
    },
    Local {
        path: PathBuf,
        file_size: Mutex<Option<u64>>
    },
}

impl FileSystemObject {
    async fn init_file_size(&self) {
        match self {
            FileSystemObject::AmazonS3 {
                client, 
                bucket, 
                key, 
                file_size, ..} => {
                
            },
            FileSystemObject::GoogleDrive {
                client, 
                file_or_parent_id, 
                file_size, ..} => {},
            FileSystemObject::Local {
                path, 
                file_size, ..} => {
                let path = path.as_path();
                let metadata = path.metadata();
                
                match metadata {
                    Ok(metadata) => {
                        if metadata.is_file() {
                            *file_size.lock().await = Some(metadata.len())
                        }
                        else {
                            // If the path is directory, it shouldn't have the length. It is nonsense.
                            *file_size.lock().await = None;
                        }
                    },
                    Err(_) => {
                        *file_size.lock().await = None;
                    }
                }
            }
        }
    }
}

