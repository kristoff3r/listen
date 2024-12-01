use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;

use crate::backend::use_backend;

#[component]
pub fn settings_page() -> impl IntoView {
    view! {
        <div class="flex flex-col flex-1 justify-center items-center">

            <button on:click=move |_| {
                let backend = use_backend();
                leptos::task::spawn_local(async move {
                    let cookie_result = backend.set_cookie().await;
                    log::info!("Setting cookie: {:?}", cookie_result);
                });
            }>
                <span>
                    "Set cookie"
                    <Icon icon=i::MdiCookiePlusOutline attr:width="32" attr:height="32" />
                </span>
            </button>

            <button on:click=move |_| {
                let backend = use_backend();
                leptos::task::spawn_local(async move {
                    let cookie_result = backend.get_cookie().await;
                    log::info!("Getting cookie: {:?}", cookie_result);
                });
            }>
                <span>
                    "Get cookie" <Icon icon=i::MdiCookieOutline attr:width="32" attr:height="32" />
                </span>
            </button>

            <button on:click=move |_| {
                let backend = use_backend();
                leptos::task::spawn_local(async move {
                    let cookie_result = backend.clear_cookie().await;
                    log::info!("Clearing cookie: {:?}", cookie_result);
                });
            }>
                <span>
                    "Clear cookie"
                    <Icon icon=i::MdiCookieMinusOutline attr:width="32" attr:height="32" />
                </span>
            </button>
        </div>
    }
}
