use crate::error_template::{AppError, ErrorTemplate};

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use videos::VideosPage;

pub mod api;
pub mod error_template;

#[cfg(feature = "ssr")]
pub mod state;
pub mod videos;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/listen.css"/>

        <Title text="Listen"/>
        <Link rel="icon" href="favicon.png" sizes="32x32"/>

        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <main class="my-0 mx-auto text-center justif">
                <ErrorBoundary fallback=|errors| {
                    view! {
                        <div class="error">
                            <p>LOL FEJL</p>
                            <ul>
                                {move || {
                                    errors
                                        .get()
                                        .into_iter()
                                        .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                                        .collect_view()
                                }}

                            </ul>
                        </div>
                    }
                }>
                    <Routes>
                        <Route path="/" view=VideosPage/>
                    </Routes>
                </ErrorBoundary>
            </main>
        </Router>
    }
}
