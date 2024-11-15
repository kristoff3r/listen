use diesel::{prelude::Insertable, Identifiable, Queryable, Selectable};
use structural_convert::StructuralConvert;
use time::OffsetDateTime;

#[derive(Clone, Debug, PartialEq, Queryable, Selectable, Identifiable, StructuralConvert)]
#[diesel(table_name = crate::schema::videos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[convert(into(api::Video))]
pub struct Video {
    pub id: i32,
    pub title: String,
    pub youtube_id: Option<String>,
    pub url: String,
    pub updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
    pub metadata: Option<serde_json::Value>,
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
