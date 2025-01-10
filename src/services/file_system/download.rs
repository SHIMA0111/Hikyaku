use std::cmp::min;
use std::io::SeekFrom;
use std::sync::Arc;
use async_trait::async_trait;
use log::{debug, error};
use reqwest::header::{AUTHORIZATION, RANGE};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::sync::mpsc::Sender;
use crate::errors::HikyakuError::{ConnectionError, FileOperationError, GoogleDriveError, NotExistFileError, S3Error};
use crate::errors::HikyakuResult;
use crate::services::file_system::{ChunkData, FileSystemObject};

#[async_trait]
pub trait Download {
    async fn download(&self, sender: Sender<ChunkData>) -> HikyakuResult<()>;
}

#[async_trait]
impl Download for FileSystemObject {
    async fn download(&self, sender: Sender<ChunkData>) -> HikyakuResult<()> {
        if !self.is_downloadable() {
            return Err(NotExistFileError(format!("File system object is not downloadable. File system object: {}", self)));
        }

        let last_offset = (self.file_size().unwrap() + self.chunk_size() - 1) / self.chunk_size();

        let mut tasks = vec![];
        let arc_sender = Arc::new(sender);
        for offset in 0..last_offset {
            let arc_sender = Arc::clone(&arc_sender);
            let clone_me = self.clone();

            let task: tokio::task::JoinHandle<HikyakuResult<()>> = tokio::spawn(async move {
                let chunk_data = clone_me.partial_download(offset).await?;
                arc_sender.send(chunk_data).await.unwrap();
                Ok(())
            });
            tasks.push(task);
        }

        Ok(())
    }
}

impl FileSystemObject {
    async fn partial_download(&self, offset: u64) -> HikyakuResult<ChunkData> {
        let chunk_size = self.chunk_size();
        // SAFETY: This method called in download func and it guaranties the filesize is always Some.
        let file_size = self.file_size().unwrap();
        let start = offset * chunk_size;
        let end = min((offset + 1) * chunk_size - 1, file_size - 1);
        let is_last = end == file_size - 1;

        match self {
            Self::AmazonS3 {
                clients,
                bucket,
                key,
                ..
            } => {
                let client = clients[(self.concurrency() as u64 % offset) as usize].clone();

                let part = client
                    .get_object()
                    .bucket(bucket.as_str())
                    .key(key.as_str())
                    .range(format!("bytes={}-{}", start, end))
                    .send()
                    .await
                    .inspect(|obj| debug!("{:#?}", obj))
                    .map_err(|e| {
                        error!("Failed to request for getting object: {:#?}", e);
                        S3Error(format!("{:?}", e))
                    })?;

                let body = part
                    .body
                    .collect()
                    .await
                    .map_err(|e| {
                        error!("Failed to collect body: {:#?}", e);
                        S3Error(format!("{:?}", e))
                    })?;

                let bytes = body.to_vec();

                let bytes = if end == file_size - 1 {
                    bytes[..(end - start + 1) as usize].to_vec()
                } else {
                    bytes
                };

                Ok(ChunkData::new(bytes, offset, is_last))
            },
            Self::GoogleDrive {
                clients,
                google_drive_token,
                queryable_file_or_parent_id,
                ..
            } => {
                let client = clients[(self.concurrency() as u64 % offset) as usize].clone();
                let url = format!("https://www.googleapis.com/drive/v3/files/{}?alt=media", queryable_file_or_parent_id);

                let res = client
                    .get(url)
                    .header(AUTHORIZATION, format!("Bearer {}", google_drive_token.get_access_token()))
                    .header(RANGE, format!("bytes={}-{}", start, end))
                    .query(&[
                        ("supportAllDrives", "true")
                    ])
                    .send()
                    .await
                    .inspect(|obj| debug!("{:#?}", obj))
                    .map_err(|e| {
                        error!("Failed to request for Google Drive API: {:#?}", e);
                        ConnectionError(format!("Failed to send request to Google Drive API: {:?}", e))
                    })?;

                if !res.status().is_success() {
                    let status = res.status();
                    let body = res.text().await.unwrap_or_default();
                    let message = format!("Google Drive API returned status code: {}, body: {}", status, body);
                    return Err(ConnectionError(message));
                }

                let bytes = res
                    .bytes()
                    .await
                    .map_err(|e| {
                        error!("Failed to collect body: {:#?}", e);
                        GoogleDriveError(format!("{:?}", e))
                    })?
                    .to_vec();

                let bytes = if end == file_size - 1 {
                    bytes[..(end - start + 1) as usize].to_vec()
                } else {
                    bytes
                };

                Ok(ChunkData::new(bytes, offset, is_last))
            },
            Self::Local {
                path,
                file,
                ..
            } => {
                let mut file_lock = file.lock().await;
                if file_lock.is_none() {
                    let f = File::open(path.as_path())
                        .await
                        .map_err(|e| {
                            error!("Failed to open file: {:#?}", e);
                            ConnectionError(format!("Failed to open file: {:?}", e))
                        })?;
                    *file_lock = Some(f);
                }

                let file = file_lock.as_mut().unwrap();

                let part_size = min(chunk_size, end - start);
                let mut buf = vec![0u8; part_size as usize];
                file.seek(SeekFrom::Start(start))
                    .await
                    .map_err(|e| {
                        error!("Failed to seek file: {:#?}", e);
                        FileOperationError(format!("Failed to seek file: {:?}", e))
                    })?;
                file.read_exact(&mut buf)
                    .await
                    .map_err(|e| {
                        error!("Failed to read file: {:#?}", e);
                        FileOperationError(format!("Failed to read file: {:?}", e))
                    })?;

                let bytes = if end == file_size - 1 {
                    buf[..(end - start + 1) as usize].to_vec()
                } else {
                    buf
                };

                drop(file_lock);

                Ok(ChunkData::new(bytes, offset, is_last))
            },
        }
    }
}