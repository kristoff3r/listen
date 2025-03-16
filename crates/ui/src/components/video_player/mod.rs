mod clock;
mod controls;
mod timeline;

use api::VideoId;
use clock::Clock;
use controls::VideoControls;
use leptos::{ev::keydown, prelude::*};
use leptos_use::{use_document, use_event_listener};
use timeline::Timeline;

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
        if &ev.key() == "ArrowLeft" {
            video_player.seek_relative(-3.0);
        }
        if &ev.key() == "ArrowRight" {
            video_player.seek_relative(3.0);
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
                on:ended=move |_| {
                    video_player.pause();
                }
                on:timeupdate=move |_| {
                    video_player.update_time();
                }

                class="w-full max-h-[calc(100vh-2rem)]"
                id=VIDEO_PLAYER_ID

                preload="auto"
                controls=false
                autoplay=false
                playsinline=true
            >
                <source src=src id=VIDEO_SOURCE_ID type="video/mp4" />
            </video>
        </div>
        <div class="flex w-full pl-2 gap-2 items-center bg-pink-400 h-[2rem]">
            <VideoControls />
            <Clock />
            <Timeline />
        </div>
    }
}
