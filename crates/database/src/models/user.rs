use diesel::{delete, insert_into, prelude::*, update, QueryDsl, Selectable, SelectableHelper};
use diesel_async::{
    scoped_futures::ScopedFutureExt, AsyncConnection, AsyncPgConnection, RunQueryDsl,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use typed_uuid::Uuid;

use super::OidcMapping;

pub type UserId = Uuid<User>;

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(primary_key(user_id))]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: UserId,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub last_login: OffsetDateTime,
    pub last_activity: OffsetDateTime,
    pub email: String,
    pub handle: String,
    pub profile_picture_url: String,
    pub is_approved: bool,
    pub is_admin: bool,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct NewUser<'a> {
    pub email: &'a str,
    pub handle: &'a str,
    pub profile_picture_url: &'a str,
}

impl User {
    pub async fn create(
        conn: &mut AsyncPgConnection,
        handle: &str,
        email: &str,
        profile_picture_url: &str,
        oidc_issuer_url: &str,
        oidc_issuer_id: &str,
    ) -> anyhow::Result<(Self, OidcMapping)> {
        conn.transaction::<_, _, _>(|conn| {
            async move {
                let user =
                    Self::create_without_oidc(conn, handle, email, profile_picture_url).await?;

                let oidc_mapping =
                    OidcMapping::create(conn, user.user_id, oidc_issuer_url, oidc_issuer_id)
                        .await?;

                Ok((user, oidc_mapping))
            }
            .scope_boxed()
        })
        .await
    }

    pub async fn create_without_oidc(
        conn: &mut AsyncPgConnection,
        handle: &str,
        email: &str,
        profile_picture_url: &str,
    ) -> anyhow::Result<Self> {
        use crate::schema::users::dsl as u;

        let result = insert_into(u::users)
            .values(NewUser {
                handle,
                email,
                profile_picture_url,
            })
            .get_result(conn)
            .await?;

        Ok(result)
    }

    pub async fn get_by_id(conn: &mut AsyncPgConnection, user_id: UserId) -> anyhow::Result<Self> {
        use crate::schema::users::dsl as u;
        let result = u::users.find(user_id).first(conn).await?;
        Ok(result)
    }

    pub async fn get_by_email(conn: &mut AsyncPgConnection, email: &str) -> anyhow::Result<Self> {
        use crate::schema::users::dsl as u;

        let result = u::users.filter(u::email.eq(email)).first(conn).await?;

        Ok(result)
    }

    pub async fn get_by_oidc(
        conn: &mut AsyncPgConnection,
        oidc_issuer_url: &str,
        oidc_issuer_id: &str,
    ) -> anyhow::Result<(Self, OidcMapping)> {
        use crate::schema::{oidc_mapping::dsl as m, users::dsl as u};

        let result = m::oidc_mapping
            .inner_join(u::users)
            .filter(m::oidc_issuer_url.eq(oidc_issuer_url))
            .filter(m::oidc_issuer_id.eq(oidc_issuer_id))
            .select((Self::as_select(), OidcMapping::as_select()))
            .first(conn)
            .await?;

        Ok(result)
    }

    pub async fn list(conn: &mut AsyncPgConnection) -> anyhow::Result<Vec<Self>> {
        use crate::schema::users::dsl as u;
        let results = u::users.get_results(conn).await?;
        Ok(results)
    }

    pub async fn update_handle(
        conn: &mut AsyncPgConnection,
        user_id: UserId,
        handle: &str,
    ) -> anyhow::Result<()> {
        use crate::schema::users::dsl as u;

        update(u::users)
            .filter(u::user_id.eq(user_id))
            .set((
                u::updated_at.eq(OffsetDateTime::now_utc()),
                u::handle.eq(handle),
            ))
            .execute(conn)
            .await?;

        Ok(())
    }

    pub async fn update_email(
        conn: &mut AsyncPgConnection,
        user_id: UserId,
        email: &str,
    ) -> anyhow::Result<()> {
        use crate::schema::users::dsl as u;

        update(u::users)
            .filter(u::user_id.eq(user_id))
            .set((
                u::updated_at.eq(OffsetDateTime::now_utc()),
                u::email.eq(email),
            ))
            .execute(conn)
            .await?;

        Ok(())
    }

    pub async fn update_profile_picture_url(
        conn: &mut AsyncPgConnection,
        user_id: UserId,
        profile_picture_url: &str,
    ) -> anyhow::Result<()> {
        use crate::schema::users::dsl as u;

        update(u::users)
            .filter(u::user_id.eq(user_id))
            .set((
                u::updated_at.eq(OffsetDateTime::now_utc()),
                u::profile_picture_url.eq(profile_picture_url),
            ))
            .execute(conn)
            .await?;

        Ok(())
    }

    pub async fn update_is_approved(
        conn: &mut AsyncPgConnection,
        user_id: UserId,
        is_approved: bool,
    ) -> anyhow::Result<()> {
        use crate::schema::users::dsl as u;

        update(u::users)
            .filter(u::user_id.eq(user_id))
            .set((
                u::updated_at.eq(OffsetDateTime::now_utc()),
                u::is_approved.eq(is_approved),
            ))
            .execute(conn)
            .await?;

        Ok(())
    }

    pub async fn update_is_admin(
        conn: &mut AsyncPgConnection,
        user_id: UserId,
        is_admin: bool,
    ) -> anyhow::Result<()> {
        use crate::schema::users::dsl as u;

        update(u::users)
            .filter(u::user_id.eq(user_id))
            .set((
                u::updated_at.eq(OffsetDateTime::now_utc()),
                u::is_admin.eq(is_admin),
            ))
            .execute(conn)
            .await?;

        Ok(())
    }

    pub async fn update_last_activity(
        conn: &mut AsyncPgConnection,
        user_id: UserId,
    ) -> anyhow::Result<()> {
        use crate::schema::users::dsl as u;

        update(u::users)
            .filter(u::user_id.eq(user_id))
            .set((
                u::updated_at.eq(OffsetDateTime::now_utc()),
                u::last_activity.eq(OffsetDateTime::now_utc()),
            ))
            .execute(conn)
            .await?;

        Ok(())
    }

    pub async fn update_last_login(
        conn: &mut AsyncPgConnection,
        user_id: UserId,
    ) -> anyhow::Result<()> {
        use crate::schema::users::dsl as u;

        update(u::users)
            .filter(u::user_id.eq(user_id))
            .set((
                u::updated_at.eq(OffsetDateTime::now_utc()),
                u::last_login.eq(OffsetDateTime::now_utc()),
            ))
            .execute(conn)
            .await?;

        Ok(())
    }

    pub async fn delete(conn: &mut AsyncPgConnection, user_id: UserId) -> anyhow::Result<()> {
        use crate::schema::users::dsl as u;

        delete(u::users)
            .filter(u::user_id.eq(user_id))
            .execute(conn)
            .await?;

        Ok(())
    }
}
