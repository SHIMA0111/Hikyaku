use std::cell::RefCell;
use std::num::NonZero;
use std::thread::available_parallelism;
use crate::errors::HikyakuResult;
use crate::utils::credential::Credential;
use crate::utils::credential::google_drive_credential::GoogleDriveCredential;
use crate::utils::credential::s3_credential::S3Credential;
use crate::utils::parser::{file_system_prefix_parser, FileSystemParseResult};

mod amazon_s3;
mod google_drive;

pub struct FileSystemBuilder<C: Credential> {
    file_info: RefCell<Option<FileSystemParseResult>>,
    file_system_credential: C,
    concurrency: RefCell<u16>,
}

impl<C: Credential> FileSystemBuilder<C> {
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

    pub fn add_file_path(self, path: &str) -> HikyakuResult<Self> {
        let parse_res = file_system_prefix_parser(path)?;
        *self.file_info.borrow_mut() = Some(parse_res);

        Ok(self)
    }

    pub fn concurrency(&self, concurrency: NonZero<u16>) -> &Self {
        *self.concurrency.borrow_mut() = concurrency.get();
        self
    }
}

impl From<S3Credential> for FileSystemBuilder<S3Credential> {
    fn from(value: S3Credential) -> Self {
        Self::new(value)
    }
}

impl From<GoogleDriveCredential> for FileSystemBuilder<GoogleDriveCredential> {
    fn from(value: GoogleDriveCredential) -> Self {
        Self::new(value)
    }
}