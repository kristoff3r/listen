use api::User;
use axum::{extract::State, Json};
use time::OffsetDateTime;

use crate::{
    error::{ListenErrorExt, Result},
    PgPool,
};

pub async fn get_profile(State(pool): State<PgPool>) -> Result<Json<api::User>> {
    let mut _conn = pool.get().await.with_internal_server_error()?;

    Ok(Json(User {
        profile_picture_url: "https://avatars.githubusercontent.com/u/160317?v=4".to_string(),
        is_admin: true,
        is_approved: true,
        user_id: Default::default(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        last_login: OffsetDateTime::now_utc(),
        last_activity: OffsetDateTime::now_utc(),
        email: "test@test.com".to_string(),
        handle: "UserHandle".to_string(),
    }))
}
