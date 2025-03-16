use leptos::prelude::*;

#[component]
#[allow(dead_code)]
pub fn EmbedYoutube(youtube_id: String) -> impl IntoView {
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
