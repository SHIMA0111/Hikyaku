use std::io::SeekFrom;
use async_trait::async_trait;
use log::{error, warn};
use reqwest::header::CONTENT_TYPE;
use serde_json::json;
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::mpsc::Receiver;
use crate::errors::HikyakuError::{FileOperationError, GoogleDriveError, InvalidArgumentError, UnknownError};
use crate::errors::HikyakuResult;
use crate::services::file_system::{ChunkData, FileSystemObject};
use crate::types::google_drive::FileId;
use crate::utils::reqwest::AuthType::Bearer;
use crate::utils::reqwest::get_client_with_token;

#[async_trait]
pub trait Upload {
    async fn upload(&self, receiver: Receiver<ChunkData>) -> HikyakuResult<()>;
}

#[async_trait]
impl Upload for FileSystemObject {
    async fn upload(&self, receiver: Receiver<ChunkData>) -> HikyakuResult<()> {
        todo!()
    }
}

impl FileSystemObject {
    async fn partial_upload(&self, chunk_data: ChunkData) -> HikyakuResult<()> {
        if !chunk_data.is_last && self.chunk_size() != chunk_data.len() as u64 {
            return Err(UnknownError(
                "The chunk size is not equal to the length of the chunk data".to_string()));
        }

        if self.is_downloadable() {
            match self {
                Self::AmazonS3 {..} | Self::GoogleDrive {..} => {
                    warn!("The same name file is already exist. Please caution.");
                }
                Self::Local {..} => {
                    error!("The same name file is already exist. Please rename it.");
                    return Err(InvalidArgumentError(
                        "The same name file is already exist. Please rename it.".to_string()))
                }
            }
        }

        match self {
            Self::AmazonS3 {
                clients,
                bucket,
                key, ..} => {
                Ok(())
            },
            Self::GoogleDrive {
                clients,
                google_drive_token,
                queryable_file_or_parent_id,
                not_exist_file_paths,
                upload_filename,
                resumable_upload_url,
                ..} => {
                if upload_filename.is_none() {
                    return Err(InvalidArgumentError(
                        "The upload filename is not specified".to_string()));
                }

                let start = chunk_data.offset * self.chunk_size();
                let end = start + chunk_data.len() as u64 - 1;

                let mut resumable_lock = resumable_upload_url.lock().await;
                if resumable_lock.is_none() {
                    let parent_dir_id = if not_exist_file_paths.is_empty() {
                        queryable_file_or_parent_id.to_string()
                    } else {
                        let mut parent_id = if queryable_file_or_parent_id.is_empty() {
                            None
                        } else {
                            Some(queryable_file_or_parent_id.as_str().to_string())
                        };
                        for dir_name in not_exist_file_paths.iter() {
                            let created_parent_id = self.create_dir(dir_name, &parent_id).await?;
                            parent_id = Some(created_parent_id);
                        }

                        parent_id.unwrap_or("".to_string())
                    };

                    let url = "https://www.googleapis.com/upload/drive/v3/files?uploadType=resumable";
                    // TODO: Implement the infer mime_type
                    let mime_type = "application/octet-stream";
                    // SAFETY: The upload_filename is always Some because the None was filtered.
                    let filename = upload_filename.clone().unwrap();

                    let mut metadata = json!({
                        "name": filename.as_str(),
                        "mimeType": mime_type
                    });

                    if !parent_dir_id.is_empty() {
                        metadata["parents"] = json!([parent_dir_id]);
                    }

                    let client = get_client_with_token(google_drive_token.get_access_token(), Bearer)?;
                    let response = client
                        .post(url)
                        .header(CONTENT_TYPE, "application/json")
                        .json(&metadata)
                        .query(&[("supportsAllDrives", "true")])
                        .send()
                        .await
                        .map_err(|e| {
                            GoogleDriveError(format!("Failed to send request to get resumable URL for {}: {:?}", filename, e))
                        })?;

                    if !response.status().is_success() {
                        return Err(GoogleDriveError(format!("Failed to get resumable URL for {}: {:?}", filename, response.status())));
                    }

                    let resumable_url = response
                        .headers()
                        .get("Location")
                        .ok_or_else(|| {
                            GoogleDriveError(format!("Failed to get resumable URL for {}: {}", filename, "Location header is not found"))
                        })?
                        .to_str()
                        .map_err(|e| {
                            GoogleDriveError(format!("Failed to convert resumable URL for {}: {:?}", filename, e))
                        })?
                        .to_string();

                    *resumable_lock = Some(resumable_url);
                }

                let resumable_url = resumable_lock.as_ref().unwrap();
                Ok(())
            },
            Self::Local {path, file, ..} => {
                let data = chunk_data.get_data();
                let offset = chunk_data.offset;

                let start = offset * self.chunk_size();

                let mut file_lock = file.lock().await;
                if file_lock.is_none() {
                    let f = File::create(path.as_path()).await
                        .map_err(|e| {
                            FileOperationError(format!("Failed to create file to {}: {:?}", path.display(), e))
                        })?;
                    *file_lock = Some(f);
                }
                let file = file_lock.as_mut().unwrap();

                file.seek(SeekFrom::Start(start))
                    .await
                    .map_err(|e| {
                        FileOperationError(format!("Failed to seek file {}: {:?}", start, e))
                    })?;
                file.write_all(data)
                    .await
                    .map_err(|e| {
                        FileOperationError(format!("Failed to write file {}: {:?}", start, e))
                    })?;

                drop(file_lock);

                Ok(())
            },
        }
    }

    async fn create_dir(&self, dir_name: &str, parent_id: &Option<String>) -> HikyakuResult<String> {
        if let Self::GoogleDrive {google_drive_token, ..} = self {
            let access_token = google_drive_token.get_access_token();
            // TODO: We should check if the client should create newly or use generated client from performance.
            let client = get_client_with_token(access_token, Bearer)?;
            let mut metadata = json!({
                "name": dir_name,
                "mimeType": "application/vnd.google-apps.folder",
            });

            if let Some(parent_id) = parent_id {
                metadata["parents"] = json!([parent_id.to_string()]);
            }

            let response = client
                .post("https://www.googleapis.com/drive/v3/files")
                .header("Content-Type", "application/json")
                .json(&metadata)
                .query(&[("supportsAllDrives", "true")])
                .send()
                .await
                .map_err(|e| {
                    GoogleDriveError(format!("Failed to send request to create directory {}: {:?}", dir_name, e))
                })?;

            if !response.status().is_success() {
                let parent_id = parent_id.as_ref().map_or("RootDir", |id| id.as_str());
                return Err(GoogleDriveError(format!("Failed to create directory {} in {}: {:?}", dir_name, parent_id, response.status())));
            }

            let file_info = response
                .json::<FileId>()
                .await
                .map_err(|e| {
                    GoogleDriveError(format!("Failed to parse response to id from create directory {}: {:?}", dir_name, e))
                })?;

            Ok(file_info.get_id())
        }
        else {
            unreachable!();
        }

    }
}
