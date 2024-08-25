use axum::{extract::State, response::IntoResponse, Json};
use database::schema;
use diesel::insert_into;
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use serde::Deserialize;
use tracing::info;
use youtube_dl::YoutubeDl;

use crate::{
    error::Result,
    types::{NewDownload, NewVideo},
    PgPool,
};

#[derive(Deserialize, Debug)]
pub struct DownloadRequest {
    url: String,
}

pub async fn add_to_download_queue(
    State(pool): State<PgPool>,
    Json(req): Json<DownloadRequest>,
) -> Result<impl IntoResponse> {
    let metadata = YoutubeDl::new(&req.url)
        .socket_timeout("15")
        .run_async()
        .await
        .expect("youtube-dl failed")
        .into_single_video()
        .expect("playlist not supported");

    let mut conn = pool.get().await?;

    conn.transaction::<(), anyhow::Error, _>(|mut conn| {
        async move {
            let id: i32 = insert_into(schema::videos::table)
                .values(NewVideo {
                    title: metadata.title.as_deref().unwrap(),
                    youtube_id: Some(&metadata.id),
                    metadata: serde_json::to_value(&metadata).ok(),
                })
                .on_conflict_do_nothing()
                .returning(schema::videos::id)
                .get_result(&mut conn)
                .await?;

            insert_into(schema::downloads::table)
                .values(NewDownload {
                    video_id: id,
                    url: &req.url,
                    error: None,
                })
                .execute(&mut conn)
                .await?;

            info!(
                "Added video: {} {:?} as id {id} to queue",
                metadata.id, metadata.title
            );

            Ok(())
        }
        .scope_boxed()
    })
    .await?;

    Ok(())
}

// pub async fn handle_download_queue(
//     State(pool): State<PgPool>,
//     State(videos_dir): State<VideosDir>,
// ) -> Result<()> {
//     let mut conn = pool.get().await?;
//
//     tokio::task::spawn(async move {
//         let dir = tempfile::tempdir_in(&*videos_dir)?;
//
//         YoutubeDl::new(&req.url)
//             .output_template("%(id)s.%(ext)s")
//             .format("bestvideo*[height<=1080]+bestaudio/best[height<=1080]")
//             .download_to_async(&dir)
//             .await
//             .expect("download failed");
//
//         let foo = dir.as_ref().read_dir()?;
//         for f in foo {
//             let f = f?;
//             info!(
//                 "Running ffmpeg on {}",
//                 f.path().file_name().unwrap().to_string_lossy()
//             );
//             Command::new("ffmpeg")
//                 .arg("-i")
//                 .arg(f.path())
//                 .args(&[
//                     "-c:v",
//                     "libx264",
//                     "-crf",
//                     "23",
//                     "-profile:v",
//                     "main",
//                     "-pix_fmt",
//                     "yuv420p",
//                     "-c:a",
//                     "aac",
//                     "-ac",
//                     "2",
//                     "-b:a",
//                     "128k",
//                     "-movflags",
//                     "faststart",
//                 ])
//                 .arg(videos_dir.join(format!("{id}.mp4")))
//                 .status()
//                 .await?;
//         }
//
//         info!("Downloaded video: {} {:?}", metadata.id, metadata.title);
//
//         Ok(())
//     });
//
//     Ok(())
// }
