use std::sync::Arc;
use log::{error};
use reqwest::{Client};
use crate::errors::HikyakuError::{BuilderError, ConnectionError, GoogleDriveError, InvalidArgumentError, UnknownError, UnsupportedError};
use crate::errors::HikyakuResult;
use crate::services::file_system::FileSystemObject;
use crate::services::file_system_builder::FileSystemBuilder;
use crate::types::FileInfo;
use crate::types::google_drive::{DriveFileInfo, DriveFileQueryResponse, GoogleDriveFile, GoogleDriveFileInfo, SharedDriveInfo, SharedDriveQueryResponse};
use crate::utils::credential::Credential;
use crate::utils::credential::google_drive_credential::GoogleDriveCredential;
use crate::utils::file_type::FileType;
use crate::utils::parser::path_to_names_vec;
use crate::utils::reqwest::AuthType::Bearer;
use crate::utils::reqwest::get_client_with_token;

impl FileSystemBuilder<GoogleDriveCredential, GoogleDriveFileInfo> {
    /// Sets the parent IDs and the file path key for the Google Drive file operation.
    ///
    /// This function updates the `file_info` state in the builder with the provided
    /// `parents_ids` list and `path`. It is useful for identifying specific parent folders
    /// on Google Drive and setting a file path.
    ///
    /// # Arguments
    ///
    /// * `parents_ids` - A vector of string slices that holds the unique identifiers of the parent folders.
    /// * `path` - A string slice that represents the file path to be associated with the parent IDs.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the builder instance for further chaining of method calls.
    pub fn set_parents_ids_and_key(self,
                                   parents_ids: Vec<&str>,
                                   path: &str) -> Self {
        let file_info = GoogleDriveFileInfo::ParentId {
            parent_ids: parents_ids.iter().map(|id| id.to_string()).collect(),
            file_path: path.to_string()
        };

        *self.file_info.borrow_mut() = Some(file_info);
        self
    }

    
    /// Sets the target file ID for the Google Drive file operation.
    ///
    /// This function updates the `file_info` state in the builder with the provided
    /// `target_file_id`. It is useful for identifying a specific file
    /// on Google Drive when constructing a `FileSystemObject`.
    ///
    /// # Arguments
    ///
    /// * `target_file_id` - A string slice that holds the unique identifier of the target file.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns the builder instance for further chaining of method calls.
    pub fn set_file_id(self, target_file_id: &str) -> Self {
        let file_info = GoogleDriveFileInfo::FileId(target_file_id.to_string());

        *self.file_info.borrow_mut() = Some(file_info);
        self
    }

    
    /// Builds a `FileSystemObject` for Google Drive using the specified credentials and file information.
    ///
    /// This function validates the file path to ensure it corresponds to a Google Drive location (either "gd://" or "gds://") 
    /// and resolves the file path to the deepest existing path in Google Drive if you set path as identity of the file. 
    /// It creates HTTP clients for operations, and prepares necessary data 
    /// such as file ID, MIME type, and upload filename for Google Drive interactions.
    ///
    /// # Returns
    ///
    /// * `HikyakuResult<FileSystemObject>` - A result containing the `FileSystemObject` if successful,
    ///   otherwise an `InvalidArgumentError` or `BuilderError` or so on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// use time::{Duration, OffsetDateTime};
    /// use hikyaku::utils::credential::google_drive_credential::GoogleDriveCredential;
    /// use hikyaku::services::file_system_builder::FileSystemBuilder;
    ///
    /// async fn example() {
    ///     let cred = GoogleDriveCredential::new("access_token", "refresh_token", OffsetDateTime::now_utc() + Duration::hours(1));
    ///     let file_obj = FileSystemBuilder::from(cred)
    ///         .set_file_id("")
    ///         .build()
    ///         .await
    ///         .unwrap();
    ///     
    ///     assert!(file_obj.to_string().contains("GoogleDrive"));
    /// }
    /// ```
    pub async fn build(self) -> HikyakuResult<FileSystemObject> {
        let get_upload_filename = |path: &str| -> Option<Arc<String>> {
            if path.is_empty() {
                None
            } else {
                Some(Arc::new(path.rsplit_once("/")
                    .map(|(_, file_name)| file_name.to_string())
                    .unwrap_or(path.to_string())))
            }
        };

        let (google_drive_file, not_exist_paths, upload_filename) = match self.file_info.borrow().as_ref() {
            Some(GoogleDriveFileInfo::Parsed(info)) => {
                if !["gd://", "gds://"].contains(&info.get_prefix()) {
                    return Err(InvalidArgumentError("File system prefix is not gd:// or gds".to_string()));
                }

                let client = get_client_with_token(
                    self.file_system_credential.get_credential().get_access_token(),
                    Bearer)?;

                let shared_drive_ids = match info.get_namespace().map(String::from) {
                    Some(name) => get_shared_drive(&client, &name).await?,
                    None => vec![]
                };
                let res = self.resolve_path_to_existing_depth(
                    &shared_drive_ids, info.get_path()).await?;
                let upload_filename = get_upload_filename(info.get_path());
                (res.0, res.1, upload_filename)
            },
            Some(GoogleDriveFileInfo::ParentId { parent_ids, file_path }) => {
                let res = self.resolve_path_to_existing_depth(
                    &parent_ids, file_path).await?;
                let upload_filename = get_upload_filename(file_path);
                (res.0, res.1, upload_filename)
            },
            Some(GoogleDriveFileInfo::FileId(file_id)) => {
                let client = get_client_with_token(
                    self.file_system_credential.get_credential().get_access_token(),
                    Bearer)?;
                let (file_info, filename) =
                    if file_id.is_empty() {
                        // My Drive root, the file id should be "".
                        let drive_file = GoogleDriveFile::new(
                            file_id,
                            "application/vnd.google-apps.folder",
                            None,
                        );
                        (drive_file, None)
                    }
                    else if let Ok(info) = get_drive_from_id(&client, file_id).await {
                        // The file id can be Shared Drive ID.
                        (info, None)
                    } else {
                        let (info, filename) = get_file_from_id(&client, file_id).await?;
                        (info, Some(Arc::new(filename)))
                    };
                (Some(file_info), vec![], filename)
            }
            None => {
                return Err(BuilderError("Path is not set".to_string()));
            },
        };

        let clients = (0..self.concurrency.into_inner())
            .map(|_| Arc::new(Client::new()))
            .collect::<Vec<_>>();
        let (queryable_file_or_parent_id, mime_type, file_size) = match google_drive_file {
            Some(file) => {
                if file.is_google_workspace_file() || file.is_invalid() {
                    return Err(UnsupportedError(format!("The {} file is currently unsupported.", file.get_mime())));
                }

                (file.get_id().to_string(), file.get_mime().to_string(), file.get_size())
            },
            None => (
                "".to_string(),
                FileType::Unknown.mime().to_string(),
                None),
        };

        let file_obj = FileSystemObject::GoogleDrive {
            clients,
            google_drive_token: Arc::new(self.file_system_credential.get_credential()),
            queryable_file_or_parent_id: Arc::new(queryable_file_or_parent_id),
            not_exist_file_paths: Arc::new(not_exist_paths),
            upload_filename,
            mime_type: Arc::new(mime_type),
            file_size,
            chunk_size: self.chunk_size.into_inner(),
        };

        Ok(file_obj)
    }


    /// Resolves the path to the most deeply existing file or folder in Google Drive
    /// from the specified parents.
    ///
    /// This function explores the given path in Google Drive and returns a tuple containing
    /// the `GoogleDriveFile` corresponding to the most deeply existing file or folder and
    /// a vector of the path components that do not exist in current GoogleDrive.
    ///
    /// # Arguments
    ///
    /// * `parent_ids` - Slice of the parent ids when the slice is empty, it represents
    /// the specified path has no parent (start from root place).
    /// * `path` - The path to be resolved, represented as a string.
    ///
    /// # Returns
    ///
    /// `HikyakuResult<(Option<GoogleDriveFile>, Vec<String>)>` - A result containing a tuple.
    /// The first element is an `Option` with the `GoogleDriveFile` corresponding to the most deeply
    /// existing file or folder. The second element is a vector of the path component names that do not exist on the current GoogleDrive.
    async fn resolve_path_to_existing_depth(&self, parent_ids: &[String], path: &str) -> HikyakuResult<(Option<GoogleDriveFile>, Vec<String>)> {
        let client = get_client_with_token(
            self.file_system_credential.get_credential().get_access_token(),
            Bearer)?;

        let path_names = path_to_names_vec(path, false)?;

        // Store the explored paths nums to skip paths when collect not exist paths.
        let mut complete_explore_path_num = 0;
        let mut parent_infos = initial_parents(parent_ids);

        for name in &path_names {
            let query_response = query_drive_files(&client, name, &parent_infos).await?;
            if query_response.is_empty() {
                break
            }
            complete_explore_path_num += 1;

            parent_infos = query_response;
        }
        
        // In the above loop, the most match(most deep match path on the current drive) treat as 
        // the user input path, even if the partway paths are multiple.
        // However, if there is 2 or more the most match path exists, it cannot identify the user input actually.
        if parent_infos.len() >= 2 {
            return Err(InvalidArgumentError(format!("File path '{}' is ambiguous. There is multiple candidate on the same depth of the path in Google Drive.", path)));
        }

        let res = parent_infos.into_iter().next();
        
        // In this function, the not exist path not create due to the builder is originally gather information
        // to create object for Hikyaku. Therefore, collect the not exist path to pass it to FileSystemObject.
        let remain_path = path_names
            .iter()
            // Skip explored paths
            .skip(complete_explore_path_num)
            .cloned()
            .collect::<Vec<_>>();

        Ok((res, remain_path))
    }
}


/// Fetches the IDs of shared drives with the given name from Google Drive.
///
/// # Arguments
///
/// * `client` - The client used to send the request to Google Drive which has token header as default.
/// * `shared_drive_name` - The name of the shared drive to search for.
///
/// # Returns
///
/// `HikyakuResult<Vec<String>>` - A result containing a vector of shared drive IDs, or an error if the operation fails.
async fn get_shared_drive(client: &Client, shared_drive_name: &str) -> HikyakuResult<Vec<String>> {
    let response = client
        .get("https://www.googleapis.com/drive/v3/drives")
        .query(&[("q", format!("name = '{}'", shared_drive_name))])
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send request to Google Drive API: {:#?}", e);
            ConnectionError(format!("Failed to send request to Google Drive API: {:?}", e))
        })?;

    let shared_drive_ids = response
        .json::<SharedDriveQueryResponse>()
        .await
        .map_err(|e| GoogleDriveError(format!("Failed to parse response from Google Drive API: {:?}", e)))?;

    if shared_drive_ids.is_empty() {
        return Err(InvalidArgumentError(format!("Shared drive name: '{}' is not found", shared_drive_name)));
    }

    let ids = shared_drive_ids
        .get_drives()
        .iter()
        .map(|shared_drive| shared_drive.id.clone())
        .collect::<Vec<_>>();

    Ok(ids)
}

/// Get initial parents as [GoogleDriveFile] from the drives ids. 
/// 
/// # Arguments
///
/// * `drives` - A slice of Google Drive Shared Drive ids. If not shared drive, it should empty vector.
/// 
/// # Returns
/// 
/// `Vec<GoogleDriveFile>` - Vector of the [GoogleDriveFile] corresponding to the input ids.
fn initial_parents(drives: &[String]) -> Vec<GoogleDriveFile> {
    drives
        .iter()
        .map(|id| GoogleDriveFile::new(id, "", None))
        .collect::<Vec<_>>()
}


/// Queries Google Drive for files or folders with a given name under specified parent directories.
///
/// # Arguments
///
/// * `client` - The client used to send the request to Google Drive which has token header as default.
/// * `file_or_folder_name` - The name of the file or folder to search for.
/// * `parents` - A slice of parent([GoogleDriveFile]) directories to search within.
///
/// # Returns
///
/// `HikyakuResult<Vec<GoogleDriveFile>>` - A result containing a vector of found Google Drive files, or an error if the operation fails.
async fn query_drive_files(client: &Client, file_or_folder_name: &str, parents: &[GoogleDriveFile]) -> HikyakuResult<Vec<GoogleDriveFile>> {
    let query = query_statement_builder(file_or_folder_name, parents);

    let response = client
        .get("https://www.googleapis.com/drive/v3/files")
        .query(&[
            ("q", &query),
            ("supportsAllDrives", &"true".to_string()),
            ("includeItemsFromAllDrives", &"true".to_string()),
            ("fields", &"files(id, mimeType, size)".to_string()),
        ])
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send request to Google Drive API: {:#?}", e);
            ConnectionError(format!("Failed to send request to Google Drive API: {:?}", e))
        })?;

    if !response.status().is_success() {
        error!("Failed to query files for Google Drive API: {}", response.status());
        return Err(ConnectionError(format!("Failed to query files for Google Drive API: {}", response.status())));
    }

    let query_response = response
        .json::<DriveFileQueryResponse>()
        .await
        .map_err(|e| UnknownError(format!("Failed to parse response from Google Drive API: {:#?}", e)))?;

    let mut query_result = vec![];
    for file in query_response.files() {
        let size = if let Some(size) = file.size() {
            // Google Drive API returns the file size via JSON string. When it cannot parse to i64, it treats as -1 for handling.
            if size < 0 {
                return Err(GoogleDriveError("Google Drive returns invalid size information. If this issue occurs, please report to the author.".to_string()));
            }

            Some(size as u64)
        } else {
            None
        };
        query_result.push(GoogleDriveFile::new(&file.id, &file.mime_type, size))
    }

    Ok(query_result)
}


/// Builds a query statement to search for files or folders in Google Drive.
///
/// # Arguments
///
/// * `file_folder_name` - The name of the file or folder to search for.
/// * `parents` - A slice of parent directories (GoogleDriveFile) to search within.
///
/// # Returns
///
/// `String` - The constructed query statement to be used in Google Drive API requests.
fn query_statement_builder(file_folder_name: &str, parents: &[GoogleDriveFile]) -> String {
    let query = format!("name = '{}'", file_folder_name);
    let mut parents_query = vec![];
    for parent_info in parents {
        parents_query.push(format!("'{}' in parents", parent_info.get_id()));
    }
    if parents_query.len() > 0 {
        format!("{} and ({})", query, parents_query.join(" or "))
    }
    else {
        query
    }
}


/// Retrieves a Google Drive file by its ID.
///
/// This function sends a request to the Google Drive API to obtain details about a shared drive
/// identified by the specified file ID.
async fn get_drive_from_id(client: &Client, drive_id: &str) -> HikyakuResult<GoogleDriveFile> {
    let request_uri = format!("https://www.googleapis.com/drive/v3/drives/{drive_id}");
    let response = client
        .get(request_uri)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send request to Google Drive API: {:#?}", e);
            ConnectionError(format!("Failed to send request to Google Drive API: {:#?}", e))
        })?;

    if !response.status().is_success() {
        error!("Failed to get drive by ID for Google Drive API: {}", response.status());
        return Err(ConnectionError(format!("Failed to get drive by ID for Google Drive API: {}", response.status())));
    }

    let get_response = response
        .json::<SharedDriveInfo>()
        .await
        .map_err(|e| UnknownError(format!("Failed to parse response from Google Drive API: {:#?}", e)))?;

    let google_drive_file = GoogleDriveFile::new(
        get_response.id.as_ref(),
        "application/vnd.google-apps.folder",
        None,
    );

    Ok(google_drive_file)
}

/// Retrieves a Google Drive file by its ID.
///
/// This function sends a request to the Google Drive API to obtain details about a file
/// identified by the specified file ID.
async fn get_file_from_id(client: &Client, file_id: &str) -> HikyakuResult<(GoogleDriveFile, String)> {
    let request_uri = format!("https://www.googleapis.com/drive/v3/files/{file_id}");

    let response = client
        .get(request_uri)
        .query(&[
            ("supportsAllDrives", &"true".to_string()),
        ])
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send request to Google Drive API: {:#?}", e);
            ConnectionError(format!("Failed to send request to Google Drive API: {:#?}", e))
        })?;

    if !response.status().is_success() {
        error!("Failed to get files by ID for Google Drive API: {}", response.status());
        return Err(ConnectionError(format!("Failed to get files by ID for Google Drive API: {}", response.status())));
    }

    let get_response = response
        .json::<DriveFileInfo>()
        .await
        .map_err(|e| UnknownError(format!("Failed to parse response from Google Drive API: {:#?}", e)))?;

    let google_drive_file = GoogleDriveFile::new(
        get_response.id.as_ref(),
        get_response.mime_type.as_ref(),
        get_response.size().map(|size| size as u64));

    Ok((google_drive_file, get_response.name))
}

#[cfg(test)]
mod tests {
    use std::env;
    use time::{Duration, OffsetDateTime};
    use super::*;

    #[tokio::test]
    async fn test_build_google_drive() {
        let access_token = env::var("GOOGLE_DRIVE_TOKEN").unwrap();
        let cred = GoogleDriveCredential::new(
            &access_token,
            "",
            OffsetDateTime::now_utc() + Duration::hours(1),
        );

        let file_obj = FileSystemBuilder::from(cred)
            .set_file_path("gds://datas/titanic/train.csv")
            .unwrap()
            .build()
            .await
            .unwrap();

        assert!(file_obj.to_string().contains("1rmRBMDEMurxCBwmpVj47THuYuDVDsco"));
    }
}