use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;

use crate::contexts::video_player::use_video_player;

const SIZE: &str = "20";

#[component]
pub fn VideoControls() -> impl IntoView {
    let video_player = use_video_player();

    let icon = Memo::new(move |_| {
        if video_player.playing.get() {
            i::TbPlayerPause
        } else {
            i::TbPlayerPlay
        }
    });

    view! {
        <Icon
            icon=icon
            attr:title="Search"
            width=SIZE
            height=SIZE
            on:click=move |_| video_player.toggle_playback()
        />
    }
}
