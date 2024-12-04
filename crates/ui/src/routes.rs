use leptos::prelude::*;
use leptos_router::components::A;
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
                <RedirectHandler />
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
                        <Route path=path!("/pending") view=PendingPage />
                    </ParentRoute>
                </Routes>
            </ErrorBoundary>
        </Router>
    }
}

#[component]
pub fn RedirectHandler() -> impl IntoView {
    let backend = use_backend();

    move || {
        let path = backend.redirect_signal().get()?;
        Some(view! {
            <Redirect
                path
                options=NavigateOptions {
                    resolve: false,
                    ..Default::default()
                }
            />
        })
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
        <button
            class="gsi-material-button"
            on:click=move |_| {
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
            }
        >
            <div class="gsi-material-button-state"></div>
            <div class="gsi-material-button-content-wrapper">
                <div class="gsi-material-button-icon">
                    <svg
                        version="1.1"
                        xmlns="http://www.w3.org/2000/svg"
                        viewBox="0 0 48 48"
                        xmlns:xlink="http://www.w3.org/1999/xlink"
                        style="display: block;"
                    >
                        <path
                            fill="#EA4335"
                            d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z"
                        ></path>
                        <path
                            fill="#4285F4"
                            d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z"
                        ></path>
                        <path
                            fill="#FBBC05"
                            d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z"
                        ></path>
                        <path
                            fill="#34A853"
                            d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z"
                        ></path>
                        <path fill="none" d="M0 0h48v48H0z"></path>
                    </svg>
                </div>
                <span class="gsi-material-button-contents">Sign in with Google</span>
                <span style="display: none;">Sign in with Google</span>
            </div>
        </button>

        {move || error.get()}
    }
}

#[component]
pub fn LogoutPage() -> impl IntoView {
    let (msg, set_msg) = signal::<Result<bool, AppError>>(Ok(false));

    if !cfg!(feature = "ssr") {
        leptos::reactive::spawn_local_scoped(async move {
            let backend = use_backend();
            match map_error(backend.logout().await) {
                Err(e) => set_msg(Err(e)),
                Ok(()) => set_msg(Ok(true)),
            }
        });
    }

    view! {
        <Show when=move || { msg.get().unwrap() } fallback=|| view! { "Logging out..." }>
            <div>
                <div>"Logged out"</div>
                <A href="/auth/login">
                    <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold rounded py-2 px-4 mt-4">
                        "Login again"
                    </button>
                </A>
            </div>
        </Show>
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

#[component]
pub fn PendingPage() -> impl IntoView {
    view! {
        <div>
            "Unfortunately your account is still pending review. You have to wait until it is approved."
        </div>
        <A href="/auth/logout">
            <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold rounded py-2 px-4 mt-4">
                Click here to try logging into a different account instead.
            </button>
        </A>
    }
}
