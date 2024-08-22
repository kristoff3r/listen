use database::Video;
use leptos::*;

#[component]
pub fn VideosPage() -> impl IntoView {
    let action = create_server_action::<GetVideos>();
    let videos = create_resource(move || action.version().get(), |_| get_videos());
    let (active, set_active) = create_signal::<Option<i32>>(None);

    view! {
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <div class="flex w-full min-h-screen">
                <div class="flex items-center justify-center w-[80%] bg-red-400">
                    {move || match active.get() {
                        Some(id) => view! { <EmbedLocal id=id/> }.into_view(),
                        None => view! { <p>"Select a video"</p> }.into_view(),
                    }}

                </div>
                <div class="w-[20%] bg-blue-400">
                    {move || match videos.get() {
                        Some(Ok(videos)) => view! { <VideoList videos set_active/> }.into_view(),
                        Some(Err(e)) => view! { {format!("error loading video: {e}").into_view()} },
                        _ => view! { <p>"Loading..."</p> }.into_view(),
                    }}

                </div>

            </div>
        </Transition>
    }
}

#[component]
pub fn video_list(videos: Vec<Video>, set_active: WriteSignal<Option<i32>>) -> impl IntoView {
    videos
        .into_iter()
        .map(|video| view! { <VideoListEntry video set_active/> })
        .collect_view()
}

#[component]
pub fn video_list_entry(video: Video, set_active: WriteSignal<Option<i32>>) -> impl IntoView {
    view! {
        <div>
            {video.title}
            <button on:click=move |_| set_active.set(Some(video.id))>{"select"}</button>
        </div>
    }
}

#[component]
pub fn embed_local(id: i32) -> impl IntoView {
    view! {
        <video class="min-w-full min-h-full" controls>
            <source src=format!("/api/videos/{id}/play") type="video/mp4"/>
        </video>
    }
}

#[component]
pub fn embed_youtube(youtube_id: String) -> impl IntoView {
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

#[server(GetVideos, "/api/leptos")]
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
