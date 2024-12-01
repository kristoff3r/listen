use leptos::prelude::*;
use leptos_router::{
    components::{Outlet, ParentRoute, Redirect, Route, Router, Routes},
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
        <Router>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorTemplate errors=errors.into() /> }
            }>
                <Routes fallback>
                    <ParentRoute path=path!("/") view=Nav>
                        <Route path=path!("") view=move || view! { <Redirect path="/videos" /> } />
                        <Route path=path!("/videos") view=VideosPage />
                        <Route path=path!("/downloads") view=DownloadsPage />
                        <Route path=path!("/settings") view=SettingsPage />
                    </ParentRoute>
                    <ParentRoute path=path!("/auth") view=Auth>
                        <Route
                            path=path!("")
                            view=move || view! { <Redirect path="/auth/login" /> }
                        />
                        <Route path=path!("/login") view=LoginPage />
                        <Route path=path!("/callback") view=LoginCallback />
                        <Route path=path!("/logout") view=LogoutPage />
                    </ParentRoute>
                </Routes>
            </ErrorBoundary>
        </Router>
    }
}

#[component]
pub fn Auth() -> impl IntoView {
    view! {
        <div>
            <div class="flex flex-col flex-1 justify-center items-center h-svh">
                <Outlet />
            </div>
        </div>
    }
}

use crate::backend::use_backend;

use icondata as i;
use leptos_icons::Icon;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <button on:click=move |_| {
            let backend = use_backend();
            leptos::task::spawn_local(async move {
                let auth_result = backend.get_auth_url().await;
                log::info!("Getting auth url: {:?}", auth_result);
            });
        }>
            <span>
                "Set auth" <Icon icon=i::MdiCookiePlusOutline attr:width="32" attr:height="32" />
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
                "Clear auth" <Icon icon=i::MdiCookieMinusOutline attr:width="32" attr:height="32" />
            </span>
        </button>
    }
}

#[component]
pub fn LogoutPage() -> impl IntoView {
    "hello2"
}

#[component]
pub fn LoginCallback() -> impl IntoView {
    "hello3"
}
