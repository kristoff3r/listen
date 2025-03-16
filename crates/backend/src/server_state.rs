use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use api::CrowdId;
use axum::extract::FromRef;
use diesel_async::{pooled_connection::deadpool::Pool, AsyncPgConnection};
use leptos::config::LeptosOptions;
use tokio::sync::{broadcast, mpsc};

pub type CrowdMap = Arc<dashmap::DashMap<CrowdId, CrowdState>>;

pub struct CrowdState {
    pub crowd_id: CrowdId,
    pub started: time::UtcDateTime,
    pub name: String,
    pub command_sender: mpsc::Sender<(time::UtcDateTime, api::CrowdParticipantCommand)>,
    pub update_receiver: broadcast::Receiver<(time::UtcDateTime, api::CrowdPlayerUpdate)>,
}

/// Derive FromRef to allow multiple items in state, using Axum’s
/// SubStates pattern.
#[derive(FromRef, Clone)]
pub struct ServerState {
    pub leptos_options: LeptosOptions,
    pub pool: Pool<AsyncPgConnection>,
    pub videos_dir: VideosDir,
    pub jwt_encoding_key: jsonwebtoken::EncodingKey,
    pub jwt_decoding_key: jsonwebtoken::DecodingKey,
    pub google_oidc_client: crate::oidc::OidcClient,
    pub crowd_map: CrowdMap,
}

#[derive(Clone, Debug)]
pub struct VideosDir(pub PathBuf);

impl std::ops::Deref for VideosDir {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
