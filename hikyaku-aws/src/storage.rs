use std::sync::Arc;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::AggregatedBytes;
use hikyaku_core::chunk::Chunk;
use hikyaku_core::errors::{HikyakuError, HikyakuResult};
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
        let start_offset = self.chunk_size * offset;
        let end_offset = self.chunk_size * (offset + 1) - 1;

        let client = self.get_client(offset as usize);
        let part_data = client
            .get_object()
            .bucket(self.bucket.as_str())
            .key(self.file.get_key())
            .range(format!("bytes={}-{}", start_offset, end_offset))
            .send()
            .await
            .map_err(|e| {
                HikyakuError::S3Error(format!("{:?}", e))
            })?;

        let bytes = part_data
            .body
            .collect()
            .await
            .map_err(|e| {
                HikyakuError::S3Error(format!("{:?}", e))
            })?
            .into_bytes();

        Ok(Chunk::new(bytes, offset as usize, self.is_last(offset)))
    }

    async fn put_data(&self, data: Chunk) -> HikyakuResult<()> {
        todo!()
    }

    async fn create_folder(&self, path: &str) -> HikyakuResult<()> {
        todo!()
    }
    fn is_last(&self, offset: u64) -> bool {
        offset * self.chunk_size == self.file.size() - 1
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