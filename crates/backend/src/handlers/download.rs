use std::time::Duration;

use anyhow::Context;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use database::{schema, Download, DownloadStatus, Video};
use diesel::{insert_into, prelude::*};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use serde::Deserialize;
use tokio::process::Command;
use tracing::{info, warn};
use ui::server_state::VideosDir;
use youtube_dl::YoutubeDl;

use crate::{
    error::{ListenError, Result},
    types::{NewDownload, NewVideo},
    PgPool,
};

#[derive(Deserialize, Debug)]
pub struct DownloadRequest {
    url: String,
}

pub async fn add_video_to_queue(
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
                    url: &req.url,
                    metadata: serde_json::to_value(&metadata).ok(),
                    file_path: None,
                })
                .on_conflict_do_nothing()
                .returning(schema::videos::id)
                .get_result(&mut conn)
                .await?;

            insert_into(schema::downloads::table)
                .values(NewDownload {
                    video_id: id,
                    error: None,
                    retry_count: None,
                    status: DownloadStatus::Pending,
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

pub async fn redownload_video(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse> {
    let mut conn = pool.get().await?;

    insert_into(schema::downloads::table)
        .values(NewDownload {
            video_id: id,
            error: None,
            retry_count: None,
            status: DownloadStatus::Pending,
        })
        .execute(&mut conn)
        .await?;

    Ok(())
}

pub async fn handle_download_queue(pool: PgPool, videos_dir: VideosDir) -> Result<()> {
    use database::schema::downloads::table as downloads_table;
    use database::schema::videos::table as videos_table;

    info!("Starting download queue handler");

    loop {
        let mut conn = pool.get().await?;

        let rows = downloads_table
            .inner_join(videos_table)
            .filter(database::schema::downloads::status.eq(DownloadStatus::Pending))
            .filter(database::schema::videos::url.is_not_null())
            .order_by((
                database::schema::downloads::retry_count.asc(),
                database::schema::downloads::created_at.asc(),
            ))
            .limit(1)
            .select((Download::as_select(), Video::as_select()))
            .load::<(Download, Video)>(&mut conn)
            .await?;

        let Some((download, video)) = rows.into_iter().next() else {
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        };

        info!("Downloading video: {} {}", video.id, video.title);
        let videos_dir = videos_dir.clone();
        let res = tokio::task::spawn(async move {
            let tmp_dir = tempfile::tempdir_in(&*videos_dir)?;
            let out_path = videos_dir.join(format!("{}.mp4", video.id));
            download_file(video.url, tmp_dir.path(), &out_path).await?;

            Ok::<_, ListenError>(())
        })
        .await?;

        if let Err(e) = res {
            warn!(
                "Download {id} {title} failed: {e}",
                id = video.id,
                title = video.title
            );
            insert_into(downloads_table)
                .values(NewDownload {
                    video_id: video.id,
                    error: Some(&format!("download failed: {e}")),
                    retry_count: Some(download.retry_count + 1),
                    status: if download.retry_count >= 3 {
                        DownloadStatus::Failed
                    } else {
                        DownloadStatus::Pending
                    },
                })
                .execute(&mut conn)
                .await?;
        } else {
            info!(
                "Download {id} {title} finished",
                id = video.id,
                title = video.title
            );
            insert_into(downloads_table)
                .values(NewDownload {
                    video_id: video.id,
                    error: None,
                    retry_count: Some(download.retry_count + 1),
                    status: DownloadStatus::Finished,
                })
                .execute(&mut conn)
                .await?;
        }
    }
}

async fn download_file(
    url: String,
    tmp_dir: &std::path::Path,
    out_path: &std::path::Path,
) -> Result<()> {
    YoutubeDl::new(url)
        .output_template("%(id)s.%(ext)s")
        .format("bestvideo*[height<=1080]+bestaudio/best[height<=1080]")
        .download_to_async(&tmp_dir)
        .await
        .context("download failed")?;

    for f in tmp_dir.read_dir()? {
        let f = f?;
        info!(
            "Running ffmpeg on {}",
            f.path().file_name().unwrap().to_string_lossy()
        );
        Command::new("ffmpeg")
            .arg("-i")
            .arg(f.path())
            .args(&[
                "-y",
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
            .arg(out_path)
            .status()
            .await
            .context("ffmpeg failed")?;
    }

    Ok(())
}
