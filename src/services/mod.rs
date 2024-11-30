mod file_system;
mod file_system_builder;

use async_trait::async_trait;

#[async_trait]
pub trait FileSystemOperation {
    async fn part_download(&self);
    async fn part_upload(&self);
}
