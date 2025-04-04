pub mod google_drive;

pub trait FileInfo {
    /// Get prefix(e.x. `s3://`, `file://`, and so)
    fn get_prefix(&self) -> &str;
    /// Get namespace(S3 bucket name, Google Drive SharedDrive name).
    /// If the file system has no namespace, return [None].
    fn get_namespace(&self) -> Option<&str>;
    /// Get path except for namespace(`file://` and `gd://` has no namespace so all path is parsed)
    fn get_path(&self) -> &str;
}
