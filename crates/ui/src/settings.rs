use api::{ApiError, AuthContext};
use icondata as i;
use leptos::{prelude::*, task::spawn_local_scoped};
use leptos_icons::Icon;

use crate::{
    client_state::{use_client_state, AuthClient, AuthToken},
    errors::AppError,
};

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
        <div class="flex flex-col flex-1 justify-center items-center">
            <button on:click=move |_| {
                spawn_local_scoped(async {
                    log::info!("Result of call: {:?}", i_am_called().await);
                });
            }>
                <span>
                    "Invoke auth function"
                    <Icon icon=i::BiLogInRegular attr:width="32" attr:height="32" />
                </span>
            </button>
        </div>

        <div class="flex flex-col flex-1 justify-center items-center">
            <button on:click=move |_| {
                use_client_state().auth_state.set(AuthToken("WHAT?!?".to_string()));
            }>
                <span>"Set my auth token"</span>
            </button>
        </div>

        <div class="flex flex-col flex-1 justify-center items-center">
            <button on:click=move |_| {
                use_client_state().auth_state.clear();
            }>
                <span>"Clear my auth token"</span>
            </button>
        </div>
    }
}

#[server(client = AuthClient)]
pub async fn i_am_called() -> Result<(), ServerFnError<ApiError>> {
    use leptos_axum::extract;

    let axum::Extension(auth_context) = extract::<axum::Extension<AuthContext>, ApiError>().await?;
    println!("Auth context = {auth_context:?}");
    Ok(())
}
