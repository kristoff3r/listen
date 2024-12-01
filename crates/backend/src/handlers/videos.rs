use std::io::ErrorKind;

use api::{ApiError, AuthContext, VideoId};
use axum::{
    body::Body,
    extract::{Path, State},
    http::header,
    response::IntoResponse,
    Extension, Json,
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::info;
use tracing_error::SpanTrace;

use crate::{
    error::{ListenError, ListenErrorExt, Result},
    server_state::VideosDir,
    PgPool,
};

pub async fn get_video(
    State(pool): State<PgPool>,
    Path(video_id): Path<VideoId>,
    Extension(auth_context): Extension<AuthContext>,
) -> Result<Json<api::Video>> {
    println!("{auth_context:?}");
    let mut conn = pool.get().await.with_internal_server_error()?;
    let res = database::models::Video::get_by_id(&mut conn, video_id)
        .await
        .with_internal_server_error()?;

    if let Some(res) = res {
        Ok(Json(res.into()))
    } else {
        Err(ApiError::NotFound.into())
    }
}

pub async fn play_video(
    State(pool): State<PgPool>,
    State(videos_dir): State<VideosDir>,
    Path(video_id): Path<VideoId>,
) -> Result<impl IntoResponse> {
    let mut conn = pool.get().await.with_internal_server_error()?;
    let Some(video) = database::models::Video::get_by_id(&mut conn, video_id)
        .await
        .with_internal_server_error()?
    else {
        return Err(ApiError::NotFound.into());
    };

    info!("videos_dir={videos_dir}", videos_dir = videos_dir.display());
    info!("file_path={file_path}", file_path = video.file_path);
    let path = videos_dir.join(video.file_path);
    let header = [(header::CONTENT_TYPE, "video/mp4")];
    let file = File::open(path).await.map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            ListenError::from(ApiError::NotFound)
        } else {
            ListenError {
                api_error: ApiError::InternalServerError,
                inner: e.into(),
                context: SpanTrace::capture(),
            }
        }
    })?;
    info!(
        "serving file with size {:?}",
        file.metadata().await.with_internal_server_error()?.len()
    );
    let stream = ReaderStream::new(file);
    Ok((header, Body::from_stream(stream)))
}

pub async fn list_videos(State(pool): State<PgPool>) -> Result<Json<Vec<api::Video>>> {
    let mut conn = pool.get().await.with_internal_server_error()?;

    let videos = database::models::Video::list(&mut conn).await.unwrap();
    let videos = videos
        .into_iter()
        .map(From::from)
        .collect::<Vec<api::Video>>();

    Ok(Json(videos))
}
