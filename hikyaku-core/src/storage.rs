use std::borrow::Cow;
use crate::chunk::Chunk;
use crate::errors::HikyakuResult;

pub trait Storage {
    fn get_data(&self, offset: u64) -> impl Future<Output = HikyakuResult<Chunk>> + Send;
    fn put_data(&self, data: Chunk) -> impl Future<Output = HikyakuResult<()>> + Send;
    fn create_folder(&self, path: &str) -> impl Future<Output = HikyakuResult<()>>;
    fn is_last(&self, offset: u64) -> bool;
}

pub struct File {
    folder_path: String,
    filename: String,
    size: u64,
}

impl File {
    pub fn new(folder_path: &str, filename: &str, size: u64) -> Self {
        Self {
            folder_path: folder_path.to_string(),
            filename: filename.to_string(),
            size,
        }
    }
    
    pub fn folder_path(&self) -> &str {
        &self.folder_path
    }
    
    pub fn filename(&self) -> &str {
        &self.filename
    }
    
    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn get_key(&self) -> String {
        format!("{}/{}", self.folder_path, self.filename)
    }
}