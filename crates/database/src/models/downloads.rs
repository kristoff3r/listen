use api::{DownloadId, VideoId};
use diesel::{
    prelude::{Associations, Insertable},
    Identifiable, Queryable, Selectable,
};
use diesel_derive_enum::DbEnum;
use structural_convert::StructuralConvert;
use time::OffsetDateTime;

use crate::models::videos::Video;

#[derive(
    Clone, Debug, PartialEq, Queryable, Selectable, Identifiable, Associations, StructuralConvert,
)]
#[diesel(table_name = crate::schema::downloads)]
#[diesel(primary_key(download_id))]
#[diesel(belongs_to(Video))]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[convert(into(api::Download))]
pub struct Download {
    pub download_id: DownloadId,
    pub video_id: VideoId,
    pub error: Option<String>,
    pub status: DownloadStatus,
    pub retry_count: i32,
    pub force: bool,
    pub updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Insertable)]
#[diesel(table_name = crate::schema::downloads)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDownload<'a> {
    pub video_id: VideoId,
    pub error: Option<&'a str>,
    pub retry_count: Option<i32>,
    pub status: DownloadStatus,
}

#[derive(DbEnum, Clone, Debug, PartialEq, StructuralConvert)]
#[convert(into(api::DownloadStatus))]
#[ExistingTypePath = "crate::schema::sql_types::DownloadStatus"]
pub enum DownloadStatus {
    Pending,
    Processing,
    Finished,
    Failed,
}
