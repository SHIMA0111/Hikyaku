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
use crate::errors::HikyakuError::{BuilderError, ConnectionError, GoogleDriveError, InvalidArgumentError, UnknownError};
use crate::errors::HikyakuResult;
use crate::types::google_drive::{DriveFileInfo, DriveFileQueryResponse, GoogleDriveFile, SharedDriveQueryResponse};
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

    async fn resolve_path_to_existing_depth(&self, shared_drive_name: Option<&str>, path: &str) -> HikyakuResult<(Option<GoogleDriveFile>, Vec<String>)> {
        let client = Client::new();

        let shared_drive_ids = if let Some(shared_drive_name) = shared_drive_name {
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

            shared_drive_ids
                .get_drives()
                .iter()
                .map(|shared_drive| shared_drive.id)
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        let components = Path::new(path)
            .components()
            // Component::CurDir never contains root dir due to the parse input, the first '/'(slash) was eliminated.
            .collect::<Vec<_>>();
        if components.iter().any(|component| matches!(component, Component::CurDir | Component::ParentDir)) {
            return Err(InvalidArgumentError(format!("File path cannot contain metacharacter to avoid ambiguous path. but got: {}", path)));
        }

        if components.iter().any(|component| component.as_os_str().to_str().is_none()) {
            return Err(InvalidArgumentError(format!("File path cannot contain non-ASCII character. but got: {}", path)));
        }

        let path_names = components
            .iter()
            // SAFETY: The components always can convert to String by the above validation.
            .map(|component| component.as_os_str().to_str().unwrap().to_string())
            .collect::<Vec<_>>();
        let mut complete_explore_path = vec![];

        let mut parent_infos = shared_drive_ids
            .iter()
            .map(|id| GoogleDriveFile::new(id, "", None))
            .collect::<Vec<_>>();
        for name in &path_names {
            complete_explore_path.push(name.clone());
            let mut query = format!("name = '{}'", name);
            for parent_info in &parent_infos {
                query.push_str(&format!(" and '{}' in parents", parent_info.get_id()));
            }

            let response = client
                .get("https://www.googleapis.com/drive/v3/files")
                .header(AUTHORIZATION, format!("Bearer {}", self.file_system_credential.get_credential().get_access_token()))
                .query(&[
                    ("q", &query),
                    ("supportsAllDrives", &"true".to_string()),
                    ("includeItemsFromAllDrives", &"true".to_string()),
                    ("fields", &"files(id, mimeType, size)".to_string()),
                ])
                .send()
                .await
                .map_err(|e| {
                    error!("Failed to send request to Google Drive API: {:#?}", e);
                    ConnectionError(format!("Failed to send request to Google Drive API: {:?}", e))
                })?;

            if !response.status().is_success() {
                error!("Failed to query files for Google Drive API: {}", response.status());
                return Err(ConnectionError(format!("Failed to query files for Google Drive API: {}", response.status())));
            }

            let query_response = response
                .json::<DriveFileQueryResponse>()
                .await
                .map_err(|e| UnknownError(format!("Failed to parse response from Google Drive API: {:#?}", e)))?;

            if query_response.is_empty() {
                break
            }
            else {
                parent_infos.clear();
                for file in query_response.files() {
                    let size = if let Some(size) = file.size() {
                        // Google Drive API returns the file size via JSON string. When it cannot parse to i64, it treats as -1 for handling.
                        if size < 0 {
                            return Err(GoogleDriveError("Google Drive returns invalid size information. If this issue occurs, please report to the author.".to_string()));
                        }

                        Some(size as u64)
                    } else {
                        None
                    };
                    parent_infos.push(GoogleDriveFile::new(&file.id, &file.mime_type, size))
                }
            }
        }
        if parent_infos.len() >= 2 {
            return Err(InvalidArgumentError(format!("File path '{}' is ambiguous. There is multiple files on the same path in Google Drive.", path)));
        }

        let res = if parent_infos.is_empty() {
            None
        } else {
            // SAFETY: The parent_infos has always only 1 content in this branch
            // because the length is not empty and parent_infos.len() < 2.
            Some(parent_infos.pop().unwrap())
        };
        let remain_path = path_names
            .iter()
            .filter(|name| !complete_explore_path.contains(name))
            .map(String::from)
            .collect::<Vec<_>>();

        Ok((res, remain_path))
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

