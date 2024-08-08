use axum::extract::{Path, State};
use axum::Json;
use database::Video;
use diesel::prelude::*;

use diesel_async::RunQueryDsl;

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
