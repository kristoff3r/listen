use leptos::prelude::*;
use leptos_router::{
    components::{Outlet, Route, Router, Routes},
    path,
};

use crate::{
    downloads::DownloadsPage,
    errors::{AppError, ErrorTemplate},
    nav::Nav,
    settings::SettingsPage,
    videos::VideosPage,
};

#[component]
pub fn ListenRoutes() -> impl IntoView {
    let fallback = || {
        let mut outside_errors = Errors::default();
        outside_errors.insert_with_default_key(AppError::NotFound);
        let errors = RwSignal::new(outside_errors);
        view! { <ErrorTemplate errors /> }
    };

    view! {
        <div id="root" class="grid grid-cols-main grid-rows-1">
            <Router>
                <Nav />
                <main class="flex flex-1 my-0 w-full h-screen text-center justif">
                    <ErrorBoundary fallback=|errors| {
                        view! { <ErrorTemplate errors=errors.into() /> }
                    }>
                        <Routes fallback>
                            <Route path=path!("/") view=VideosPage />
                            <Route path=path!("/videos") view=VideosPage />
                            <Route path=path!("/downloads") view=DownloadsPage />
                            <Route path=path!("/settings") view=SettingsPage />
                            <Route path=path!("/auth/login") view=LoginPage />
                            <Route path=path!("/auth/callback") view=LoginCallback />
                            <Route path=path!("/auth/logout") view=LogoutPage />
                        </Routes>
                    </ErrorBoundary>
                </main>
            </Router>
        </div>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    "hello1"
}

#[component]
pub fn LogoutPage() -> impl IntoView {
    "hello2"
}

#[component]
pub fn LoginCallback() -> impl IntoView {
    "hello3"
}
