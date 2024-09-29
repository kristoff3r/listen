use icondata as i;
use leptos::*;
use leptos_icons::Icon;

use crate::errors::AppError;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let (read_oops, write_oops) = create_signal(Ok("works"));

    view! {
        <div class="flex flex-col flex-1 justify-center items-center">

            <button on:click=move |_| { write_oops.set(Err(AppError::Crashed("TEXT HERE"))) }>
                <span>"Crash" <Icon icon=i::AiBugOutlined width="32" height="32"/></span>
            </button>
            {move || read_oops.get()}
        </div>
    }
}
