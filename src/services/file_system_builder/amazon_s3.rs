use std::cell::RefCell;
use std::sync::Arc;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use crate::errors::HikyakuError::{BuilderError, InvalidArgumentError};
use crate::errors::HikyakuResult;
use crate::services::file_system::FileSystemObject;
use crate::services::file_system_builder::FileSystemBuilder;
use crate::utils::credential::Credential;
use crate::utils::credential::s3_credential::S3Credential;

impl FileSystemBuilder<S3Credential> {
    pub async fn build(self) -> HikyakuResult<FileSystemObject> {
        let (bucket, key) = match self.file_info.borrow().as_ref() {
            Some(file_info) => {
                if file_info.get_prefix() != "s3://" {
                    return Err(InvalidArgumentError("File system prefix is not s3://".to_string()));
                }
                let bucket = file_info.get_namespace()
                    .ok_or(BuilderError("Bucket name cannot found".to_string()))?
                    .to_string();

                (bucket, file_info.get_path().to_string())
            },
            None => {
                return Err(BuilderError("Path is not set".to_string()));
            }
        };

        let file_system_credential = self.file_system_credential;

        let shared_config = aws_config::defaults(BehaviorVersion::v2024_03_28())
            .region(file_system_credential.get_region())
            .credentials_provider(file_system_credential.get_credential())
            .load()
            .await;
        let concurrency = self.concurrency.borrow().to_owned();
        let clients = (0..concurrency)
            .map(|_| Arc::new(Client::new(&shared_config)))
            .collect::<Vec<_>>();
        let file_obj = FileSystemObject::AmazonS3 {
            clients,
            bucket,
            key,
            file_size: None,
        };
        todo!()
    }
}