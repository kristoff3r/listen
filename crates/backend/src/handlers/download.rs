use std::time::Duration;

use anyhow::Context;
use api::VideoId;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use database::{
    models::{
        downloads::{Download, DownloadStatus, NewDownload},
        videos::{NewVideo, Video},
    },
    schema,
};
use diesel::{insert_into, prelude::*, update};
use diesel_async::{scoped_futures::ScopedFutureExt, AsyncConnection, RunQueryDsl};
use serde::Deserialize;
use tokio::process::Command;
use tracing::{info, warn};
use ui::server_state::VideosDir;
use youtube_dl::YoutubeDl;

use crate::{
    error::{ListenError, Result},
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
            let video_id: VideoId = insert_into(schema::videos::table)
                .values(NewVideo {
                    title: metadata.title.as_deref().unwrap(),
                    youtube_id: Some(&metadata.id),
                    url: &req.url,
                    metadata: serde_json::to_value(&metadata).ok(),
                    file_path: None,
                })
                .on_conflict_do_nothing()
                .returning(schema::videos::video_id)
                .get_result(&mut conn)
                .await?;

            insert_into(schema::downloads::table)
                .values(NewDownload {
                    video_id,
                    error: None,
                    retry_count: None,
                    status: DownloadStatus::Pending,
                })
                .execute(&mut conn)
                .await?;

            info!(
                "Added video: {} {:?} as id {video_id} to queue",
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
    Path(video_id): Path<VideoId>,
) -> Result<impl IntoResponse> {
    let mut conn = pool.get().await?;

    insert_into(schema::downloads::table)
        .values(NewDownload {
            video_id,
            error: None,
            retry_count: None,
            status: DownloadStatus::Pending,
        })
        .execute(&mut conn)
        .await?;

    Ok(())
}

pub async fn handle_download_queue(pool: PgPool, videos_dir: VideosDir) -> Result<()> {
    use database::schema::downloads::dsl as d;
    use database::schema::videos::dsl as v;

    info!("Starting download queue handler");

    loop {
        let mut conn = pool.get().await?;

        let rows = d::downloads
            .inner_join(v::videos)
            .filter(d::status.eq(DownloadStatus::Pending))
            .filter(v::url.is_not_null())
            .order_by((d::retry_count.asc(), d::created_at.asc()))
            .limit(1)
            .select((Download::as_select(), Video::as_select()))
            .load::<(Download, Video)>(&mut conn)
            .await?;

        let Some((cur_download, cur_video)) = rows.into_iter().next() else {
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        };

        let videos_dir = videos_dir.clone();
        let out_path = videos_dir.join(format!("{}.mp4", cur_video.video_id));
        if out_path.exists() && !cur_download.force {
            info!(
                "Video {id} {title} already exists, skipping",
                id = cur_video.video_id,
                title = cur_video.title
            );
            update(d::downloads.filter(d::download_id.eq(cur_download.download_id)))
                .set((
                    d::retry_count.eq(cur_download.retry_count + 1),
                    d::error.eq("video already exists"),
                    d::status.eq(DownloadStatus::Finished),
                ))
                .execute(&mut conn)
                .await?;
            continue;
        }

        info!(
            "Downloading video: {video_id} {title}",
            video_id = cur_video.video_id,
            title = cur_video.title
        );
        let res = tokio::task::spawn(async move {
            let tmp_dir = tempfile::tempdir_in(&*videos_dir)?;
            download_file(cur_video.url, tmp_dir.path(), &out_path).await?;

            Ok::<_, ListenError>(())
        })
        .await?;

        if let Err(e) = res {
            warn!(
                "Download {video_id} {title} failed: {e}",
                video_id = cur_video.video_id,
                title = cur_video.title
            );
            update(d::downloads.filter(d::download_id.eq(cur_download.download_id)))
                .set((
                    d::retry_count.eq(cur_download.retry_count + 1),
                    d::error.eq(Some(&format!("download failed: {e}"))),
                    if cur_download.retry_count >= 3 {
                        d::status.eq(DownloadStatus::Failed)
                    } else {
                        d::status.eq(DownloadStatus::Pending)
                    },
                ))
                .execute(&mut conn)
                .await?;
        } else {
            info!(
                "Download {video_id} {title} finished",
                video_id = cur_video.video_id,
                title = cur_video.title
            );
            update(d::downloads.filter(d::download_id.eq(cur_download.download_id)))
                .set((
                    d::retry_count.eq(cur_download.retry_count + 1),
                    d::error.eq(""),
                    d::status.eq(DownloadStatus::Finished),
                ))
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
