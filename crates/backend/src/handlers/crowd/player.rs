use std::net::SocketAddr;

use crate::server_state::CrowdMap;

use super::ShouldContinue;
use anyhow::{bail, Context};
use api::CrowdId;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use tracing::{info, Instrument};

struct PlayerConnectionState {
    command_receiver:
        tokio::sync::mpsc::Receiver<(time::UtcDateTime, api::CrowdParticipantCommand)>,
    update_publisher: tokio::sync::broadcast::Sender<(time::UtcDateTime, api::CrowdPlayerUpdate)>,
    websocket: WebSocket,
}

impl PlayerConnectionState {
    async fn handle(mut self) {
        loop {
            let should_continue = tokio::select! {
                msg = self.websocket.recv() => self.handle_websocket_message(msg).await,
                command = self.command_receiver.recv() =>self.handle_participant_command(command).await,
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                    tracing::warn!("Player connection closed");
                    Ok(ShouldContinue::Stop)
                }
            };
            let Ok(ShouldContinue::Continue) = should_continue else {
                break;
            };
        }
        let _ = self.websocket.close().await;
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn handle_participant_command(
        &mut self,
        command: Option<(time::UtcDateTime, api::CrowdParticipantCommand)>,
    ) -> anyhow::Result<ShouldContinue> {
        let Some(command) = command else {
            tracing::info!("No more commands");
            return Ok(ShouldContinue::Stop);
        };
        let command = serde_json::to_string(&command)?;
        self.websocket
            .send(axum::extract::ws::Message::Text(command))
            .await?;
        Ok(ShouldContinue::Continue)
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn handle_websocket_message(
        &mut self,
        msg: Option<Result<axum::extract::ws::Message, axum::Error>>,
    ) -> anyhow::Result<ShouldContinue> {
        let Some(msg) = msg else {
            tracing::info!("No more websocket messages");
            return Ok(ShouldContinue::Stop);
        };
        let msg = msg.context("Error while receiving websocket message")?;
        match msg {
            axum::extract::ws::Message::Text(msg) => {
                let msg: (time::UtcDateTime, api::CrowdPlayerUpdate) = serde_json::from_str(&msg)?;
                if let api::CrowdPlayerUpdate::Ping = &msg.1 {
                    let command = serde_json::to_string(&api::CrowdParticipantCommand::Ping)?;
                    self.websocket.send(Message::Text(command)).await?;
                } else {
                    self.update_publisher
                        .send(msg)
                        .context("Crowd closed, no more receivers?")?;
                }
                Ok(ShouldContinue::Continue)
            }
            msg => anyhow::bail!("Unexpected websocket message: {msg:?}"),
        }
    }
}

pub async fn ws_handler_player(
    State(crowd_map): State<CrowdMap>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    info!("`{user_agent}` at {addr} connected to player websocket.");
    ws.on_upgrade(move |socket| async move {
        let _ = handle_player_websocket(crowd_map, socket, addr).await;
    })
}

#[tracing::instrument(skip(crowd_map, websocket), err(Debug))]
async fn handle_player_websocket(
    crowd_map: CrowdMap,
    mut websocket: WebSocket,
    addr: SocketAddr,
) -> anyhow::Result<()> {
    let (command_sender, command_receiver) = tokio::sync::mpsc::channel(50);
    let (update_publisher, update_receiver) = tokio::sync::broadcast::channel(50);
    let Some(msg) = websocket.recv().await else {
        let _ = websocket.close().await;
        bail!("Could not receive initial message");
    };

    let name = match msg {
        Ok(Message::Text(name)) => name,
        Ok(msg) => {
            let _ = websocket.close().await;
            bail!("Unexpected initial message: {msg:?}");
        }
        Err(e) => {
            let _ = websocket.close().await;
            bail!("Could not receive initial message: {e:?}");
        }
    };
    let name = name.trim().to_string();

    let mut crowd_id;
    let span;
    loop {
        crowd_id = CrowdId::new_random();
        let dashmap::Entry::Vacant(entry) = crowd_map.entry(crowd_id) else {
            continue;
        };
        span = tracing::info_span!("crowd player", %crowd_id, %name);
        entry.insert(crate::server_state::CrowdState {
            crowd_id,
            started: time::UtcDateTime::now(),
            name,
            command_sender,
            update_receiver,
        });
        break;
    }

    {
        let _span = span.enter();
        tracing::info!("New crowd player");
    }

    let player_state = PlayerConnectionState {
        command_receiver,
        update_publisher,
        websocket,
    };

    player_state.handle().instrument(span).await;
    crowd_map.remove(&crowd_id);
    Ok(())
}
