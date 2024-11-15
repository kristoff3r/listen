use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Download {
    pub id: i32,
    pub video_id: i32,
    pub error: Option<String>,
    pub status: DownloadStatus,
    pub retry_count: i32,
    pub force: bool,
    pub updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Processing,
    Finished,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Video {
    pub id: i32,
    pub title: String,
    pub youtube_id: Option<String>,
    pub url: String,
    pub updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
    pub metadata: Option<serde_json::Value>,
}
