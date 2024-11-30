use std::time::Duration;

use anyhow::Context;
use api::VideoId;
use axum::{
    extract::{Path, State},
    Json,
};
use database::models::{Download, DownloadStatus};
use serde::Deserialize;
use tokio::process::Command;
use tracing::{info, warn};
use ui::server_state::VideosDir;
use youtube_dl::YoutubeDl;

use crate::{
    error::{ListenError, ListenErrorExt, Result},
    PgPool,
};

#[derive(Deserialize, Debug)]
pub struct DownloadRequest {
    url: String,
}

pub async fn add_video_to_queue(
    State(pool): State<PgPool>,
    Json(req): Json<DownloadRequest>,
) -> Result<Json<()>> {
    let metadata = YoutubeDl::new(&req.url)
        .socket_timeout("15")
        .run_async()
        .await
        .expect("youtube-dl failed")
        .into_single_video()
        .expect("playlist not supported");

    let mut conn = pool.get().await.with_internal_server_error()?;

    let file_path = format!("{}.mp4", metadata.id);

    let (video, _download) = database::models::Video::create(
        &mut conn,
        metadata.title.as_deref().unwrap(),
        &metadata.id,
        &req.url,
        serde_json::to_value(&metadata).with_internal_server_error()?,
        &file_path,
    )
    .await
    .with_internal_server_error()?;

    info!(
        "Added video: {youtube_id} {title:?} as id {video_id} to queue",
        youtube_id = metadata.id,
        title = metadata.title,
        video_id = video.video_id
    );

    Ok(Json(()))
}

pub async fn redownload_video(
    State(pool): State<PgPool>,
    Path(video_id): Path<VideoId>,
) -> Result<Json<()>> {
    let mut conn = pool.get().await.with_internal_server_error()?;

    database::models::Download::create(&mut conn, video_id)
        .await
        .with_internal_server_error()?;

    Ok(Json(()))
}

pub async fn list_downloads(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<(api::Video, Vec<api::Download>)>>> {
    use diesel::GroupedBy;

    let mut conn = pool.get().await.with_internal_server_error()?;

    let videos = database::models::Video::list(&mut conn).await.unwrap();
    let downloads = database::models::Download::list_for_videos(&mut conn, &videos)
        .await
        .unwrap();

    let res = downloads
        .grouped_by(&videos)
        .into_iter()
        .zip(videos)
        .map(|(downloads, video)| {
            (
                video.into(),
                downloads.into_iter().map(Into::into).collect(),
            )
        })
        .collect::<Vec<(api::Video, Vec<api::Download>)>>();

    Ok(Json(res))
}

pub async fn handle_download_queue(pool: PgPool, videos_dir: VideosDir) -> Result<()> {
    info!("Starting download queue handler");

    loop {
        let mut conn = pool.get().await.with_internal_server_error()?;

        let Some((cur_video, cur_download)) = Download::get_next_download(&mut conn)
            .await
            .with_internal_server_error()?
        else {
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        };

        let videos_dir = videos_dir.clone();
        let out_path = videos_dir.join(cur_video.file_path);
        if out_path.exists() && !cur_download.force {
            info!(
                "Video {video_id} {title} already exists, skipping",
                video_id = cur_video.video_id,
                title = cur_video.title
            );
            database::models::Download::update_set_status(
                &mut conn,
                cur_download.download_id,
                DownloadStatus::Finished,
                "video already exists",
            )
            .await
            .with_internal_server_error()?;
            continue;
        }

        info!(
            "Downloading video: {video_id} {title}",
            video_id = cur_video.video_id,
            title = cur_video.title
        );
        let res = tokio::task::spawn(async move {
            let tmp_dir = tempfile::tempdir_in(&*videos_dir).with_internal_server_error()?;
            download_file(cur_video.url, tmp_dir.path(), &out_path).await?;

            Ok::<_, ListenError>(())
        })
        .await
        .with_internal_server_error()?;

        if let Err(e) = res {
            warn!(
                "Download {video_id} {title} failed: {e}",
                video_id = cur_video.video_id,
                title = cur_video.title
            );

            database::models::Download::update_set_status(
                &mut conn,
                cur_download.download_id,
                if cur_download.retry_count >= 3 {
                    DownloadStatus::Failed
                } else {
                    DownloadStatus::Pending
                },
                &format!("download failed: {e}"),
            )
            .await
            .with_internal_server_error()?;
        } else {
            info!(
                "Download {video_id} {title} finished",
                video_id = cur_video.video_id,
                title = cur_video.title
            );
            database::models::Download::update_set_status(
                &mut conn,
                cur_download.download_id,
                DownloadStatus::Finished,
                "",
            )
            .await
            .with_internal_server_error()?;
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
        .context("download failed")
        .with_internal_server_error()?;

    for f in tmp_dir.read_dir().with_internal_server_error()? {
        let f = f.with_internal_server_error()?;
        info!(
            "Running ffmpeg on {}",
            f.path().file_name().unwrap().to_string_lossy()
        );
        Command::new("ffmpeg")
            .arg("-i")
            .arg(f.path())
            .args([
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
            .context("ffmpeg failed")
            .with_internal_server_error()?;
    }

    Ok(())
}
