use leptos::prelude::*;
use leptos_router::components::Outlet;

#[component]
pub fn AuthLayout() -> impl IntoView {
    view! {
        <div>
            <div class="flex flex-col flex-1 justify-center items-center h-svh">
                <Outlet />
            </div>
        </div>
    }
}
