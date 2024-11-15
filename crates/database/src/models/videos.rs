use api::VideoId;
use diesel::{dsl::insert_into, prelude::*};
use diesel_async::{
    scoped_futures::ScopedFutureExt, AsyncConnection, AsyncPgConnection, RunQueryDsl,
};
use structural_convert::StructuralConvert;
use time::OffsetDateTime;

use super::{Download, Result};

#[derive(Clone, Debug, PartialEq, Queryable, Selectable, Identifiable, StructuralConvert)]
#[diesel(primary_key(video_id))]
#[diesel(table_name = crate::schema::videos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[convert(into(api::Video))]
pub struct Video {
    pub video_id: VideoId,
    pub title: String,
    pub youtube_id: Option<String>,
    pub url: String,
    #[convert(into(api::Video, skip))]
    pub file_path: Option<String>,
    pub metadata: Option<serde_json::Value>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Insertable)]
#[diesel(table_name = crate::schema::videos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewVideo<'a> {
    pub title: &'a str,
    pub youtube_id: Option<&'a str>,
    pub url: &'a str,
    pub file_path: Option<&'a str>,
    pub metadata: Option<serde_json::Value>,
}

impl Video {
    pub async fn create(
        conn: &mut AsyncPgConnection,
        title: &str,
        youtube_id: &str,
        url: &str,
        metadata: serde_json::Value,
    ) -> Result<(Self, Download)> {
        conn.transaction(|conn| {
            async move {
                let video = Self::create_raw(conn, title, youtube_id, url, metadata).await?;
                let download = Download::create(conn, video.video_id).await?;

                Ok((video, download))
            }
            .scope_boxed()
        })
        .await
    }
    pub async fn create_raw(
        conn: &mut AsyncPgConnection,
        title: &str,
        youtube_id: &str,
        url: &str,
        metadata: serde_json::Value,
    ) -> Result<Self> {
        use crate::schema::videos::dsl as v;

        let result = insert_into(v::videos)
            .values(NewVideo {
                title,
                youtube_id: Some(youtube_id),
                url,
                metadata: Some(metadata),
                file_path: None,
            })
            .on_conflict_do_nothing()
            .get_result(conn)
            .await
            .optional()?;

        if let Some(result) = result {
            Ok(result)
        } else {
            let result = v::videos
                .filter(v::youtube_id.eq(youtube_id))
                .first(conn)
                .await?;
            Ok(result)
        }
    }

    pub async fn get_by_id(
        conn: &mut AsyncPgConnection,
        video_id: VideoId,
    ) -> Result<Option<Self>> {
        use crate::schema::videos::dsl as v;
        let result = v::videos.find(video_id).first(conn).await.optional()?;
        Ok(result)
    }

    pub async fn list(conn: &mut AsyncPgConnection) -> Result<Vec<Self>> {
        use crate::schema::videos::dsl as v;
        let results = v::videos.get_results(conn).await;
        let results = results?;
        Ok(results)
    }
}
