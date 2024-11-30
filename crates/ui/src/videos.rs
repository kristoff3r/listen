use api::{Video, VideoId};
use leptos::prelude::*;

use crate::backend::use_backend;

#[component]
pub fn videos_page() -> impl IntoView {
    let backend = use_backend();
    let videos = LocalResource::new(move || {
        let backend = backend.clone();
        async move { backend.list_videos().await.unwrap() }
    });
    let selected = RwSignal::new(None);

    view! {
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <div class="flex w-full min-h-screen">
                <div class="w-[200px] bg-blue-400">
                    {move || match videos.get().map(|v| v.take()) {
                        Some(Ok(videos)) => view! { <VideoList videos selected /> }.into_any(),
                        Some(Err(e)) => view! { {format!("error loading video: {e}").into_any()} },
                        _ => view! { <p>"Loading..."</p> }.into_any(),
                    }}

                </div>
                <div class="flex flex-1 items-center justify-center w-fit bg-black text-gray-400">
                    {move || match selected.get() {
                        Some(id) => view! { <EmbedLocal id=id /> }.into_any(),
                        None => view! { <p>"Select a video"</p> }.into_any(),
                    }}

                </div>
            </div>
        </Transition>
    }
}

#[component]
pub fn video_list(videos: Vec<Video>, selected: RwSignal<Option<VideoId>>) -> impl IntoView {
    let entries = videos
        .into_iter()
        .map(|video| view! { <VideoListEntry video selected /> })
        .collect_view();
    view! { <div class="flex flex-col gap-2">{entries}</div> }
}

#[component]
pub fn video_list_entry(video: Video, selected: RwSignal<Option<VideoId>>) -> impl IntoView {
    let is_selected = move || selected.get() == Some(video.video_id);
    view! {
        <button
            class=("bg-green-400", is_selected)
            class="hover:bg-green-500"
            on:click=move |_| selected.set(Some(video.video_id))
        >
            {video.title}
        </button>
    }
}

#[component]
pub fn embed_local(id: VideoId) -> impl IntoView {
    view! {
        <video class="h-full" controls>
            <source src=format!("/api/videos/{id}/play") type="video/mp4" />
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
            // frameborder="0"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
            referrerpolicy="strict-origin-when-cross-origin"
            allowfullscreen
        ></iframe>
    }
}
