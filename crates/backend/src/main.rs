use std::{net::SocketAddr, path::PathBuf, time::Duration};

use anyhow::Context;
use api::ApiError;
use axum::{
    middleware::{map_request, map_request_with_state},
    routing::{get, post},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use database::MIGRATIONS;
use error::ListenError;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    timeout::TimeoutLayer,
    trace::{self, TraceLayer},
};
use tracing::{info, warn, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ui::App;

use crate::{
    db::setup_database_pool,
    server_state::{ServerState, VideosDir},
};

mod app_env_vars;
mod csrf_protection;
pub mod db;
pub mod error;
pub mod handlers;
mod oidc;
mod server_state;
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

    let app_env_vars = app_env_vars::AppEnvVars::load_from_env()?;

    let google_client_id = openidconnect::ClientId::new(app_env_vars.google_oidc_client_id);
    let google_client_secret =
        openidconnect::ClientSecret::new(app_env_vars.google_oidc_client_secret);
    let google_issuer_url =
        openidconnect::IssuerUrl::new("https://accounts.google.com".to_string())
            .context("Unable to create issue url")?;
    let redirect_url = openidconnect::RedirectUrl::new(
        "https://dev.listen.pwnies.dk:3000/auth/callback".to_string(),
    )?;
    let google_oidc_client = oidc::OidcClient::new(
        google_issuer_url,
        google_client_id,
        google_client_secret,
        redirect_url,
    );

    info!(
        "Connecting to database {database_url}",
        database_url = app_env_vars.database_url
    );
    let pool = setup_database_pool(&app_env_vars.database_url).context("setup datbase pool")?;

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

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    let jwt_encoding_key =
        jsonwebtoken::EncodingKey::from_secret(app_env_vars.jwt_secret.as_bytes());
    let jwt_decoding_key =
        jsonwebtoken::DecodingKey::from_secret(app_env_vars.jwt_secret.as_bytes());
    let state = ServerState {
        pool,
        leptos_options,
        videos_dir: VideosDir(
            PathBuf::from(std::env::var("VIDEOS_DIR").unwrap_or_else(|_| "videos".to_string()))
                .canonicalize()
                .unwrap(),
        ),
        jwt_encoding_key,
        jwt_decoding_key,
        google_oidc_client,
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

    let app = routes(state.clone());

    if cfg!(feature = "local-https") {
        serve_https(addr, app).await.unwrap();
    } else {
        serve_http(addr, app).await.unwrap();
    }

    Ok(())
}

async fn serve_https(addr: SocketAddr, app: Router<()>) -> anyhow::Result<()> {
    let app = app.into_make_service_with_connect_info::<SocketAddr>();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Could not install default crypto provider for rustls");
    let config = RustlsConfig::from_pem_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("dev-certificates")
            .join("dev.listen.pwnies.dk.crt"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("dev-certificates")
            .join("dev.listen.pwnies.dk.key"),
    )
    .await.context(
        "Could not create rustls config. Did you run the script to generate the self-signed certificates"
    )?;

    let handle = axum_server::Handle::new();
    tokio::spawn({
        let handle = handle.clone();
        async move {
            shutdown_signal().await;
            tracing::trace!("received graceful shutdown signal. Telling tasks to shutdown");
            handle.graceful_shutdown(Some(Duration::from_secs(1)));
        }
    });

    axum_server::bind_rustls(addr, config)
        .handle(handle)
        .serve(app)
        .await?;
    Ok(())
}

async fn serve_http(addr: SocketAddr, app: Router<()>) -> anyhow::Result<()> {
    let app = app.into_make_service_with_connect_info::<SocketAddr>();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn routes(state: ServerState) -> Router {
    let routes = generate_route_list(App);
    Router::new()
        .nest("/api", api_routes(state.clone()))
        .route("/health_check", get(|| async { "" }))
        // .leptos_routes(generate_route_list(App), get(leptos_routes_handler))
        .leptos_routes(&state, routes, {
            let leptos_options = state.leptos_options.clone();
            move || ui::shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<ServerState, _>(
            ui::shell,
        ))
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
        .with_state(state)
}

fn api_routes(state: ServerState) -> Router<ServerState> {
    let csrf_layer = map_request(csrf_protection::csrf_layer);
    let auth_required_layer =
        map_request_with_state(state.clone(), handlers::auth::auth_required_layer);
    let user_session_layer = map_request_with_state(state, handlers::auth::user_session_layer);

    // Routes will full protection: CSRF + authentication required
    let api_routes = Router::new()
        .route("/videos", get(handlers::videos::list_videos))
        .route("/videos/:id", get(handlers::videos::get_video))
        .route("/downloads", get(handlers::download::list_downloads))
        .route("/users/profile", get(handlers::user::get_profile))
        .route_layer(csrf_layer.clone())
        .layer(auth_required_layer.clone());

    // Routes for simple get requires issued by the browser, e.g. through a <source> tag.
    // These should get authentication protection, but not csrf protection.
    let non_csrf_api_routes = Router::new()
        .route("/videos/:id/play", get(handlers::videos::play_video))
        .route_layer(auth_required_layer);

    // Routes we want to access without authentication. They still need csrf protection
    let unauthenticated_routes = Router::new()
        .route(
            "/downloads/add",
            post(handlers::download::add_video_to_queue),
        )
        .route("/auth/logout", post(handlers::auth::auth_logout))
        .route("/auth/auth-url", post(handlers::auth::auth_url))
        .route("/auth/auth-verify", post(handlers::auth::auth_verify))
        .route(
            "/auth/test-unauthorized",
            get(|| async { ListenError::from(ApiError::NotAuthorized) }),
        )
        .route(
            "/auth/test-authorization-pending",
            get(|| async { ListenError::from(ApiError::AuthorizationPending) }),
        )
        .layer(csrf_layer);

    api_routes
        .merge(non_csrf_api_routes)
        .merge(unauthenticated_routes)
        .layer(user_session_layer)
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
