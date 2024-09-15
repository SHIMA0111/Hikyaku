use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::utils::errors::HikyakuError::{OAuth2Error, OpenIDConnectionError};

pub type HikyakuResult<T> = Result<T, HikyakuError>;

#[derive(Debug)]
pub enum HikyakuError {
    OAuth2Error(String),
    OpenIDConnectionError(String),
}

impl Display for HikyakuError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuth2Error(err) => write!(f, "OAuth failed: {}", err),
            OpenIDConnectionError(err) => write!(f, "Open ID connection failed: {}", err),
        }
    }
}

impl Error for HikyakuError {}
