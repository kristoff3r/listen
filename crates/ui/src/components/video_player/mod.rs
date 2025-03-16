mod controls;
mod timeline;

use api::VideoId;
use controls::VideoControls;
use leptos::{ev::keydown, prelude::*};
use leptos_use::{use_document, use_event_listener};

use crate::contexts::video_player::{
    use_video_player, video_src_url, VIDEO_PLAYER_ID, VIDEO_SOURCE_ID,
};

#[component]
pub fn VideoPlayer(id: VideoId) -> impl IntoView {
    let video_player = use_video_player();
    video_player.update_source();

    let src = video_src_url(id);

    let _ = use_event_listener(use_document(), keydown, move |ev| {
        if &ev.key() == " " {
            video_player.toggle_playback();
            ev.prevent_default();
            ev.stop_propagation();
        }
    });

    view! {
        <div class="flex flex-1 items-center justify-center w-full bg-black text-gray-400">
            <video
                on:waiting=move |_| {
                    video_player.set_ready(false);
                }

                on:canplay=move |_| {
                    video_player.set_ready(true);
                }

                class="w-full max-h-[calc(100vh-2rem)]"
                id=VIDEO_PLAYER_ID
                on:timeupdate=move |_| {
                    video_player.update_time();
                }

                preload="auto"
                controls=false
                autoplay=false
                playsinline=true
            >
                <source src=src id=VIDEO_SOURCE_ID type="video/mp4" />
            </video>
        </div>
        <VideoControls />
    }
}
