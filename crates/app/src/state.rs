use axum::extract::FromRef;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection};
use leptos::LeptosOptions;

/// Derive FromRef to allow multiple items in state, using Axum’s
/// SubStates pattern.
#[derive(FromRef, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub pool: Pool<AsyncPgConnection>,
}
