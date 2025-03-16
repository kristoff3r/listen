use leptos::prelude::*;

use crate::contexts::video_player::use_video_player;

#[component]
pub fn Timeline() -> impl IntoView {
    let video_player = use_video_player();

    let x = move || 100.0 * video_player.current_time.get() / video_player.duration.get();
    let fillColor = "#fff";

    view! {
        <div class="w-full bg-green-400">
            <svg class="w-full" height="2rem" viewBox="0 0 100 100" preserveAspectRatio="none">
                <circle cx=x cy=50 r=10 />
            </svg>
        </div>
    }
}
