use std::cell::RefCell;
use std::num::NonZero;
use std::thread::available_parallelism;
use crate::errors::HikyakuResult;
use crate::types::FileInfo;
use crate::types::google_drive::GoogleDriveFileInfo;
use crate::utils::credential::Credential;
use crate::utils::credential::google_drive_credential::GoogleDriveCredential;
use crate::utils::credential::s3_credential::S3Credential;
use crate::utils::parser::{file_system_prefix_parser, FileSystemParseResult};

pub mod amazon_s3;
pub mod google_drive;


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