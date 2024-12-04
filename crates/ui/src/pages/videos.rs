use api::{Video, VideoId};
use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::storage::use_local_storage;

use crate::contexts::{
    backend::use_backend,
    video_player::{
        provide_video_player, use_video_player, VideoPlayer, VideoStorage, VIDEO_PLAYER_ID,
        VIDEO_STATE_KEY,
    },
};

#[component]
pub fn VideosPage() -> impl IntoView {
    let backend = use_backend();
    let videos = LocalResource::new(move || {
        let backend = backend.clone();
        async move { backend.list_videos().await.unwrap() }
    });

    let (state, set_state, _) = use_local_storage::<VideoStorage, JsonSerdeCodec>(VIDEO_STATE_KEY);

    let video_player = VideoPlayer::default();

    Effect::new(move || {
        let state = state.get_untracked();
        video_player.load(&state);
    });

    Effect::new(move || {
        set_state(video_player.save());
    });

    provide_video_player(video_player);

    view! {
        <div class="flex w-full min-h-screen">
            <div class="w-[200px] bg-blue-400">
                <Transition fallback={move || {
                    view! { <p>"Loading..."</p> }
                }}>
                    {move || match videos.get().map(|v| v.take()) {
                        Some(Ok(videos)) => view! { <VideoList videos /> }.into_any(),
                        Some(Err(e)) => view! { {format!("error loading video: {e}").into_any()} },
                        _ => view! { <p>"Loading..."</p> }.into_any(),
                    }}

                </Transition>
            </div>
            <div class="flex flex-1 items-center justify-center w-fit bg-black text-gray-400">
                {move || match video_player.selected.get() {
                    Some(id) => view! { <EmbedLocal id={id} /> }.into_any(),
                    None => view! { <p>"Select a video"</p> }.into_any(),
                }}

            </div>
        </div>
    }
}

#[component]
pub fn VideoList(videos: Vec<Video>) -> impl IntoView {
    let entries = videos
        .into_iter()
        .map(|video| view! { <VideoListEntry video /> })
        .collect_view();
    view! { <div class="flex flex-col gap-2">{entries}</div> }
}

#[component]
pub fn VideoListEntry(video: Video) -> impl IntoView {
    let video_signals = use_video_player();
    let is_selected = move || video_signals.selected.get() == Some(video.video_id);
    view! {
        <button
            class:bg-green-400={is_selected}
            class="hover:bg-green-500"
            on:click={move |_| video_signals.selected.set(Some(video.video_id))}
        >
            {video.title}
        </button>
    }
}

#[component]
pub fn EmbedLocal(id: VideoId) -> impl IntoView {
    let video_player = use_video_player();

    view! {
        <video
            // data-fullwindow=move || style.full_window.get().to_string()
            // data-fullscreen=move || style.fullscreen.get().to_string()
            // on:waiting=move |_| {
            // let _ = state.set_video_ready(false);
            // }

            // on:loadedmetadata=move |_| {
            // if is_webkit() {
            // let _ = state.set_video_ready(true);
            // }
            // }

            // on:canplay=move |_| {
            // let _ = state.set_video_ready(true);
            // }

            // class="w-full rounded max-h-[calc(100vh-12rem)] data-[fullwindow=true]:max-h-screen data-[fullscreen=true]:max-h-screen"
            id={VIDEO_PLAYER_ID}
            on:timeupdate={move |_| { video_player.update_time() }}

            // poster=video.thumbnails.first().map(|thumb| thumb.url.clone())
            preload="auto"
            controls=true
            autoplay=false
            playsinline=true
        >
            <source src={format!("/api/videos/{id}/play")} type="video/mp4" />
        </video>
    }
}

#[allow(dead_code, unused_variables)]
#[component]
pub fn EmbedYoutube(youtube_id: String) -> impl IntoView {
    view! {
        <iframe
            width="560"
            height="315"
            src={format!("https://www.youtube.com/embed/{youtube_id}")}
            title="YouTube video player"
            // frameborder="0"
            allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
            referrerpolicy="strict-origin-when-cross-origin"
            allowfullscreen
        ></iframe>
    }
}
