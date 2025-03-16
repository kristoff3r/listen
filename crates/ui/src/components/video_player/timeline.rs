use leptos::prelude::*;

use crate::contexts::video_player::use_video_player;

#[component]
pub fn Timeline() -> impl IntoView {
    let video_player = use_video_player();

    let x = move || 100.0 * video_player.current_time.get() / video_player.duration.get();

    view! {
        <div class="w-full bg-green-400">
            <svg class="w-full" height="2rem">
                <line
                    x1="0%"
                    x2=move || format!("{x}%", x = x())
                    y1="50%"
                    y2="50%"
                    stroke="red"
                    stroke-width=5
                    stroke-linecap="round"
                />
                <circle cx=move || format!("{x}%", x = x()) cy="50%" r=7 />
            </svg>
        </div>
    }
}
