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
                    let auth_result = backend.set_auth().await;
                    log::info!("Setting auth: {:?}", auth_result);
                });
            }>
                <span>
                    "Set auth"
                    <Icon icon=i::MdiCookiePlusOutline attr:width="32" attr:height="32" />
                </span>
            </button>

            <button on:click=move |_| {
                let backend = use_backend();
                leptos::task::spawn_local(async move {
                    let auth_result = backend.get_auth().await;
                    log::info!("Getting auth: {:?}", auth_result);
                });
            }>
                <span>
                    "Get auth" <Icon icon=i::MdiCookieOutline attr:width="32" attr:height="32" />
                </span>
            </button>

            <button on:click=move |_| {
                let backend = use_backend();
                leptos::task::spawn_local(async move {
                    let auth_result = backend.clear_auth().await;
                    log::info!("Clearing auth: {:?}", auth_result);
                });
            }>
                <span>
                    "Clear auth"
                    <Icon icon=i::MdiCookieMinusOutline attr:width="32" attr:height="32" />
                </span>
            </button>
        </div>
    }
}
