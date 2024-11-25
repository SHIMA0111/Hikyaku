pub type HikyakuResult<T> = Result<T, HikyakuError>;

#[derive(thiserror::Error, Debug)]
pub enum HikyakuError {
    #[error("OAuth failed: {0}")]
    OAuth2Error(String),
    #[error("Failed to the google drive process: {0}")]
    GoogleDriveError(String),
    #[error("Failed to the box process: {0}")]
    BoxError(String),
    #[error("Failed to build: {0}")]
    BuilderError(String),
    #[error("Get invalid argument error: {0}")]
    InvalidArgumentError(String),
    #[error("Env credential error: {0}")]
    EnvCredentialError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Unknown error: {0}")]
    UnknownError(String),
}
