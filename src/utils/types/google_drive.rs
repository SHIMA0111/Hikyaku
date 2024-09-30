use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GoogleDriveResponse {
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
    #[serde(rename = "incompleteSearch")]
    incomplete_search: bool,
    kind: String,
    drives: Option<Vec<GoogleSharedDriveDetails>>,
    files: Option<Vec<GoogleDriveFilesDetails>>
}

#[derive(Deserialize, Debug)]
pub struct GoogleSharedDriveDetails {
    id: String,
    name: String,
    kind: String,
    #[serde(rename = "createdTime")]
    created_time: Option<String>,
    hidden: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct GoogleDriveFilesDetails {
    id: String,
    name: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
    kind: String,
    #[serde(rename = "createdTime")]
    created_time: Option<String>,
    hidden: Option<bool>,
    #[serde(rename = "driveId")]
    drive_id: Option<String>,
    #[serde(rename = "fileExtension")]
    file_extension: Option<String>,
    #[serde(rename = "md5Checksum")]
    md5_checksum: Option<String>,
    parents: Option<Vec<String>>,
}

impl GoogleDriveResponse {
    pub fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }

    pub fn incomplete_search(&self) -> bool {
        self.incomplete_search
    }

    pub fn kind(&self) -> &str {
        self.kind.as_str()
    }

    pub fn drives(&self) -> &[GoogleSharedDriveDetails] {
        match self.drives.as_ref() {
            Some(drives) => drives,
            None => &[]
        }
    }

    pub fn files(&self) -> &[GoogleDriveFilesDetails] {
        match self.files.as_ref() {
            Some(files) => files,
            None => &[],
        }
    }
}

impl GoogleSharedDriveDetails {
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn kind(&self) -> &str {
        self.kind.as_str()
    }
}

impl GoogleDriveFilesDetails {
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn drive_id(&self) -> Option<&str> {
        self.drive_id.as_deref()
    }

    pub fn file_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn mime_type(&self) -> &str {
        self.mime_type.as_str()
    }

    pub fn kind(&self) -> &str {
        self.kind.as_str()
    }
}
