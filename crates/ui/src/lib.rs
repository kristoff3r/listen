use contexts::{
    backend::provide_backend, global_redirect::provide_global_redirect,
    video_store::VideoStoreProvider,
};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link, MetaTags, Stylesheet, Title};
use routes::ListenRoutes;

mod components;
mod contexts;
mod errors;
mod layouts;
mod pages;
mod routes;
mod util;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    provide_global_redirect();
    provide_backend();

    view! {
        <Stylesheet id="leptos" href="/pkg/listen.css" />
        <Title text="Listen" />
        <Link rel="icon" href="/favicon.png" sizes="32x32" />
        <ListenRoutes />
        <VideoStoreProvider />
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}
