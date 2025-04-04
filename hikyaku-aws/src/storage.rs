use std::sync::Arc;
use aws_sdk_s3::Client;
use hikyaku_core::chunk::Chunk;
use hikyaku_core::errors::HikyakuResult;
use hikyaku_core::storage::{File, Storage};

pub struct S3 {
    clients: Vec<Arc<Client>>,
    bucket: String,
    file: File,
    chunk_size: u64,
}

impl S3 {
    pub fn new(clients: Vec<Client>,
               bucket: String,
               file: File,
               chunk_size: u64) -> Self {
        Self {
            clients: clients.into_iter().map(|c| Arc::new(c)).collect(),
            bucket,
            file,
            chunk_size,
        }
    }
}

impl Storage for S3 {
    async fn get_data(&self, offset: u64) -> HikyakuResult<Chunk> {
        todo!()
    }

    async fn put_data(&self, data: Chunk) -> HikyakuResult<()> {
        todo!()
    }

    async fn create_folder(&self, path: &str) -> HikyakuResult<()> {
        todo!()
    }
}

impl S3 {
    fn get_client(&self, idx: usize) -> Arc<Client> {
        self.clients[idx % self.clients.len()].clone()
    }

    fn chunk_size(&self) -> u64 {
        self.chunk_size
    }
}