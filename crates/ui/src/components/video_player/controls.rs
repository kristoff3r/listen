use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;
use log::info;
use reactive_stores::Store;

use crate::{
    components::video_player::timeline::Timeline,
    contexts::{
        video_player::use_video_player,
        video_store::{VideoStore, VideoStoreStoreFields},
    },
};

const SIZE: &str = "20";

#[component]
pub fn VideoControls() -> impl IntoView {
    let video_player = use_video_player();

    let videos: Store<VideoStore> = expect_context();

    info!("videos: {:?}", videos.videos());

    let icon = Memo::new(move |_| {
        if video_player.playing.get() {
            i::TbPlayerPause
        } else {
            i::TbPlayerPlay
        }
    });

    view! {
        <div class="flex px-2 gap-2 items-center bg-pink-400 h-[2rem]">
            <Icon
                icon=icon
                attr:title="Search"
                width=SIZE
                height=SIZE
                on:click=move |_| video_player.toggle_playback()
            />
            <Timeline />
        </div>
    }
}
