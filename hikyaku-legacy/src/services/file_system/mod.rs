mod download;
mod upload;

use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;
use reqwest::Client;
use aws_sdk_s3::client::Client as S3Client;
use tokio::fs::File;
use tokio::sync::Mutex;
use crate::utils::credential::google_drive_credential::GoogleDriveTokens;

#[derive(Clone)]
pub enum FileSystemObject {
    AmazonS3 {
        clients: Vec<Arc<S3Client>>,
        bucket: Arc<String>,
        key: Arc<String>,
        file_size: Option<u64>,
        chunk_size: u64,
    },
    GoogleDrive {
        clients: Vec<Arc<Client>>,
        google_drive_token: Arc<GoogleDriveTokens>,
        queryable_file_or_parent_id: Arc<String>,
        not_exist_file_paths: Arc<Vec<String>>,
        upload_filename: Option<Arc<String>>,
        mime_type: Arc<String>,
        resumable_upload_url: Arc<Mutex<Option<String>>>,
        file_size: Option<u64>,
        chunk_size: u64,
    },
    Local {
        path: Arc<PathBuf>,
        file: Arc<Mutex<Option<File>>>,
        is_dir: bool,
        file_size: Option<u64>,
        concurrency: u16,
        chunk_size: u64,
    },
}

impl FileSystemObject {
    pub(crate) fn is_downloadable(&self) -> bool {
        match self {
            Self::AmazonS3 { file_size, .. } |
            Self::GoogleDrive { file_size, .. } |
            Self::Local { file_size, .. }=> {
                match file_size {
                    Some(_) => true,
                    None => false,
                }
            },
        }
    }

    pub(crate) fn chunk_size(&self) -> u64 {
        match self {
            Self::AmazonS3 { chunk_size, .. } |
            Self::GoogleDrive { chunk_size, .. } |
            Self::Local { chunk_size, .. }=> {
                *chunk_size
            },
        }
    }

    pub(crate) fn concurrency(&self) -> u16 {
        match self {
            Self::AmazonS3 {clients, ..} => clients.len() as u16,
            Self::GoogleDrive {clients, ..} => clients.len() as u16,
            Self::Local {concurrency, ..} => *concurrency,
        }
    }

    pub(crate) fn file_size(&self) -> Option<u64> {
        match self {
            Self::AmazonS3 {file_size, ..} |
            Self::GoogleDrive {file_size, ..} |
            Self::Local {file_size, ..} => file_size.clone(),
        }
    }

    pub fn set_chunk_size(&mut self, size: u64) {
        match self {
            Self::AmazonS3 {chunk_size, ..} |
            Self::GoogleDrive {chunk_size, ..} |
            Self::Local {chunk_size, ..} => {
                *chunk_size = size;
            }
        }
    }
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
            Self::Local {path, file_size, ..} => {
                write!(f, "Local: path: {}, file_size: {:?}", path.display(), file_size)
            }
        }
    }
}

pub struct ChunkData {
    data: Vec<u8>,
    offset: u64,
    is_last: bool,
}

impl ChunkData {
    pub fn new(data: Vec<u8>,
               offset: u64,
               is_last: bool) -> Self {
        Self {
            data,
            offset,
            is_last,
        }
    }

    pub(crate) fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub(crate) fn get_offset(&self) -> u64 {
        self.offset
    }

    pub(crate) fn is_last(&self) -> bool {
        self.is_last
    }

    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }
    
    pub(crate) fn get_raw_data(self) -> Vec<u8> {
        self.data
    }
}
