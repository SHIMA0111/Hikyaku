pub type HikyakuResult<T> = Result<T, HikyakuError>;

#[derive(thiserror::Error, Debug)]
pub enum HikyakuError {
    #[error("OAuth failed: {0}")]
    OAuth2Error(String),
    #[error("Failed to the google drive process: {0}")]
    GoogleDriveError(String),
    #[error("Failed to the s3 process: {0}")]
    S3Error(String),
    #[error("Failed to parse: {0}")]
    ParseError(String),
    #[error("Failed to build: {0}")]
    BuilderError(String),
    #[error("Get invalid argument error: {0}")]
    InvalidArgumentError(String),
    #[error("Env credential error: {0}")]
    EnvCredentialError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("File object not exist error: {0}")]
    NotExistFileError(String),
    #[error("File operation error: {0}")]
    FileOperationError(String),
    #[error("Unsupported error: {0}")]
    UnsupportedError(String),
    #[error("Unknown error: {0}")]
    UnknownError(String),
}
