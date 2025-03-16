use leptos::{component, view, IntoView};
use leptos_router::hooks::use_navigate;

use crate::contexts::backend::use_backend;

#[component]
pub fn CrowdListPage() -> impl IntoView {
    let backend = use_backend();
    let navigate = use_navigate();

    view! { "list" }
}
