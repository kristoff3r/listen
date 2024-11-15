use std::{net::SocketAddr, path::PathBuf, time::Duration};

use anyhow::Context;
use axum::{
    body::Body,
    extract::{Path, Request, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use database::MIGRATIONS;
use fileserv::file_and_error_handler;
use leptos::{logging::log, *};
use leptos_axum::{generate_route_list, LeptosRoutes};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    timeout::TimeoutLayer,
    trace::{self, TraceLayer},
};
use tracing::{info, warn, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ui::{
    server_state::{ServerState, VideosDir},
    App,
};

use crate::db::setup_database_pool;

pub mod db;
pub mod error;
pub mod fileserv;
pub mod handlers;
pub mod ws;

pub use db::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");

    info!("Connecting to database {db_url}");
    let pool = setup_database_pool(&db_url).context("setup datbase pool")?;

    {
        let mut retries = 0;
        let mut conn = loop {
            match pool.get().await {
                Ok(conn) => break conn,
                Err(e) => {
                    retries += 1;
                    if retries > 3 {
                        panic!("Could not connect to database: {e}");
                    }
                    warn!("Database not ready: {e}");
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            };
        };
        MIGRATIONS.run_pending_migrations(&mut conn).await?;
        info!("Finished running migrations");
    }

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    let state = ServerState {
        pool,
        leptos_options,
        videos_dir: VideosDir(
            PathBuf::from(std::env::var("VIDEOS_DIR").unwrap_or_else(|_| "videos".to_string()))
                .canonicalize()
                .unwrap(),
        ),
    };

    info!("listening on {}", addr);
    info!("video dir: {}", state.videos_dir.display());

    {
        let videos_dir = state.videos_dir.clone();
        let pool = state.pool.clone();
        tokio::task::spawn(async move {
            handlers::download::handle_download_queue(pool, videos_dir).await
        });
    }

    let app = routes().with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

    Ok(())
}

fn routes() -> Router<ServerState> {
    Router::new()
        .route(
            "/api/leptos/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .route("/health_check", get(|| async { "" }))
        .route("/api/videos/:id", get(handlers::videos::get_video))
        .route("/api/videos/:id/play", get(handlers::videos::play_video))
        .route(
            "/api/download",
            post(handlers::download::add_video_to_queue),
        )
        .leptos_routes_with_handler(generate_route_list(App), get(leptos_routes_handler))
        .fallback(file_and_error_handler)
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                        .on_failure(trace::DefaultOnFailure::new().level(Level::ERROR))
                        .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
                )
                .layer(TimeoutLayer::new(Duration::from_secs(3))),
        )
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn server_fn_handler(
    State(app_state): State<ServerState>,
    path: Path<String>,
    request: Request<Body>,
) -> impl IntoResponse {
    log!("{:?}", path);

    leptos_axum::handle_server_fns_with_context(
        move || {
            provide_context(app_state.clone());
            provide_context(app_state.pool.clone());
        },
        request,
    )
    .await
}

pub async fn leptos_routes_handler(
    State(app_state): State<ServerState>,
    State(option): State<leptos::LeptosOptions>,
    request: Request<Body>,
) -> axum::response::Response {
    let handler = leptos_axum::render_app_async_with_context(
        option.clone(),
        move || {
            provide_context(app_state.clone());
            provide_context(app_state.pool.clone());
        },
        move || view! { <App/> },
    );

    handler(request).await.into_response()
}
