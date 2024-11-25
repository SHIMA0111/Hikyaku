use std::cell::RefCell;
use std::num::NonZero;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::thread::available_parallelism;
use aws_config::{BehaviorVersion};
use reqwest::Client;
use aws_sdk_s3::client::Client as S3Client;
use log::error;
use reqwest::header::AUTHORIZATION;
use crate::errors::HikyakuError::{BuilderError, ConnectionError, GoogleDriveError, InvalidArgumentError};
use crate::errors::HikyakuResult;
use crate::types::google_drive::{DriveFileInfo, DriveFileQueryResponse, SharedDriveQueryResponse};
use crate::utils::credential::Credential;
use crate::utils::credential::google_drive_credential::{GoogleDriveCredential, GoogleDriveTokens};
use crate::utils::credential::s3_credential::S3Credential;
use crate::utils::file_type::FileType;
use crate::utils::parser::{file_system_prefix_parser, FileSystemParseResult};

pub struct FileSystemBuilder<C: Credential> {
    file_info: RefCell<Option<FileSystemParseResult>>,
    file_system_credential: C,
    concurrency: RefCell<u16>,
}

impl<C: Credential> FileSystemBuilder<C> {
    fn new(file_system_credential: C) -> Self {
        let parallelism = available_parallelism()
            // SAFETY: NonZero is always Some if the input is not `0`
            .unwrap_or(NonZero::new(1).unwrap())
            .get() * 2;
        let concurrency = RefCell::new(if parallelism > u16::MAX as usize {
            u16::MAX
        } else {
            parallelism as u16
        });
        
        Self {
            file_info: RefCell::new(None),
            file_system_credential,
            concurrency,
        }
    }
    
    pub async fn add_file_path(&self, path: &str) -> HikyakuResult<&Self> {
        let parse_res = file_system_prefix_parser(path)?;
        *self.file_info.borrow_mut() = Some(parse_res);
        
        Ok(self)
    }
    
    pub fn concurrency(&self, concurrency: NonZero<u16>) -> &Self {
        *self.concurrency.borrow_mut() = concurrency.get();
        self
    }
}

impl FileSystemBuilder<S3Credential> {
    pub async fn build(self) -> HikyakuResult<FileSystemObject> {
        let (bucket, key) = match self.file_info.borrow().as_ref() {
            Some(file_info) => {
                if file_info.get_prefix() != "s3://" {
                    return Err(InvalidArgumentError("File system prefix is not s3://".to_string()));
                }
                let bucket = file_info.get_namespace()
                    .ok_or(BuilderError("Bucket name cannot found".to_string()))?
                    .to_string();
                
                (bucket, file_info.get_path().to_string())
            },
            None => {
                return Err(BuilderError("Path is not set".to_string()));
            }
        };

        let file_system_credential = self.file_system_credential;

        let shared_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .region(file_system_credential.get_region())
            .credentials_provider(file_system_credential.get_credential())
            .load()
            .await;
        let concurrency = self.concurrency.borrow().to_owned();
        let clients = (0..concurrency)
            .map(|_| Arc::new(S3Client::new(&shared_config)))
            .collect::<Vec<_>>();
        let file_obj = FileSystemObject::AmazonS3 {
            clients,
            bucket,
            key,
            file_size: RefCell::new(None),
        };
        todo!()
    }
}

impl FileSystemBuilder<GoogleDriveCredential> {
    pub async fn build(self) -> HikyakuResult<FileSystemObject> {
        let (shared_drive_name, path) = match self.file_info.borrow().as_ref() {
            Some(info) => {
                if !["gd://", "gds://"].contains(&info.get_prefix()) {
                    return Err(InvalidArgumentError("File system prefix is not gd:// or gds".to_string()));
                }

                (info.get_namespace().map(String::from), info.get_path().to_string())
            },
            None => {
                return Err(BuilderError("Path is not set".to_string()));
            }
        };
    }

    async fn get_file_id_and_mime_type(&self, shared_drive_name: &str) -> HikyakuResult<(String, FileType)> {
        let client = Client::new();
        let response = client
            .get("https://www.googleapis.com/drive/v3/drives")
            .query(&[("q", format!("name = '{}'", shared_drive_name))])
            .header(AUTHORIZATION, format!("Bearer {}", self.file_system_credential.get_credential().get_access_token()))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to Google Drive API: {:#?}", e);
                ConnectionError(format!("Failed to send request to Google Drive API: {:?}", e))
            })?;

        let shared_drive_ids = response
            .json::<SharedDriveQueryResponse>()
            .await
            .map_err(|e| GoogleDriveError(format!("Failed to parse response from Google Drive API: {:?}", e)))?;

        if shared_drive_ids.is_empty() {
            return Err(InvalidArgumentError(format!("Shared drive name: '{}' is not found", shared_drive_name)));
        }

        let shared_drive_ids = shared_drive_ids
            .get_drives()
            .iter()
            .map(|shared_drive| shared_drive.id)
            .collect::<Vec<_>>();

        let query_file_id = |drive_ids: &[String], file_path: &str| -> HikyakuResult<Vec<DriveFileInfo>> {
            let components = Path::new(file_path)
                .components()
                // Component::CurDir never contains root dir due to the parse input, the first '/'(slash) was eliminated.
                .collect::<Vec<_>>();
            if components.iter().any(|component| matches!(component, Component::CurDir | Component::ParentDir)) {
                return Err(InvalidArgumentError(format!("File path cannot contain metacharacter to avoid ambiguous path. but got: {}", file_path)));
            }

            if components.iter().any(|component| component.as_os_str().to_str().is_none()) {
                return Err(InvalidArgumentError(format!("File path cannot contain non-ASCII character. but got: {}", file_path)));
            }

            let path_names = components
                .iter()
                // SAFETY: The components always can convert to String by the above validation.
                .map(|component| component.as_os_str().to_str().unwrap().to_string())
                .collect::<Vec<_>>();

            let last_file_info: Option<DriveFileQueryResponse> =
        }
    }
}

impl From<S3Credential> for FileSystemBuilder<S3Credential> {
    fn from(value: S3Credential) -> Self {
        Self::new(value)
    }
}

impl From<GoogleDriveCredential> for FileSystemBuilder<GoogleDriveCredential> {
    fn from(value: GoogleDriveCredential) -> Self {
        Self::new(value)
    }
}


pub enum FileSystemObject {
    AmazonS3 {
        clients: Vec<Arc<S3Client>>,
        bucket: String,
        key: String,
        file_size: RefCell<Option<u64>>,
    },
    GoogleDrive {
        clients: Vec<Arc<Client>>,
        google_drive_token: Arc<GoogleDriveTokens>,
        queryable_file_or_parent_id: String,
        not_exist_file_paths: Vec<String>,
        upload_filename: Option<String>,
        mime_type: String,
        file_size: RefCell<Option<u64>>,
    },
    Local {
        path: PathBuf,
        file_size: RefCell<Option<u64>>
    },
}

impl FileSystemObject {
    async fn init_file_size(&self) {
        match self {
            FileSystemObject::AmazonS3 {
                clients, 
                bucket, 
                key, 
                file_size, ..} => {
                
            },
            FileSystemObject::GoogleDrive {
                clients, 
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
                            *file_size.borrow_mut() = Some(metadata.len())
                        }
                        else {
                            // If the path is directory, it shouldn't have the length. It is nonsense.
                            *file_size.borrow_mut() = None;
                        }
                    },
                    Err(_) => {
                        *file_size.borrow_mut() = None;
                    }
                }
            }
        }
    }
}

