use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GoogleDriveResponse {
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
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
}