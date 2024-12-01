use std::sync::Arc;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use crate::errors::HikyakuError::{BuilderError, InvalidArgumentError};
use crate::errors::{HikyakuError, HikyakuResult};
use crate::services::file_system::FileSystemObject;
use crate::services::file_system_builder::FileSystemBuilder;
use crate::types::FileInfo;
use crate::utils::credential::Credential;
use crate::utils::credential::s3_credential::S3Credential;
use crate::utils::parser::FileSystemParseResult;

impl FileSystemBuilder<S3Credential, FileSystemParseResult> {
    /// Builds a `FileSystemObject` for Amazon S3 using specified credentials and file information.
    ///
    /// This function validates the file path to ensure it has the "s3://" prefix and then
    /// extracts the bucket and key information. It loads AWS configuration using the given
    /// credentials, creates S3 clients and retrieves the file size for the given object.
    ///
    /// # Returns
    ///
    /// * `HikyakuResult<FileSystemObject>` - A result containing the `FileSystemObject` if successful,
    ///   otherwise an `InvalidArgumentError` or `BuilderError` on failure.
    ///
    /// # Errors
    ///
    /// Returns an `InvalidArgumentError` if the file prefix is not "s3://".
    /// Returns a `BuilderError` if the bucket name cannot be found or the path is not set.
    ///
    /// # Example
    ///
    /// ```
    /// use hikyaku::utils::credential::s3_credential::S3Credential;
    /// use hikyaku::services::file_system_builder::FileSystemBuilder;
    ///
    /// async fn example() {
    ///     let cred = S3Credential::from_env().await.unwrap();
    ///     let file_obj = FileSystemBuilder::from(cred)
    ///         .set_file_path("s3://bucket-name/path/to/file")
    ///         .unwrap()
    ///         .build()
    ///         .await
    ///         .unwrap();
    ///     
    ///     assert!(file_obj.to_string().contains("AmazonS3"));
    /// }
    /// ```
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
        let client = Client::new(&shared_config);

        let file_size = Self::get_file_size(client, &bucket, &key).await?;

        let file_obj = FileSystemObject::AmazonS3 {
            clients,
            bucket,
            key,
            file_size,
        };

        Ok(file_obj)
    }

    async fn get_file_size(client: Client, bucket: &str, key: &str) -> HikyakuResult<Option<u64>> {
        let result = client
            .list_objects_v2()
            .bucket(bucket)
            .prefix(key)
            .send()
            .await
            .map_err(|e| {
                HikyakuError::ConnectionError(format!("Failed to get objects: {}", e))
            })?;

        let objects = result.contents();
        if objects.len() != 1 {
            Ok(None)
        }
        else {
            // This objects always has 1 object.
            let object = objects.get(0).unwrap();

            Ok(object.size().map(|size| size as u64))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_amazon_s3() {
        let cred =  S3Credential::from_env().await.unwrap();
        let file_obj = FileSystemBuilder::from(cred)
            .set_file_path("s3://test-bucket-hikyaku/datas/titanic/train.csv")
            .unwrap()
            .build()
            .await
            .unwrap();

        assert!(file_obj.to_string().contains("AmazonS3"));
        assert!(file_obj.to_string().contains("train.csv"));
    }
}