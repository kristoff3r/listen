use database::Video;
use leptos::*;

#[component]
pub fn VideosPage() -> impl IntoView {
    let action = create_server_action::<GetVideos>();
    let videos = create_resource(move || action.version().get(), |_| get_videos());

    view! {
        <h2>"video"</h2>
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <ul>
                {move || match videos.get() {
                    Some(Ok(videos)) => {
                        videos.into_iter().map(|video| view! { <Video video/> }).collect_view()
                    }
                    Some(Err(e)) => view! { {format!("error loading video: {e}").into_view()} },
                    _ => view! { <p>"Loading..."</p> }.into_view(),
                }}

            </ul>
        </Transition>
    }
}
#[component]
pub fn video(video: Video) -> impl IntoView {
    view! {
        <div>
            <h3>{video.title} - {video.id}</h3>
            {video
                .youtube_id
                .clone()
                .map(|youtube_id| {
                    view! { <Embed youtube_id/> }
                })}

        </div>
    }
}

#[component]
pub fn embed(youtube_id: String) -> impl IntoView {
    view! {
        <iframe
            width="560"
            height="315"
            src=format!("https://www.youtube.com/embed/{youtube_id}")
            title="YouTube video player"
            frameborder="0"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
            referrerpolicy="strict-origin-when-cross-origin"
            allowfullscreen
        ></iframe>
    }
}

#[server]
pub async fn get_videos() -> Result<Vec<Video>, ServerFnError> {
    use database::schema::videos::table as videos_table;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let pool = expect_context::<crate::state::AppState>().pool;
    let mut conn = pool.get().await?;

    let videos = videos_table
        .select(Video::as_select())
        .get_results(&mut conn)
        .await?;

    Ok(videos)
}
