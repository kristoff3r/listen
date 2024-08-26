use client_state::ClientState;
use leptos::*;
use leptos_meta::*;
use routes::ListenRoutes;

pub mod api;
pub mod error_template;

pub mod client_state;
pub mod downloads;
mod hooks;
pub mod loading;
mod routes;
#[cfg(feature = "ssr")]
pub mod server_state;
pub mod videos;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    provide_context(ClientState::new());

    view! {
        <Stylesheet id="leptos" href="/pkg/listen.css"/>

        <Title text="Listen"/>
        <Link rel="icon" href="favicon.png" sizes="32x32"/>

        <main class="my-0 mx-auto text-center justif">
            <ListenRoutes/>
        </main>
    }
}
