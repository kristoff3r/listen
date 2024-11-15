use api::VideoId;
use axum::{
    body::Body,
    extract::{Path, State},
    http::header,
    response::IntoResponse,
    Json,
};
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::info;
use ui::server_state::VideosDir;

use crate::{
    error::{ListenErrorType, Result},
    PgPool,
};

pub async fn get_video(
    State(pool): State<PgPool>,
    Path(video_id): Path<VideoId>,
) -> Result<Json<api::Video>> {
    let mut conn = pool.get().await?;
    let res = database::models::Video::get_by_id(&mut conn, video_id).await?;

    if let Some(res) = res {
        Ok(Json(res.into()))
    } else {
        Err(ListenErrorType::NotFound.into())
    }
}

pub async fn play_video(
    State(pool): State<PgPool>,
    State(videos_dir): State<VideosDir>,
    Path(video_id): Path<VideoId>,
) -> Result<impl IntoResponse> {
    let mut conn = pool.get().await?;
    let Some(video) = database::models::Video::get_by_id(&mut conn, video_id).await? else {
        return Err(ListenErrorType::NotFound.into());
    };

    let path = videos_dir.join(video.file_path);
    let header = [(header::CONTENT_TYPE, "video/mp4")];
    let file = File::open(path).await?;
    info!("serving file with size {:?}", file.metadata().await?.len());
    let stream = ReaderStream::new(file);
    Ok((header, Body::from_stream(stream)))
}
