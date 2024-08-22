use axum::{
    body::Body,
    extract::{Path, State},
    http::header,
    response::IntoResponse,
    Json,
};
use database::Video;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use tracing::info;
use ui::state::VideosDir;

use crate::{error::Result, PgPool};

pub async fn get_video(State(pool): State<PgPool>, Path(id): Path<i32>) -> Result<Json<Video>> {
    use database::schema::videos::table as video_table;
    let mut conn = pool.get().await?;

    let res = video_table
        .find(id)
        .select(Video::as_select())
        .first(&mut conn)
        .await?;

    Ok(Json(res))
}

pub async fn play_video(
    State(videos_dir): State<VideosDir>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse> {
    let path = videos_dir.join(format!("{id}.mp4"));
    let header = [(header::CONTENT_TYPE, "video/mp4")];
    let file = File::open(path).await?;
    info!("serving file with size {:?}", file.metadata().await?.len());
    let stream = ReaderStream::new(file);
    Ok((header, Body::from_stream(stream)))
}
