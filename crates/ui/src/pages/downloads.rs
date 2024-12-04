use leptos::{either::EitherOf3, prelude::*};

use crate::contexts::backend::use_backend;

#[component]
pub fn DownloadsPage() -> impl IntoView {
    let backend = use_backend();
    let downloads = LocalResource::new(move || {
        let backend = backend.clone();
        async move { backend.list_downloads().await.unwrap() }
    });

    view! {
        <Transition fallback=move || view! { <Loading /> }>
            <div class="flex w-full min-h-screen">
                <div class="w-[20%] bg-blue-400">
                    {move || {
                        match downloads.get().map(|v| v.take()) {
                            Some(Ok(downloads)) => {
                                EitherOf3::A(view! { <DownloadList downloads /> })
                            }
                            Some(Err(e)) => {
                                EitherOf3::B(view! { {format!("error loading: {e}").into_view()} })
                            }
                            None => EitherOf3::C(view! { <Loading /> }),
                        }
                    }}

                </div>
            </div>
        </Transition>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! { <p>"Loading..."</p> }
}

#[component]
fn DownloadList(downloads: Vec<(api::Video, Vec<api::Download>)>) -> impl IntoView {
    let entries = downloads
        .into_iter()
        .map(|(video, downloads)| view! { <DownloadListEntry video downloads /> })
        .collect_view();
    view! { <div class="flex flex-col gap-2">{entries}</div> }
}

#[component]
fn DownloadListEntry(video: api::Video, downloads: Vec<api::Download>) -> impl IntoView {
    view! {
        <div class="hover:bg-green-500">
            {video.title}
            <div>
                {downloads
                    .iter()
                    .map(|d| view! { <p>{format!("{}", d.created_at)}</p> })
                    .collect_view()}
            </div>
        </div>
    }
}
