use leptos::prelude::*;
use leptos::{component, server::LocalResource, view, IntoView};
use leptos_use::use_interval_fn;

use crate::contexts::backend::use_backend;

#[component]
pub fn CrowdListPage() -> impl IntoView {
    let backend = use_backend();

    let crowd_list = LocalResource::new(move || {
        let backend = backend.clone();
        async move { backend.get_crowd_list().await.unwrap() }
    });

    use_interval_fn(move || crowd_list.refetch(), 2000);

    view! {
        <Transition fallback=move || {
            view! { <p>"Loading..."</p> }
        }>
            <div>
                {move || match crowd_list.get().map(|l| l.take()) {
                    Some(Ok(list)) => view! { {format!("{list:?}")} }.into_any(),
                    Some(Err(e)) => view! { {format!("Error: {e:?}")} }.into_any(),
                    _ => view! { <p>"Loading..."</p> }.into_any(),
                }}
            </div>
        </Transition>
    }
}
