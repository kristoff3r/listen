use diesel::prelude::Insertable;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Insertable, Deserialize)]
#[diesel(table_name = database::schema::videos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewVideo<'a> {
    pub title: &'a str,
    pub youtube_id: Option<&'a str>,
}
