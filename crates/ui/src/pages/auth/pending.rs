use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn PendingPage() -> impl IntoView {
    view! {
        <div>
            "Unfortunately your account is still pending review. You have to wait until it is approved."
        </div>
        <A href="/auth/logout">
            <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold rounded py-2 px-4 mt-4">
                "Click here to try logging into a different account instead."
            </button>
        </A>
    }
}
