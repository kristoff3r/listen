use anyhow::Ok;
use axum::{extract::State, Json};
use database::schema;
use diesel::insert_into;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use tracing::info;
use ui::state::AppState;
use youtube_dl::YoutubeDl;

use crate::types::NewVideo;

#[derive(Deserialize, Debug)]
pub struct DownloadRequest {
    url: String,
}

pub async fn download_url(State(state): State<AppState>, Json(req): Json<DownloadRequest>) {
    let pool = state.pool;
    let videos_dir = state.videos_dir.clone();
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

        // ffmpeg -i input.fil -c:v libx264 -crf 23 -profile:v main -pix_fmt yuv420p -c:a aac -ac 2 -b:a 128k -movflags faststart output.mp4
        YoutubeDl::new(&req.url)
            .format("mp4")
            .output_template(format!("{id}.mp4"))
            .download_to_async(videos_dir)
            .await
            .expect("download failed");

        info!("Downloaded video: {} {:?}", res.id, res.title);

        Ok(())
    });
}
