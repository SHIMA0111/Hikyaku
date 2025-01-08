use tokio::sync::mpsc::Sender;
use crate::errors::HikyakuError::NotExistFileError;
use crate::errors::HikyakuResult;
use crate::services::file_system::{ChunkData, FileSystemObject};

pub trait Download {
    fn download(&self, sender: Sender<ChunkData>) -> HikyakuResult<()>;
}

impl Download for FileSystemObject {
    fn download(&self, sender: Sender<ChunkData>) -> HikyakuResult<()> {
        if !self.is_downloadable() {
            return Err(NotExistFileError(format!("File system object is not downloadable. File system object: {}", self)));
        }

        Ok(())
    }
}

impl FileSystemObject {
    fn partial_download(&self) -> HikyakuResult<ChunkData> {
        let chunk_size = self.chunk_size();

        match self {
            Self::AmazonS3 {
                clients,
                bucket,
                key,
                file_size,
                ..
            } => Ok(ChunkData::new(vec![], 0, false)),
            Self::GoogleDrive {
                clients,
                google_drive_token,
                queryable_file_or_parent_id,
                not_exist_file_paths,
                mime_type,
                file_size,
                ..
            } => Ok(ChunkData::new(vec![], 0, false)),
            Self::Local {
                path,
                file_size,
                concurrency,
                ..
            } => Ok(ChunkData::new(vec![], 0, false)),
        }
    }
}