use leptos::prelude::*;
use leptos_router::{components::Redirect, NavigateOptions};

#[derive(Copy, Clone)]
pub struct GlobalRedirect {
    redirect_signal: RwSignal<Option<String>>,
}

pub fn provide_global_redirect() {
    provide_context(GlobalRedirect {
        redirect_signal: RwSignal::new(None),
    });
}

pub fn use_global_redirect() -> GlobalRedirect {
    expect_context()
}

#[component]
pub fn GlobalRedirectHandler() -> impl IntoView {
    let redirect_signal = expect_context::<GlobalRedirect>().redirect_signal;

    move || {
        Some(view! {
            <Redirect
                path=redirect_signal.get()?
                options=NavigateOptions {
                    resolve: false,
                    ..Default::default()
                }
            />
        })
    }
}

impl GlobalRedirect {
    pub fn navigate(&self, path: String) {
        self.redirect_signal.set(Some(path))
    }
}
