use leptos::prelude::*;
use leptos_router::{hooks::use_navigate, NavigateOptions};

use crate::contexts::backend::use_backend;

#[component]
pub fn settings_page() -> impl IntoView {
    let backend = use_backend();
    let navigate = use_navigate();

    let profile = LocalResource::new(move || {
        let backend = backend.clone();
        async move { backend.get_profile().await.unwrap() }
    });

    let profile_view = move || match profile.get().map(|p| p.take()) {
        Some(Ok(profile)) => view! {
            "You are logged in as"
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
                <div>"Username: " {profile.handle}</div>
                <div>"Email: " {profile.email}</div>
            </div>
        }
        .into_any(),
        Some(Err(_e)) => view! { "Error" }.into_any(),
        _ => view! { <p>"Loading..."</p> }.into_any(),
    };

    let youtube_url = RwSignal::new(String::new());

    view! {
        <div class="flex flex-col flex-1 justify-center items-center">
            <Transition fallback=move || view! {}>{profile_view}</Transition>
            <button
                class="bg-blue-500 hover:bg-blue-700 text-white font-bold rounded py-2 px-4 mt-4"
                on:click=move |_| {
                    let backend = use_backend();
                    leptos::task::spawn_local(async move {
                        log::info!("Got response {:?}", backend.get_unauthorized().await);
                    });
                }
            >
                "Test being unauthorized"
            </button>
            <button
                class="bg-blue-500 hover:bg-blue-700 text-white font-bold rounded py-2 px-4 mt-4"
                on:click=move |_| {
                    let backend = use_backend();
                    leptos::task::spawn_local(async move {
                        log::info!("Got response {:?}", backend.get_authorization_pending().await);
                    });
                }
            >
                "Test being authorization pending"
            </button>
            <div>
                <input
                    class="bg-gray-100 border"
                    placeholder="Youtube URL"
                    type="text"
                    on:input=move |e| { youtube_url.set(event_target_value(&e)) }
                />
                <button
                    class="bg-blue-500 hover:bg-blue-700 text-white font-bold rounded py-1 px-4 mt-4"
                    on:click=move |_| {
                        let backend = use_backend();
                        leptos::task::spawn_local(async move {
                            let response = backend
                                .add_download(
                                    &api::DownloadRequest {
                                        url: youtube_url.get_untracked(),
                                    },
                                )
                                .await;
                            log::info!("Got response {:?}", response);
                        });
                    }
                >
                    "Add to download queue"
                </button>
            </div>
            <button
                class="bg-blue-500 hover:bg-blue-700 text-white font-bold rounded py-2 px-4 mt-4"
                on:click=move |_| {
                    navigate("/auth/logout", NavigateOptions::default());
                }
            >
                "Logout"
            </button>

        </div>
    }
}
