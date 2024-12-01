use api::ApiError;
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::{
    components::{Outlet, A},
    hooks::use_navigate,
};

use crate::backend::use_backend;

pub const SIZE: &str = "32";

#[component]
pub fn Nav() -> impl IntoView {
    let navigate = use_navigate();
    let backend = use_backend();

    let profile = LocalResource::new(move || {
        let backend = backend.clone();
        let navigate = navigate.clone();
        async move {
            match backend.get_profile().await.unwrap() {
                Err(ApiError::NotAuthorized) => {
                    navigate("/auth/login", Default::default());
                    Err(ApiError::NotAuthorized)
                }
                res => res,
            }
        }
    });

    view! {
        <div id="root" class="grid grid-cols-main grid-rows-1">
            <nav class="flex flex-1 flex-col items-center bg-slate-500 h-screen gap-2 py-2">
                <A href="/videos">
                    <Icon icon=i::OcVideoLg attr:title="Videos" width=SIZE height=SIZE />
                </A>
                <A href="/videos">
                    <Icon icon=i::TbListSearch attr:title="Search" width=SIZE height=SIZE />
                </A>
                <A href="/videos">
                    <Icon icon=i::LuListVideo attr:title="Playlist" width=SIZE height=SIZE />
                </A>
                <A href="/downloads">
                    <Icon icon=i::BsCloudArrowDown attr:title="Downloads" width=SIZE height=SIZE />
                </A>
                <Transition fallback=move || view! {}>
                    <div class="mt-auto mb-4">
                        {move || match profile.get().map(|p| p.take()) {
                            Some(Ok(profile)) => {
                                view! {
                                    <>
                                        <A href="/settings">
                                            <img
                                                class="rounded-full w-12 h-12"
                                                src=profile
                                                    .profile_picture_url
                                                    .unwrap_or_else(|| {
                                                        "https://avatars.githubusercontent.com/u/160317?v=4"
                                                            .to_string()
                                                    })
                                                alt="Profile"
                                                title=profile.handle
                                            />
                                        </A>
                                    </>
                                }
                                    .into_any()
                            }
                            _ => view! {}.into_any(),
                        }}
                    </div>
                </Transition>
            </nav>
            <main class="flex flex-1 my-0 w-full h-screen text-center justif">
                <Outlet />
            </main>
        </div>
    }
}
