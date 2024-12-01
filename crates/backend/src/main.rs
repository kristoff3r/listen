use std::{net::SocketAddr, path::PathBuf, time::Duration};

use anyhow::Context;
use axum::{
    extract::Request,
    middleware::{map_request, map_request_with_state},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum_server::tls_rustls::RustlsConfig;
use database::MIGRATIONS;
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
use ui::{
    server_state::{ServerState, VideosDir},
    App,
};

use crate::db::setup_database_pool;

mod auth;
mod csrf_protection;
pub mod db;
pub mod error;
pub mod handlers;
pub mod ws;

pub use db::PgPool;

const PORT: u16 = 3000;

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

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr: SocketAddr = SocketAddr::new("0.0.0.0".parse().unwrap(), PORT);
    let jwt_secret = b"HEFJKSHFKJSHFKJLASEHFLKWS";
    let jwt_encoding_key = jsonwebtoken::EncodingKey::from_secret(jwt_secret);
    let jwt_decoding_key = jsonwebtoken::DecodingKey::from_secret(jwt_secret);
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
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Could not install default crypto provider for rustls");
    let app = app.into_make_service_with_connect_info::<SocketAddr>();
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
    println!("{routes:?}");
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
    let csrf_layer = map_request(csrf_protection::csrf_protection);
    let auth_layer = map_request_with_state(state, auth_required);

    // Routes will full protection: CSRF + authentication required
    let api_routes = Router::new()
        .route("/videos", get(handlers::videos::list_videos))
        .route("/videos/:id", get(handlers::videos::get_video))
        .route("/downloads", get(handlers::download::list_downloads))
        .route(
            "/downloads/add",
            post(handlers::download::add_video_to_queue),
        )
        .route_layer(csrf_layer.clone())
        .layer(auth_layer.clone());

    // Routes for simple get requires issued by the browser, e.g. through a <source> tag.
    // These should get authentication protection, but not csrf protection.
    let non_csrf_api_routes = Router::new()
        .route("/videos/:id/play", get(handlers::videos::play_video))
        .route_layer(auth_layer);

    // Routes we want to access without authentication. They still need csrf protection
    let unauthenticated_routes = Router::new()
        .route("/get-cookie", get(get_cookie))
        .route("/set-cookie", post(set_cookie))
        .route("/clear-cookie", post(clear_cookie))
        .layer(csrf_layer);

    api_routes
        .merge(non_csrf_api_routes)
        .merge(unauthenticated_routes)
}

const COOKIE_NAME: &str = "__Host-user_token";

async fn set_cookie(cookie_jar: CookieJar) -> error::Result<impl IntoResponse> {
    let cookie_jar = cookie_jar.add(
        Cookie::build((COOKIE_NAME, "bar"))
            .http_only(true)
            .secure(true)
            .path("/")
            .same_site(axum_extra::extract::cookie::SameSite::Strict)
            .expires(time::OffsetDateTime::now_utc().saturating_add(time::Duration::days(30))),
    );

    Ok((cookie_jar, axum::Json(())))
}

async fn get_cookie(cookie_jar: CookieJar) -> error::Result<axum::Json<Option<String>>> {
    Ok(axum::Json(
        cookie_jar.get(COOKIE_NAME).map(|c| c.value().to_string()),
    ))
}

async fn clear_cookie(cookie_jar: CookieJar) -> error::Result<impl IntoResponse> {
    let cookie_jar = cookie_jar.remove(
        Cookie::build(COOKIE_NAME)
            .removal()
            .http_only(true)
            .secure(true)
            .path("/")
            .same_site(axum_extra::extract::cookie::SameSite::Strict),
    );

    Ok((cookie_jar, axum::Json(())))
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

async fn auth_required(cookie_jar: CookieJar, request: Request) -> error::Result<Request> {
    if cookie_jar.get(COOKIE_NAME).is_some() {
        Ok(request)
    } else {
        Err(api::ApiError::NotAuthorized.into())
    }
}
