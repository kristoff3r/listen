use leptos::prelude::*;
use leptos_router::{
    hooks::{use_navigate, use_query},
    params::Params,
    NavigateOptions,
};

use crate::{
    contexts::backend::use_backend,
    errors::{map_gloo_net_error, AppError},
};

#[derive(PartialEq, Eq, Clone, Debug, leptos::Params)]
struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
}

#[component]
pub fn LoginCallbackPage() -> impl IntoView {
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

            match map_gloo_net_error(
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
