use database::DownloadStatus;
use diesel::prelude::Insertable;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Insertable, Deserialize)]
#[diesel(table_name = database::schema::videos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewVideo<'a> {
    pub title: &'a str,
    pub youtube_id: Option<&'a str>,
    pub url: &'a str,
    pub file_path: Option<&'a str>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Insertable, Deserialize)]
#[diesel(table_name = database::schema::downloads)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDownload<'a> {
    pub video_id: i32,
    pub error: Option<&'a str>,
    pub retry_count: Option<i32>,
    pub status: DownloadStatus,
}
