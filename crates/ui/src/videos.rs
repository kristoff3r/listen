use database::Video;
use icondata as i;
use leptos::*;
use leptos_icons::Icon;
use leptos_router::A;

#[component]
pub fn VideosPage() -> impl IntoView {
    let action = create_server_action::<GetVideos>();
    let videos = create_resource(move || action.version().get(), |_| get_videos());
    let selected = create_rw_signal::<Option<i32>>(None);

    view! {
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <div class="flex w-full min-h-screen">
                <div class="flex items-center justify-center w-[80%] bg-black text-gray-400">
                    {move || match selected.get() {
                        Some(id) => view! { <EmbedLocal id=id/> }.into_view(),
                        None => view! { <p>"Select a video"</p> }.into_view(),
                    }}

                </div>
                <div class="w-[20%] bg-blue-400">
                    {move || match videos.get() {
                        Some(Ok(videos)) => view! { <VideoList videos selected/> }.into_view(),
                        Some(Err(e)) => view! { {format!("error loading video: {e}").into_view()} },
                        _ => view! { <p>"Loading..."</p> }.into_view(),
                    }}

                </div>

            </div>
        </Transition>
    }
}

#[component]
pub fn video_list(videos: Vec<Video>, selected: RwSignal<Option<i32>>) -> impl IntoView {
    let entries = videos
        .into_iter()
        .map(|video| view! { <VideoListEntry video selected/> })
        .collect_view();
    view! {
        <div class="flex flex-col gap-2">
            <div>
                <A class="flex flex-row bg-slate-500 py-2 px-2" href="/downloads">
                    <Icon icon=i::BsCloudArrowDown/>
                </A>
            </div>
            {entries}
        </div>
    }
}

#[component]
pub fn video_list_entry(video: Video, selected: RwSignal<Option<i32>>) -> impl IntoView {
    let is_selected = move || selected.get() == Some(video.id);
    view! {
        <button
            class=("bg-green-400", is_selected)
            class="hover:bg-green-500"
            on:click=move |_| selected.set(Some(video.id))
        >
            {video.title}
        </button>
    }
}

#[component]
pub fn embed_local(id: i32) -> impl IntoView {
    view! {
        <video class="h-full" controls>
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

    let pool = expect_context::<crate::server_state::ServerState>().pool;
    let mut conn = pool.get().await?;

    let videos = videos_table
        .select(Video::as_select())
        .get_results(&mut conn)
        .await?;

    Ok(videos)
}
