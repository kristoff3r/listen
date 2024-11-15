use std::path::{Path, PathBuf};

use axum::extract::FromRef;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection};
use leptos::config::LeptosOptions;

/// Derive FromRef to allow multiple items in state, using Axum’s
/// SubStates pattern.
#[derive(FromRef, Clone)]
pub struct ServerState {
    pub leptos_options: LeptosOptions,
    pub pool: Pool<AsyncPgConnection>,
    pub videos_dir: VideosDir,
}

#[derive(Clone, Debug)]
pub struct VideosDir(pub PathBuf);

impl std::ops::Deref for VideosDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
