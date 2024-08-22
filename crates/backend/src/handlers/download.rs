use anyhow::Ok;
use axum::{extract::State, Json};
use database::schema;
use diesel::insert_into;
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use tokio::process::Command;
use tracing::info;
use ui::state::VideosDir;
use youtube_dl::YoutubeDl;

use crate::{types::NewVideo, PgPool};

#[derive(Deserialize, Debug)]
pub struct DownloadRequest {
    url: String,
}

pub async fn download_url(
    State(pool): State<PgPool>,
    State(videos_dir): State<VideosDir>,
    Json(req): Json<DownloadRequest>,
) {
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

        let dir = tempfile::tempdir_in(&*videos_dir)?;

        YoutubeDl::new(&req.url)
            .output_template("%(id)s.%(ext)s")
            .format("bestvideo*[height<=1080]+bestaudio/best[height<=1080]")
            .download_to_async(&dir)
            .await
            .expect("download failed");

        let foo = dir.as_ref().read_dir()?;
        for f in foo {
            let f = f?;
            info!(
                "Running ffmpeg on {}",
                f.path().file_name().unwrap().to_string_lossy()
            );
            Command::new("ffmpeg")
                .arg("-i")
                .arg(f.path())
                .args(&[
                    "-c:v",
                    "libx264",
                    "-crf",
                    "23",
                    "-profile:v",
                    "main",
                    "-pix_fmt",
                    "yuv420p",
                    "-c:a",
                    "aac",
                    "-ac",
                    "2",
                    "-b:a",
                    "128k",
                    "-movflags",
                    "faststart",
                ])
                .arg(videos_dir.join(format!("{id}.mp4")))
                .status()
                .await?;
        }

        info!("Downloaded video: {} {:?}", res.id, res.title);

        Ok(())
    });
}
