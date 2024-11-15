use diesel::{delete, insert_into, prelude::*, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use time::OffsetDateTime;
use typed_uuid::Uuid;

use super::UserId;

pub type OidcMappingId = Uuid<OidcMapping>;

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(primary_key(oidc_mapping_id))]
#[diesel(table_name = crate::schema::oidc_mapping)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Clone, Debug)]
pub struct OidcMapping {
    pub oidc_mapping_id: OidcMappingId,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub oidc_issuer_url: String,
    pub oidc_issuer_id: String,
    pub user_id: UserId,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::oidc_mapping)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Clone, Debug)]
struct NewOidcMapping<'a> {
    pub oidc_issuer_url: &'a str,
    pub oidc_issuer_id: &'a str,
    pub user_id: UserId,
}

impl OidcMapping {
    pub async fn create(
        conn: &mut AsyncPgConnection,
        user_id: UserId,
        oidc_issuer_url: &str,
        oidc_issuer_id: &str,
    ) -> anyhow::Result<Self> {
        use crate::schema::oidc_mapping::dsl as m;

        let result = insert_into(m::oidc_mapping)
            .values(NewOidcMapping {
                user_id,
                oidc_issuer_url,
                oidc_issuer_id,
            })
            .get_result(conn)
            .await?;

        Ok(result)
    }

    pub async fn get_by_id(
        conn: &mut AsyncPgConnection,
        oidc_mapping_id: OidcMappingId,
    ) -> anyhow::Result<Self> {
        use crate::schema::oidc_mapping::dsl as m;
        let result = m::oidc_mapping.find(oidc_mapping_id).first(conn).await?;
        Ok(result)
    }

    pub async fn list_by_user_id(
        conn: &mut AsyncPgConnection,
        user_id: UserId,
    ) -> anyhow::Result<Vec<Self>> {
        use crate::schema::oidc_mapping::dsl as m;

        let results = m::oidc_mapping
            .filter(m::user_id.eq(user_id))
            .get_results(conn)
            .await?;

        Ok(results)
    }

    pub async fn delete(
        conn: &mut AsyncPgConnection,
        oidc_mapping_id: OidcMappingId,
    ) -> anyhow::Result<()> {
        use crate::schema::oidc_mapping::dsl as m;

        delete(m::oidc_mapping)
            .filter(m::oidc_mapping_id.eq(oidc_mapping_id))
            .execute(conn)
            .await?;

        Ok(())
    }
}
