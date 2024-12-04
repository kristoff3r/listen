use leptos::prelude::*;
use leptos_router::{
    components::{ParentRoute, Redirect, Route, Router, Routes},
    path, NavigateOptions,
};

use crate::{
    contexts::global_redirect::GlobalRedirectHandler,
    errors::{AppError, ErrorTemplate},
    layouts, pages,
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
            <ErrorBoundary fallback={|errors| {
                view! { <ErrorTemplate errors={errors.into()} /> }
            }}>
                <GlobalRedirectHandler />
                <Routes fallback>
                    <ParentRoute path={path!("/")} view={layouts::PageLayout}>
                        <Route path={path!("")} view={move || redirect_replace("/videos")} />
                        <Route path={path!("/videos")} view={pages::VideosPage} />
                        <Route path={path!("/downloads")} view={pages::DownloadsPage} />
                        <Route path={path!("/settings")} view={pages::SettingsPage} />
                    </ParentRoute>
                    <ParentRoute path={path!("/auth")} view={layouts::AuthLayout}>
                        <Route path={path!("")} view={move || redirect_replace("/auth/login")} />
                        <Route path={path!("/login")} view={pages::auth::LoginPage} />
                        <Route path={path!("/callback")} view={pages::auth::LoginCallbackPage} />
                        <Route path={path!("/logout")} view={pages::auth::LogoutPage} />
                        <Route path={path!("/pending")} view={pages::auth::PendingPage} />
                    </ParentRoute>
                </Routes>
            </ErrorBoundary>
        </Router>
    }
}

pub fn redirect_replace(path: &'static str) -> impl IntoView {
    view! {
        <Redirect
            path={path}
            options={NavigateOptions {
                replace: true,
                scroll: true,
                resolve: false,
                ..Default::default()
            }}
        />
    }
}
