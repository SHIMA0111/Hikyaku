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
        match self {
            Self::AmazonS3 {..} => Ok(ChunkData::new(vec![], 0, false)),
            Self::GoogleDrive {..} => Ok(ChunkData::new(vec![], 0, false)),
            Self::Local {..} => Ok(ChunkData::new(vec![], 0, false)),
        }
    }
}