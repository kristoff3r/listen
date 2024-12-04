use leptos::prelude::*;
use leptos_router::components::A;

use crate::{
    contexts::backend::use_backend,
    errors::{map_gloo_net_error, AppError},
};

#[component]
pub fn LogoutPage() -> impl IntoView {
    let (msg, set_msg) = signal::<Result<bool, AppError>>(Ok(false));

    if !cfg!(feature = "ssr") {
        leptos::reactive::spawn_local_scoped(async move {
            let backend = use_backend();
            match map_gloo_net_error(backend.logout().await) {
                Err(e) => set_msg(Err(e)),
                Ok(()) => set_msg(Ok(true)),
            }
        });
    }

    view! {
        <Show when=move || { msg.get().unwrap() } fallback=|| view! { "Logging out..." }>
            <div>
                <div>Logged out</div>
                <A href="/auth/login">
                    <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold rounded py-2 px-4 mt-4">
                        "Login again"
                    </button>
                </A>
            </div>
        </Show>
    }
}
