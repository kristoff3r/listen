use api::{DownloadId, VideoId};
use diesel::{
    dsl::{insert_into, now},
    prelude::*,
    update, BelongingToDsl, Identifiable, Queryable, Selectable,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_derive_enum::DbEnum;
use structural_convert::StructuralConvert;
use time::OffsetDateTime;

use super::{Result, Video};

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

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
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

impl Download {
    pub async fn create(conn: &mut AsyncPgConnection, video_id: VideoId) -> Result<Download> {
        use crate::schema::downloads::dsl as d;

        let result = insert_into(d::downloads)
            .values(NewDownload {
                video_id,
                error: None,
                retry_count: None,
                status: DownloadStatus::Pending,
            })
            .get_result(conn)
            .await?;

        Ok(result)
    }

    pub async fn list(conn: &mut AsyncPgConnection) -> Result<Vec<Self>> {
        use crate::schema::downloads::dsl as d;
        let results = d::downloads.get_results(conn).await;
        let results = results?;
        Ok(results)
    }

    pub async fn get_next_download(conn: &mut AsyncPgConnection) -> Result<Option<(Video, Self)>> {
        use crate::schema::downloads::dsl as d;
        use crate::schema::videos::dsl as v;

        let results = d::downloads
            .inner_join(v::videos)
            .filter(d::status.eq(DownloadStatus::Pending))
            .filter(v::url.is_not_null())
            .order_by((d::retry_count.asc(), d::created_at.asc()))
            .select((Video::as_select(), Download::as_select()))
            .first(conn)
            .await
            .optional()?;

        Ok(results)
    }

    pub async fn list_for_videos(
        conn: &mut AsyncPgConnection,
        videos: &[Video],
    ) -> Result<Vec<Self>> {
        let results = Download::belonging_to(videos).get_results(conn).await?;
        Ok(results)
    }

    pub async fn update_set_status(
        conn: &mut AsyncPgConnection,
        download_id: DownloadId,
        download_status: DownloadStatus,
        error_text: &str,
    ) -> Result<()> {
        use crate::schema::downloads::dsl as d;

        update(d::downloads.filter(d::download_id.eq(download_id)))
            .set((
                d::updated_at.eq(now),
                d::retry_count.eq(d::retry_count + 1),
                d::error.eq(error_text),
                d::status.eq(download_status),
            ))
            .execute(conn)
            .await?;
        Ok(())
    }
}
