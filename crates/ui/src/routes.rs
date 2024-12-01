use leptos::prelude::*;
use leptos_router::hooks::use_query;
use leptos_router::params::Params;
use leptos_router::{
    components::{Outlet, ParentRoute, Redirect, Route, Router, Routes},
    hooks::use_navigate,
    path, NavigateOptions,
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
                        <Route path=path!("") view=move || redirect_replace("/videos") />
                        <Route path=path!("/videos") view=VideosPage />
                        <Route path=path!("/downloads") view=DownloadsPage />
                        <Route path=path!("/settings") view=SettingsPage />
                    </ParentRoute>
                    <ParentRoute path=path!("/auth") view=Auth>
                        <Route path=path!("") view=move || redirect_replace("/auth/login") />
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

fn map_error<T>(v: Result<Result<T, api::ApiError>, gloo_net::Error>) -> Result<T, AppError> {
    match v {
        Err(e) => Err(AppError::Crashed(format!(
            "Crashed while doing a backend request: {e:?}"
        ))),
        Ok(Err(e)) => Err(AppError::ApiError(e)),
        Ok(Ok(v)) => Ok(v),
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let (error, set_error) = signal::<Result<(), AppError>>(Ok(()));
    view! {
        <button on:click=move |_| {
            let backend = use_backend();
            let navigate = use_navigate();
            leptos::task::spawn_local(async move {
                let auth_url = backend.auth_url().await;
                match map_error(auth_url) {
                    Err(e) => {
                        set_error(Err(e));
                    }
                    Ok(auth_url) => {
                        navigate(
                            &auth_url.url,
                            NavigateOptions {
                                resolve: false,
                                ..Default::default()
                            },
                        );
                    }
                }
            });
        }>
            <span>
                "Login with google"
                <Icon icon=i::MdiCookiePlusOutline attr:width="32" attr:height="32" />
            </span>
        </button>

        {move || error.get()}
    }
}

#[component]
pub fn LogoutPage() -> impl IntoView {
    let (msg, set_msg) = signal::<Result<Option<String>, AppError>>(Ok(None));

    if !cfg!(feature = "ssr") {
        leptos::reactive::spawn_local_scoped(async move {
            let backend = use_backend();
            match map_error(backend.logout().await) {
                Err(e) => set_msg(Err(e)),
                Ok(()) => set_msg(Ok(Some(format!("You have been logged out")))),
            }
        });
    }

    {
        move || msg.get()
    }
}

#[derive(PartialEq, Eq, Clone, Debug, leptos::Params)]
struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
}

#[component]
pub fn LoginCallback() -> impl IntoView {
    let (error, set_error) = signal::<Result<(), AppError>>(Ok(()));

    if !cfg!(feature = "ssr") {
        leptos::reactive::spawn_local_scoped(async move {
            let backend = use_backend();
            let navigate = use_navigate();
            let params = use_query::<CallbackParams>();

            let params = match params.get_untracked() {
                Ok(params) => params,
                Err(e) => {
                    set_error(Err(AppError::Crashed(format!(
                        "Crashed while getting callback params: {e:?}"
                    ))));
                    return;
                }
            };
            let Some(code) = params.code else {
                set_error(Err(AppError::Crashed(
                    "OAuth callback did not contain a code".to_string(),
                )));
                return;
            };

            let Some(state) = params.state else {
                set_error(Err(AppError::Crashed(
                    "OAuth callback did not contain a state".to_string(),
                )));
                return;
            };

            match map_error(
                backend
                    .auth_verify(&api::AuthVerificationRequest { state, code })
                    .await,
            ) {
                Err(e) => {
                    set_error(Err(e));
                }
                Ok(true) => {
                    navigate(
                        "/",
                        NavigateOptions {
                            resolve: false,
                            replace: true,
                            ..Default::default()
                        },
                    );
                }
                Ok(false) => {
                    set_error(Err(AppError::Crashed("unable to log in :(".to_string())));
                }
            }
        });
    }

    move || error.get()
}

pub fn redirect_replace(path: &'static str) -> impl IntoView {
    view! {
        <Redirect
            path=path
            options=NavigateOptions {
                replace: true,
                scroll: true,
                resolve: false,
                ..Default::default()
            }
        />
    }
}
