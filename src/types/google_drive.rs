use serde::Deserialize;
use crate::utils::file_type::FileType;

#[derive(Deserialize, Debug)]
pub(crate) struct SharedDriveInfo {
    pub(crate) id: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct SharedDriveQueryResponse {
    drives: Vec<SharedDriveInfo>,
}

impl SharedDriveQueryResponse {
    pub(crate) fn is_empty(&self) -> bool {
        self.drives.is_empty()
    }

    pub(crate) fn get_drives(&self) -> &[SharedDriveInfo] {
        &self.drives
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct DriveFileInfo {
    pub(crate) id: String,
    #[serde(rename = "mimeType")]
    pub(crate) mime_type: String,
    size: Option<String>,
}

impl DriveFileInfo {
    pub(crate) fn size(&self) -> Option<i64> {
        self.size
            .as_ref()
            .map(|s| s.parse::<i64>().unwrap_or(-1))
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct DriveFileQueryResponse {
    files: Vec<DriveFileInfo>
}

impl DriveFileQueryResponse {
    pub(crate) fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub(crate) fn files(&self) -> &[DriveFileInfo] {
        &self.files
    }
}

#[derive(Debug)]
pub(crate) struct GoogleDriveFile {
    id: String,
    mime_type: FileType,
    size: Option<u64>,
}

impl GoogleDriveFile {
    pub(crate) fn new(id: &str, mime_type: &str, size: Option<u64>) -> Self {
        let mime_type = FileType::from_mime(mime_type);

        Self {
            id: id.to_string(),
            mime_type,
            size,
        }
    }

    pub(crate) fn is_dir(&self) -> bool {
        self.mime_type == FileType::GoogleDriveFolder
    }

    pub (crate) fn is_invalid(&self) -> bool {
        matches!(self.mime_type, FileType::GoogleShortcut | FileType::GoogleDriveThirdPartyShortcut)
    }

    pub(crate) fn is_google_workspace_file(&self) -> bool {
        matches!(self.mime_type, FileType::GoogleDriveFile)
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.id
    }

    pub(crate) fn get_mime(&self) -> &str {
        self.mime_type.mime()
    }

    pub(crate) fn get_size(&self) -> Option<u64> {
        self.size
    }
}
