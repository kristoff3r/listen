use leptos::prelude::*;

use crate::contexts::video_player::use_video_player;

#[component]
pub fn Clock() -> impl IntoView {
    let video_player = use_video_player();

    let time = move || {
        let t = video_player.current_time.get() as i64;
        let max_t = video_player.duration.get() as i64;
        format!(
            "{:02}:{:02}/{:02}:{:02}",
            t / 60,
            t % 60,
            max_t / 60,
            max_t % 60
        )
    };

    view! { <span>{time}</span> }
}
