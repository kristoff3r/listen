use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="flex flex-1 flex-col items-center bg-slate-500 h-screen gap-2 py-2">
            <A href="/videos">
                <Icon icon=i::OcVideoLg attr:title="Videos" attr:width="32" attr:height="32" />
            </A>
            <A href="/videos">
                <Icon icon=i::TbListSearch attr:title="Search" attr:width="32" attr:height="32" />
            </A>
            <A href="/videos">
                <Icon icon=i::LuListVideo attr:title="Playlist" attr:width="32" attr:height="32" />
            </A>
            <A href="/downloads">
                <Icon
                    icon=i::BsCloudArrowDown
                    attr:title="Downloads"
                    attr:width="32"
                    attr:height="32"
                />
            </A>
            <A href="/settings">
                <Icon icon=i::FiSettings attr:title="Settings" attr:width="32" attr:height="32" />
            </A>
        </nav>
    }
}
