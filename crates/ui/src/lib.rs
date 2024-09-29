use client_state::ClientState;
use leptos::*;
use leptos_meta::*;
use routes::ListenRoutes;

pub mod api;
pub mod client_state;
pub mod downloads;
pub mod errors;
mod hooks;
pub mod loading;
pub mod nav;
mod routes;
#[cfg(feature = "ssr")]
pub mod server_state;
pub mod videos;
pub mod settings;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    provide_context(ClientState::new());

    view! {
        <Stylesheet id="leptos" href="/pkg/listen.css"/>

        <Title text="Listen"/>
        <Link rel="icon" href="favicon.png" sizes="32x32"/>

        <ListenRoutes/>
    }
}
