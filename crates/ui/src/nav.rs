use icondata as i;
use leptos::*;
use leptos_icons::Icon;
use leptos_router::*;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="flex flex-1 flex-col items-center bg-slate-500 h-screen gap-2 py-2">
            <A href="/videos">
                <Icon icon=i::OcVideoLg width="32" height="32"/>
            </A>
            <A href="/videos">
                <Icon icon=i::TbListSearch width="32" height="32"/>
            </A>
            <A href="/videos">
                <Icon icon=i::LuListVideo width="32" height="32"/>
            </A>
            <A href="/downloads">
                <Icon icon=i::BsCloudArrowDown width="32" height="32"/>
            </A>
            <A href="/settings">
                <Icon icon=i::FiSettings width="32" height="32"/>
            </A>
        </nav>
    }
}
