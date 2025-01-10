use std::io::SeekFrom;
use async_trait::async_trait;
use log::{error, warn};
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::mpsc::Receiver;
use crate::errors::HikyakuError::{FileOperationError, InvalidArgumentError, UnknownError};
use crate::errors::HikyakuResult;
use crate::services::file_system::{ChunkData, FileSystemObject};

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
                ..} => {
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
}
