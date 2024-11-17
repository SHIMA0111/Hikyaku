mod file_system;

use std::path::Path;
use async_trait::async_trait;
use tokio::sync::Mutex;
use crate::errors::HikyakuError;

#[async_trait]
pub trait FileSystemOperation {
    async fn part_download(&self);
    async fn part_upload(&self);
}
