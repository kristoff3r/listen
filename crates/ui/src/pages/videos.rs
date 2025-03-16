use api::Video;
use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::storage::use_local_storage;
use log::info;

use crate::components::video_player::VideoPlayer;
use crate::contexts::video_player::{
    provide_video_player, use_video_player, VideoPlayer, VideoStorage, VIDEO_STATE_KEY,
};
use crate::contexts::video_store::{use_video_store, VideoStoreStoreFields};

#[component]
pub fn VideosPage() -> impl IntoView {
    let (state, set_state, _) = use_local_storage::<VideoStorage, JsonSerdeCodec>(VIDEO_STATE_KEY);

    let video_player = VideoPlayer::default();

    Effect::new(move || {
        let state = state.get_untracked();
        video_player.load(&state);
        info!("Loaded state {state:?}");
    });

    Effect::new(move || {
        let state = video_player.save();
        info!("Saved state {state:?}");
        set_state(state);
    });

    provide_video_player(video_player);

    let video_store = use_video_store();

    view! {
        <>
            <div class="flex w-full min-h-screen">
                <div class="w-[200px] bg-blue-400">
                    <Transition fallback=move || {
                        view! { <p>"Loading..."</p> }
                    }>
                        {move || view! { <VideoList videos=video_store.videos().get() /> }}
                    </Transition>
                </div>
                <div class="flex flex-col w-full">
                    {move || match video_player.selected.get() {
                        Some(id) => {
                            info!("selected={:?} id={id:?}", video_player.selected.get());

                            view! { <VideoPlayer id=id /> }
                                .into_any()
                        }
                        None => {
                            view! {
                                <div class="flex flex-1 items-center justify-center w-full bg-black text-gray-400">
                                    <p>"Select a video"</p>
                                </div>
                            }
                                .into_any()
                        }
                    }}

                </div>
            </div>
        </>
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
    let video_player = use_video_player();
    let is_selected = move || video_player.selected.get() == Some(video.video_id);
    view! {
        <button
            class=("bg-green-400", is_selected)
            class="hover:bg-green-500"
            on:click=move |_| {
                video_player.pause();
                video_player.selected.set(Some(video.video_id));
            }
        >
            {video.title}
        </button>
    }
}
