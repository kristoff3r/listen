use leptos::{component, prelude::*, view, CollectView, ErrorBoundary, Errors, IntoView};
use leptos_router::{Outlet, Route, Router, Routes};

use crate::{
    downloads::DownloadsPage,
    error_template::{AppError, ErrorTemplate},
    hooks::auth::AuthRequired,
    videos::VideosPage,
};

#[component]
pub fn ListenRoutes() -> impl IntoView {
    view! {
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
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
                    <Route
                        path="/authed"
                        view=|| {
                            view! {
                                <AuthRequired>
                                    <Outlet/>

                                </AuthRequired>
                            }
                        }
                    >

                        <Route path="/videos" view=VideosPage/>
                        <Route path="/downloads" view=DownloadsPage/>
                    </Route>

                </Routes>
            </ErrorBoundary>
        </Router>
    }
}
