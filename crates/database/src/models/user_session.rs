use diesel::{delete, insert_into, prelude::*, update, QueryDsl, Selectable};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use openidconnect::{CsrfToken, Nonce, PkceCodeVerifier};
use time::OffsetDateTime;
use typed_uuid::Uuid;

use super::UserId;

pub type UserSessionId = Uuid<UserSession>;

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(primary_key(user_session_id))]
#[diesel(table_name = crate::schema::user_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
struct NewUserSession<'a> {
    pub oidc_issuer_url: &'a str,
    pub csrf_token: &'a str,
    pub nonce: &'a str,
    pub pkce_code_verifier: &'a str,
}

impl UserSession {
    pub async fn create(
        conn: &mut AsyncPgConnection,
        oidc_issuer_url: &str,
        csrf_token: CsrfToken,
        nonce: Nonce,
        pkce_code_verifier: PkceCodeVerifier,
    ) -> anyhow::Result<Self> {
        use crate::schema::user_sessions::dsl as s;

        let result = insert_into(s::user_sessions)
            .values(NewUserSession {
                oidc_issuer_url,
                csrf_token: csrf_token.secret(),
                nonce: nonce.secret(),
                pkce_code_verifier: pkce_code_verifier.secret(),
            })
            .get_result(conn)
            .await?;

        Ok(result)
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