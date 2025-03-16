use axum::{extract::State, routing::get, Json, Router};
use participant::ws_handler_participant;
use player::ws_handler_player;

use crate::server_state::{CrowdMap, ServerState};

pub mod participant;
pub mod player;

enum ShouldContinue {
    Stop,
    Continue,
}

pub fn routes() -> Router<ServerState> {
    Router::new()
        .route("/player", get(ws_handler_player))
        .route("/participant", get(ws_handler_participant))
        .route("/list", get(list))
}

async fn list(State(crowd_map): State<CrowdMap>) -> Json<Vec<api::CrowdListEntry>> {
    Json(
        crowd_map
            .iter()
            .map(|entry| api::CrowdListEntry {
                started_time: entry.started,
                crowd_id: entry.crowd_id,
                name: entry.name.clone(),
                participant_count: entry.command_sender.strong_count().saturating_sub(1),
            })
            .collect(),
    )
}
