use api::{OidcMappingId, UserId};
use diesel::{delete, insert_into, prelude::*, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use openidconnect::{IssuerUrl, SubjectIdentifier};
use structural_convert::StructuralConvert;
use time::OffsetDateTime;

use super::{Result, User};

#[derive(
    Clone, Debug, PartialEq, Queryable, Selectable, Identifiable, Associations, StructuralConvert,
)]
#[diesel(primary_key(oidc_mapping_id))]
#[diesel(table_name = crate::schema::oidc_mapping)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(User))]
#[convert(into(api::OidcMapping))]
pub struct OidcMapping {
    pub oidc_mapping_id: OidcMappingId,
    pub oidc_issuer_url: String,
    pub oidc_issuer_id: String,
    pub user_id: UserId,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
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
        oidc_issuer_url: &IssuerUrl,
        oidc_issuer_id: &SubjectIdentifier,
    ) -> Result<Self> {
        use crate::schema::oidc_mapping::dsl as m;

        let result = insert_into(m::oidc_mapping)
            .values(NewOidcMapping {
                user_id,
                oidc_issuer_url: oidc_issuer_url.as_str(),
                oidc_issuer_id: oidc_issuer_id.as_str(),
            })
            .get_result(conn)
            .await?;

        Ok(result)
    }

    pub async fn get_by_id(
        conn: &mut AsyncPgConnection,
        oidc_mapping_id: OidcMappingId,
    ) -> Result<Option<Self>> {
        use crate::schema::oidc_mapping::dsl as m;
        let result = m::oidc_mapping
            .find(oidc_mapping_id)
            .first(conn)
            .await
            .optional()?;
        Ok(result)
    }

    pub async fn list_by_user_id(
        conn: &mut AsyncPgConnection,
        user_id: UserId,
    ) -> Result<Vec<Self>> {
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
    ) -> Result<()> {
        use crate::schema::oidc_mapping::dsl as m;

        delete(m::oidc_mapping)
            .filter(m::oidc_mapping_id.eq(oidc_mapping_id))
            .execute(conn)
            .await?;

        Ok(())
    }
}
