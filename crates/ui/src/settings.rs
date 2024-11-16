use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;

use crate::errors::AppError;

#[component]
pub fn settings_page() -> impl IntoView {
    let oops: RwSignal<Result<&'static str>> = RwSignal::new(Ok("works"));
    let (read_oops, write_oops) = RwSignal::split(&oops);

    view! {
        <div class="flex flex-col flex-1 justify-center items-center">

            <button on:click=move |_| {
                write_oops.set(Err(AppError::Crashed("TEXT HERE").into()))
            }>
                <span>"Crash" <Icon icon=i::AiBugOutlined attr:width="32" attr:height="32" /></span>
            </button>
            {move || read_oops.get()}
        </div>
    }
}
