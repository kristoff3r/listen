use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        ConnectInfo,
    },
    response::IntoResponse,
};
use axum_extra::TypedHeader;
use std::net::SocketAddr;
use tracing::info;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    info!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, _who: SocketAddr) {
    while let Some(msg) = socket.recv().await {
        let msg = msg.unwrap();
        println!("{msg:?}");
    }
}

