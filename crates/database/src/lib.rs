#[cfg(feature = "diesel")]
use diesel::{Associations, Identifiable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[cfg(feature = "diesel")]
pub mod schema;

#[cfg(feature = "diesel")]
pub const MIGRATIONS: diesel_async_migrations::EmbeddedMigrations =
    diesel_async_migrations::embed_migrations!();

#[cfg_attr(feature = "diesel", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "diesel", diesel(table_name = crate::schema::videos))]
#[cfg_attr(feature = "diesel", diesel(check_for_backend(diesel::pg::Pg)))]
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

#[cfg_attr(
    feature = "diesel",
    derive(Queryable, Selectable, Identifiable, Associations)
)]
#[cfg_attr(feature = "diesel", diesel(table_name = crate::schema::downloads))]
#[cfg_attr(feature = "diesel", diesel(belongs_to(Video)))]
#[cfg_attr(feature = "diesel", diesel(check_for_backend(diesel::pg::Pg)))]
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

#[cfg_attr(feature = "diesel", derive(diesel_derive_enum::DbEnum))]
#[cfg_attr(
    feature = "diesel",
    ExistingTypePath = "crate::schema::sql_types::DownloadStatus"
)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Processing,
    Finished,
    Failed,
}
