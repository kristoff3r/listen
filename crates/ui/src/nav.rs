use api::ApiError;
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::{
    components::{Outlet, A},
    hooks::use_navigate,
};

use crate::backend::use_backend;

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
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            <div id="root" class="grid grid-cols-main grid-rows-1">
                <nav class="flex flex-1 flex-col items-center bg-slate-500 h-screen gap-2 py-2">
                    <A href="/videos">
                        <Icon
                            icon=i::OcVideoLg
                            attr:title="Videos"
                            attr:width="32"
                            attr:height="32"
                        />
                    </A>
                    <A href="/videos">
                        <Icon
                            icon=i::TbListSearch
                            attr:title="Search"
                            attr:width="32"
                            attr:height="32"
                        />
                    </A>
                    <A href="/videos">
                        <Icon
                            icon=i::LuListVideo
                            attr:title="Playlist"
                            attr:width="32"
                            attr:height="32"
                        />
                    </A>
                    <A href="/downloads">
                        <Icon
                            icon=i::BsCloudArrowDown
                            attr:title="Downloads"
                            attr:width="32"
                            attr:height="32"
                        />
                    </A>
                    <div class="mt-auto mb-4">
                        {move || match profile.get().map(|p| p.take()) {
                            Some(Ok(profile)) => {
                                view! {
                                    <>
                                        <A href="/settings">
                                            <img
                                                class="rounded-full w-12 h-12"
                                                src=profile.profile_picture_url
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
                </nav>
                <main class="flex flex-1 my-0 w-full h-screen text-center justif">
                    <Outlet />
                </main>
            </div>
        </Transition>
    }
}
