use std::net::SocketAddr;

use crate::server_state::{CrowdMap, CrowdState};

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

struct ParticipantConnectionState {
    command_sender: tokio::sync::mpsc::Sender<(time::UtcDateTime, api::CrowdParticipantCommand)>,
    update_receiver: tokio::sync::broadcast::Receiver<(time::UtcDateTime, api::CrowdPlayerUpdate)>,
    websocket: WebSocket,
    interested_after: InterestedAfterTimes,
}

// When are we next interested in updates to these state elements?
struct InterestedAfterTimes {
    playback_position: time::UtcDateTime,
    is_paused: time::UtcDateTime,
    speed: time::UtcDateTime,
    queue: time::UtcDateTime,
}

impl Default for InterestedAfterTimes {
    fn default() -> Self {
        Self {
            playback_position: time::UtcDateTime::UNIX_EPOCH,
            is_paused: time::UtcDateTime::UNIX_EPOCH,
            speed: time::UtcDateTime::UNIX_EPOCH,
            queue: time::UtcDateTime::UNIX_EPOCH,
        }
    }
}

impl ParticipantConnectionState {
    async fn handle(mut self) {
        loop {
            let should_continue = tokio::select! {
                msg = self.websocket.recv() => self.handle_websocket_message(msg).await,
                update = self.update_receiver.recv() =>self.handle_player_update(update).await,
            };
            let Ok(ShouldContinue::Continue) = should_continue else {
                break;
            };
        }
        let _ = self.websocket.close().await;
    }

    #[tracing::instrument(skip(self), err(Debug))]
    async fn handle_player_update(
        &mut self,
        update: Result<
            (time::UtcDateTime, api::CrowdPlayerUpdate),
            tokio::sync::broadcast::error::RecvError,
        >,
    ) -> anyhow::Result<ShouldContinue> {
        let (time, update) = update.context("Broadcast died?")?;
        let is_interested = match &update {
            api::CrowdPlayerUpdate::PlaybackPosition(_) => {
                time >= self.interested_after.playback_position
            }
            api::CrowdPlayerUpdate::IsPaused(_) => time >= self.interested_after.is_paused,
            api::CrowdPlayerUpdate::Speed(_) => time >= self.interested_after.speed,
            api::CrowdPlayerUpdate::Queue { .. } => time >= self.interested_after.queue,
        };

        let update = serde_json::to_string(&update)?;

        if is_interested {
            self.websocket
                .send(axum::extract::ws::Message::Text(update))
                .await?;
        }

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
                let msg: api::CrowdParticipantCommand = serde_json::from_str(&msg)?;
                let now = time::UtcDateTime::now();
                match &msg {
                    api::CrowdParticipantCommand::SetPlaybackPosition(_) => {
                        self.interested_after.playback_position = now
                    }
                    api::CrowdParticipantCommand::SetIsPaused(_) => {
                        self.interested_after.is_paused = now
                    }
                    api::CrowdParticipantCommand::SetSpeed(_) => self.interested_after.speed = now,
                    api::CrowdParticipantCommand::GoTo(_)
                    | api::CrowdParticipantCommand::AddToQueue(_)
                    | api::CrowdParticipantCommand::MoveInQueue { .. }
                    | api::CrowdParticipantCommand::DeleteFromQueue(_) => {
                        self.interested_after.queue = now
                    }
                }
                self.command_sender
                    .send((now, msg))
                    .await
                    .context("Command sender closed, not more connection?")?;
                Ok(ShouldContinue::Continue)
            }
            msg => anyhow::bail!("Unexpected websocket message: {msg:?}"),
        }
    }
}

pub async fn ws_handler_participant(
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
    info!("`{user_agent}` at {addr} connected to participant websocket.");
    ws.on_upgrade(move |socket| async move {
        let _ = handle_participant_websocket(crowd_map, socket, addr).await;
    })
}

#[tracing::instrument(skip(crowd_map, websocket), err(Debug))]
async fn handle_participant_websocket(
    crowd_map: CrowdMap,
    mut websocket: WebSocket,
    addr: SocketAddr,
) -> anyhow::Result<()> {
    let Some(msg) = websocket.recv().await else {
        bail!("Could not receive initial message");
    };

    let crowd_id = match msg {
        Ok(Message::Text(crowd_id)) => crowd_id,
        Ok(msg) => {
            let _ = websocket.close().await;
            bail!("Unexpected initial message: {msg:?}");
        }
        Err(e) => {
            let _ = websocket.close().await;
            bail!("Could not receive initial message: {e:?}");
        }
    };
    let crowd_id: CrowdId = crowd_id.trim().parse()?;
    let Some(crowd_state) = crowd_map.get(&crowd_id) else {
        bail!("Could not find crown with id {crowd_id}");
    };
    let CrowdState {
        name,
        command_sender,
        update_receiver,
        ..
    } = &*crowd_state;

    let span = tracing::info_span!("crowd participant", %crowd_id, %name);

    {
        let _span = span.enter();
        tracing::info!("New crowd participant");
    }

    let participant_state = ParticipantConnectionState {
        command_sender: command_sender.clone(),
        update_receiver: update_receiver.resubscribe(),
        websocket,
        interested_after: InterestedAfterTimes::default(),
    };

    participant_state.handle().instrument(span).await;

    Ok(())
}
