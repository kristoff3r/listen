use leptos::prelude::*;

use crate::client_state::use_auth_token;

#[component]
pub fn AuthRequired(children: ChildrenFn) -> impl IntoView {
    let _ = children;
    let token = use_auth_token();

    Effect::new(move |_| {
        log::info!("Got token update: {:?}", token.get());
    });

    view! {
        <Show
            when=move || {
                let token = token.get();
                let res = token.is_some();
                log::info!("Inside when, token={token:?}, returning {res}");
                res
            }

            fallback=|| view! { "Not logged in" }
        >
            "A"
        </Show>
    }
}
