use crate::components::nav_bar::NavBar;
use leptos::prelude::*;
use leptos_router::components::Outlet;

#[component]
pub fn PageLayout() -> impl IntoView {
    view! {
        <div id="root" class="grid grid-cols-main grid-rows-1">
            <NavBar />
            <main class="flex flex-1 my-0 w-full h-screen text-center justif">
                <Outlet />
            </main>
        </div>
    }
}
