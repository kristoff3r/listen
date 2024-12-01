use leptos::prelude::*;

use crate::backend::use_backend;

#[component]
pub fn settings_page() -> impl IntoView {
    let backend = use_backend();

    let profile = LocalResource::new(move || {
        let backend = backend.clone();
        async move { backend.get_profile().await.unwrap() }
    });

    let profile_view = move || match profile.get().map(|p| p.take()) {
        Some(Ok(profile)) => view! {
            {"You are logged in as"}
            <div class="flex flex-row gap-2">
                <img
                    src=profile.profile_picture_url
                    alt="avatar"
                    class="w-16 h-16 rounded-full"
                    width="16"
                    height="16"
                />
            </div>
            <div class="flex flex-col gap-2">
                <div>{"Username: "} {profile.handle}</div>
                <div>{"Email: "} {profile.email}</div>
            </div>
        }
        .into_any(),
        Some(Err(_e)) => view! { "Error" }.into_any(),
        _ => view! { <p>"Loading..."</p> }.into_any(),
    };

    view! {
        <div class="flex flex-col flex-1 justify-center items-center">
            <Transition fallback=move || view! {}>{profile_view}</Transition>
        </div>
    }
}
