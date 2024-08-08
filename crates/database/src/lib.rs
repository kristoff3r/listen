#[cfg(feature = "diesel")]
use diesel::{Queryable, Selectable};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[cfg(feature = "diesel")]
pub mod schema;

#[cfg_attr(feature = "diesel", derive(Queryable, Selectable))]
#[cfg_attr(feature = "diesel", diesel(table_name = crate::schema::videos))]
#[cfg_attr(feature = "diesel", diesel(check_for_backend(diesel::pg::Pg)))]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Video {
    pub id: i32,
    pub title: String,
    pub youtube_id: Option<String>,
    pub updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}
