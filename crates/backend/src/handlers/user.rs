use axum::{Extension, Json};

pub async fn get_profile(Extension(user): Extension<database::models::User>) -> Json<api::User> {
    Json(user.into())
}
