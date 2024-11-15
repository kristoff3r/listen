use diesel::{delete, insert_into, prelude::*, update, QueryDsl, Selectable, SelectableHelper};
use diesel_async::{
    scoped_futures::ScopedFutureExt, AsyncConnection, AsyncPgConnection, RunQueryDsl,
};
use openidconnect::{CsrfToken, Nonce, PkceCodeVerifier};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use typed_uuid::Uuid;

pub type UserId = Uuid<User>;
pub type OidcMappingId = Uuid<OidcMapping>;
pub type UserSessionId = Uuid<UserSession>;

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
pub struct NewUser<'a> {
    pub email: &'a str,
    pub handle: &'a str,
    pub profile_picture_url: &'a str,
}

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(primary_key(oidc_mapping_id))]
#[diesel(table_name = crate::schema::oidc_mapping)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewOidcMapping<'a> {
    pub oidc_issuer_url: &'a str,
    pub oidc_issuer_id: &'a str,
    pub user_id: UserId,
}

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(primary_key(user_session_id))]
#[diesel(table_name = crate::schema::user_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub user_session_id: UserSessionId,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    // This info is generated by the oidc client and is needed
    // to validate that the user has successfully logged in.
    // Delete after logging in
    pub oidc_issuer_url: Option<String>,
    pub csrf_token: Option<String>,
    pub nonce: Option<String>,
    pub pkce_code_verifier: Option<String>,
    //
    // // Only set after logged in
    pub user_id: Option<UserId>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::user_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewUserSession<'a> {
    pub oidc_issuer_url: &'a str,
    pub csrf_token: &'a str,
    pub nonce: &'a str,
    pub pkce_code_verifier: &'a str,
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
                use crate::schema::oidc_mapping::dsl as m;
                use crate::schema::users::dsl as u;

                let user: User = insert_into(u::users)
                    .values(NewUser {
                        handle,
                        email,
                        profile_picture_url,
                    })
                    .get_result(conn)
                    .await?;

                let oidc_mapping = insert_into(m::oidc_mapping)
                    .values(NewOidcMapping {
                        user_id: user.user_id,
                        oidc_issuer_url,
                        oidc_issuer_id,
                    })
                    .get_result(conn)
                    .await?;

                Ok((user, oidc_mapping))
            }
            .scope_boxed()
        })
        .await
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
        use crate::schema::oidc_mapping::dsl as m;
        use crate::schema::users::dsl as u;

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

impl OidcMapping {
    pub async fn create(
        conn: &mut AsyncPgConnection,
        user_id: UserId,
        oidc_issuer_url: &str,
        oidc_issuer_id: &str,
    ) -> anyhow::Result<()> {
        use crate::schema::oidc_mapping::dsl as m;

        insert_into(m::oidc_mapping)
            .values(NewOidcMapping {
                user_id,
                oidc_issuer_url,
                oidc_issuer_id,
            })
            .execute(conn)
            .await?;

        Ok(())
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

impl UserSession {
    pub async fn create(
        conn: &mut AsyncPgConnection,
        oidc_issuer_url: &str,
        csrf_token: CsrfToken,
        nonce: Nonce,
        pkce_code_verifier: PkceCodeVerifier,
    ) -> anyhow::Result<UserSessionId> {
        use crate::schema::user_sessions::dsl as s;

        let session_id = insert_into(s::user_sessions)
            .values(NewUserSession {
                oidc_issuer_url,
                csrf_token: csrf_token.secret(),
                nonce: nonce.secret(),
                pkce_code_verifier: pkce_code_verifier.secret(),
            })
            .returning(s::user_session_id)
            .get_result(conn)
            .await?;

        Ok(session_id)
    }

    pub async fn get_by_id(
        conn: &mut AsyncPgConnection,
        user_session_id: UserSessionId,
    ) -> anyhow::Result<Self> {
        use crate::schema::user_sessions::dsl as s;
        let result = s::user_sessions.find(user_session_id).first(conn).await?;
        Ok(result)
    }

    pub async fn list_by_user_id(
        conn: &mut AsyncPgConnection,
        user_id: UserId,
    ) -> anyhow::Result<Vec<Self>> {
        use crate::schema::user_sessions::dsl as s;

        let results = s::user_sessions
            .filter(s::user_id.eq(user_id))
            .get_results(conn)
            .await?;

        Ok(results)
    }

    pub async fn update_after_completed_login(
        conn: &mut AsyncPgConnection,
        user_session_id: UserSessionId,
        user_id: UserId,
    ) -> anyhow::Result<()> {
        use crate::schema::user_sessions::dsl as s;

        update(s::user_sessions)
            .filter(s::user_session_id.eq(user_session_id))
            .set((
                s::updated_at.eq(OffsetDateTime::now_utc()),
                s::user_id.eq(user_id),
                s::oidc_issuer_url.eq(None::<String>),
                s::csrf_token.eq(None::<String>),
                s::nonce.eq(None::<String>),
                s::pkce_code_verifier.eq(None::<String>),
            ))
            .execute(conn)
            .await?;
        Ok(())
    }

    pub async fn delete(
        conn: &mut AsyncPgConnection,
        user_session_id: UserSessionId,
    ) -> anyhow::Result<()> {
        use crate::schema::user_sessions::dsl as s;

        delete(s::user_sessions)
            .filter(s::user_session_id.eq(user_session_id))
            .execute(conn)
            .await?;
        Ok(())
    }
}
