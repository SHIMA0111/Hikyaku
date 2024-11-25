use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(crate) struct SharedDriveInfo {
    pub(crate) id: String,
    pub(crate) name: String,
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

pub(crate) struct DriveFileInfo {
    pub(crate) id: String,
    #[serde(mimeType)]
    pub(crate) mime_type: String,
}

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
