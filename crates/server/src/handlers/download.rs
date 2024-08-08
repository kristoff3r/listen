use anyhow::Ok;
use axum::extract::State;
use axum::Json;
use database::schema;
use diesel::insert_into;
use serde::Deserialize;
use tracing::info;
use youtube_dl::YoutubeDl;

use diesel_async::RunQueryDsl;

use crate::types::NewVideo;
use crate::PgPool;

#[derive(Deserialize, Debug)]
pub struct DownloadRequest {
    url: String,
}

pub async fn download_url(State(pool): State<PgPool>, Json(req): Json<DownloadRequest>) {
    tokio::task::spawn(async move {
        let res = YoutubeDl::new(&req.url)
            .socket_timeout("15")
            .run_async()
            .await
            .expect("youtube-dl failed")
            .into_single_video()
            .expect("playlist not supported");

        let mut conn = pool.get().await?;

        let id: i32 = insert_into(schema::videos::table)
            .values(NewVideo {
                title: res.title.as_deref().unwrap(),
                youtube_id: Some(&res.id),
            })
            .returning(schema::videos::id)
            .get_result(&mut conn)
            .await?;

        info!("Inserted video: {} {:?} as id {id}", res.id, res.title);

        YoutubeDl::new(&req.url)
            .output_template(format!("{id}"))
            .download_to_async("videos")
            .await
            .expect("download failed");

        info!("Downloaded video: {} {:?}", res.id, res.title);

        Ok(())
    });
}
