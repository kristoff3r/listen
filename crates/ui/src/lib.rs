use client_state::ClientState;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, MetaTags, Stylesheet, Title};
use routes::ListenRoutes;

pub mod client_state;
pub mod downloads;
pub mod errors;
mod hooks;
pub mod loading;
pub mod nav;
mod routes;
#[cfg(feature = "ssr")]
pub mod server_state;
pub mod settings;
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
        <ListenRoutes/>
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}
