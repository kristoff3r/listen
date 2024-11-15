use diesel::{deserialize, pg, serialize, sql_types};

use crate::Uuid;

impl<T> deserialize::FromSql<sql_types::Uuid, pg::Pg> for Uuid<T> {
    fn from_sql(value: pg::PgValue<'_>) -> deserialize::Result<Self> {
        uuid::Uuid::from_sql(value).map(Into::into)
    }
}

impl<T> serialize::ToSql<sql_types::Uuid, pg::Pg> for Uuid<T> {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, pg::Pg>) -> serialize::Result {
        <uuid::Uuid as serialize::ToSql<sql_types::Uuid, pg::Pg>>::to_sql(&self.uuid, out)
    }
}
