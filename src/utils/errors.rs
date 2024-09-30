use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::utils::errors::HikyakuError::{GoogleDriveError, OAuth2Error, BoxError};

pub type HikyakuResult<T> = Result<T, HikyakuError>;

#[derive(Debug)]
pub enum HikyakuError {
    OAuth2Error(String),
    GoogleDriveError(String),
    BoxError(String),
}

impl Display for HikyakuError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuth2Error(err) => write!(f, "OAuth failed: {}", err),
            GoogleDriveError(err) => write!(f, "Failed to the google drive process: {}", err),
            BoxError(err) => write!(f, "Failed to the box process: {}", err)
        }
    }
}

impl Error for HikyakuError {}
