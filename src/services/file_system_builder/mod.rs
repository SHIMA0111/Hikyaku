use std::cell::RefCell;
use std::io;
use std::num::NonZero;
use std::path::{Path, PathBuf};
use std::thread::available_parallelism;
use log::error;
use crate::errors::HikyakuError::InvalidArgumentError;
use crate::errors::HikyakuResult;
use crate::services::file_system::FileSystemObject;
use crate::types::FileInfo;
use crate::types::google_drive::GoogleDriveFileInfo;
use crate::utils::credential::{Credential, NoCredential};
use crate::utils::credential::google_drive_credential::GoogleDriveCredential;
use crate::utils::credential::s3_credential::S3Credential;
use crate::utils::parser::{file_system_prefix_parser, FileSystemParseResult};

pub(crate) mod amazon_s3;
pub(crate) mod google_drive;


/// A builder for constructing instances of a file system with a specified
/// credential type and file information type.
///
/// # Type Parameters
///
/// * `C`: The type of credential used for the file system, 
///   must implement the `Credential` trait.
/// * `FI`: The type of file information, must implement `FileInfo` and
///   `From<FileSystemParseResult>`.
pub struct FileSystemBuilder<C, FI>
where
    C: Credential,
    FI: FileInfo + From<FileSystemParseResult>,
{
    file_info: RefCell<Option<FI>>,
    file_system_credential: C,
    concurrency: RefCell<u16>,
}

impl<C, FI> FileSystemBuilder<C, FI>
where
    C: Credential,
    FI: FileInfo + From<FileSystemParseResult>
{
    fn new(file_system_credential: C) -> Self {
        let parallelism = available_parallelism()
            // SAFETY: NonZero is always Some if the input is not `0`
            .unwrap_or(NonZero::new(1).unwrap())
            .get() * 2;
        let concurrency = RefCell::new(if parallelism > u16::MAX as usize {
            u16::MAX
        } else {
            parallelism as u16
        });

        Self {
            file_info: RefCell::new(None),
            file_system_credential,
            concurrency,
        }
    }

    
    /// Sets the file path for the file system and updates the file information.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice representing the path to the file. The path will be parsed
    ///   to derive file information which is set in the builder.
    ///
    /// # Returns
    ///
    /// * `HikyakuResult<Self>` - Returns the updated instance of `FileSystemBuilder` wrapped
    ///   in a result type. If the parsing of the file system prefix fails, an error is returned.
    ///
    /// # Errors
    ///
    /// An error is returned if the `file_system_prefix_parser` fails to parse the provided `path`.
    ///
    pub fn set_file_path(self, path: &str) -> HikyakuResult<Self> {
        let parse_res = file_system_prefix_parser(path)?;
        let info = FI::from(parse_res);
        *self.file_info.borrow_mut() = Some(info);

        Ok(self)
    }


    /// Sets the concurrency level for the file system operations.
    ///
    /// # Arguments
    ///
    /// * `concurrency` - A `NonZero<u16>` specifying the desired concurrency level.
    ///
    /// # Returns
    ///
    /// * `&Self` - Returns a reference to the updated instance of the builder.
    ///
    /// This method allows adjusting the level of concurrency for operations
    /// performed by the file system, enabling more efficient utilization of
    /// system resources.
    pub fn concurrency(&self, concurrency: NonZero<u16>) -> &Self {
        *self.concurrency.borrow_mut() = concurrency.get();
        self
    }
}

impl FileSystemBuilder<NoCredential, FileSystemParseResult> {
    /// Creates a new instance of `FileSystemBuilder` for local file systems.
    ///
    /// This method initializes a file system builder configured to work with local files 
    /// by using the `NoCredential` type. The builder allows you to set file paths 
    /// and configure concurrency for file operations.
    ///
    /// # Returns
    ///
    /// * `FileSystemBuilder<NoCredential, FileSystemParseResult>` - A new instance configured 
    ///   with no authentication credentials, suitable for local file system operations.
    pub fn new_local() -> Self {
        Self::new(NoCredential)
    }


    /// Builds the file system object for local file systems.
    ///
    /// This method finalizes the configuration of the file system builder and 
    /// creates an instance of `FileSystemObject` based on the current state of 
    /// the builder. It checks that the path begins with "file://" and determines 
    /// if the path is a file or directory. 
    ///
    /// # Returns
    ///
    /// * `HikyakuResult<FileSystemObject>` - An instance of `FileSystemObject` 
    ///   representing the configured file system. Returns a result type; if the 
    ///   path is not set or does not start with "file://", it returns an 
    ///   `InvalidArgumentError`.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    ///
    /// - The file system prefix is not "file://".
    /// - The path has not been set.
    pub fn build(&self) -> HikyakuResult<FileSystemObject> {
        let path = match self.file_info.borrow().as_ref() {
            Some(file_info) => {
                if file_info.get_prefix() != "file://" {
                    return Err(InvalidArgumentError("File system prefix is not file://".to_string()));
                }
                format!("/{}", file_info.get_path())
            },
            None => {
                return Err(InvalidArgumentError("Path is not set".to_string()));
            }
        };

        let (is_dir, file_size) = match Path::new(&path).metadata() {
            Ok(metadata) => {
                let size = if metadata.is_file() {
                    Some(metadata.len())
                } else {
                    None
                };
                (metadata.is_dir(), size)
            },
            Err(e) => {
                if let io::ErrorKind::PermissionDenied = e.kind() {
                    error!("Input path: {} cannot be accessed.", path);
                    return Err(InvalidArgumentError("Permission denied".to_string()));
                }

                // When metadata cannot get, it means there is no file/dir or not have permission.
                if path.ends_with("/") {
                    (true, None)
                } else {
                    (false, None)
                }
            }
        };

        let file_obj = FileSystemObject::Local {
            path: PathBuf::from(path),
            is_dir,
            file_size,
        };

        Ok(file_obj)
    }
}

impl From<S3Credential> for FileSystemBuilder<S3Credential, FileSystemParseResult> {
    fn from(value: S3Credential) -> Self {
        Self::new(value)
    }
}

impl From<GoogleDriveCredential> for FileSystemBuilder<GoogleDriveCredential, GoogleDriveFileInfo> {
    fn from(value: GoogleDriveCredential) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod test {
    use std::env;
    use crate::services::file_system_builder::FileSystemBuilder;

    #[tokio::test]
    async fn test_build_local() {
        let current_dir = env::current_dir().unwrap();
        let file_obj = FileSystemBuilder::new_local()
            .set_file_path(&format!("file://{}/.gitignore", current_dir.display()))
            .unwrap()
            .build()
            .unwrap();

        assert!(file_obj.to_string().contains("Local"));
        assert!(file_obj.to_string().contains(".gitignore"));
    }
}
