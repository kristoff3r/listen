use leptos::{either::EitherOf3, prelude::*};

use crate::loading::Loading;

#[component]
pub fn DownloadsPage() -> impl IntoView {
    let action = ServerAction::<GetDownloads>::new();
    let downloads = Resource::new(move || action.version().get(), |_| get_downloads());

    view! {
        <Transition fallback=move || view! { <Loading/> }>
            <div class="flex w-full min-h-screen">
                <div class="w-[20%] bg-blue-400">
                    {move || match downloads.get() {
                        Some(Ok(downloads)) => EitherOf3::A(view! { <DownloadList downloads/> }),
                        Some(Err(e)) => {
                            EitherOf3::B(view! { {format!("error loading: {e}").into_view()} })
                        }
                        _ => EitherOf3::C(view! { <Loading/> }),
                    }}

                </div>
            </div>
        </Transition>
    }
}

#[component]
pub fn download_list(downloads: GetDownloadsResult) -> impl IntoView {
    let entries = downloads
        .into_iter()
        .map(|(video, downloads)| view! { <DownloadListEntry video downloads/> })
        .collect_view();
    view! { <div class="flex flex-col gap-2">{entries}</div> }
}

#[component]
pub fn download_list_entry(video: api::Video, downloads: Vec<api::Download>) -> impl IntoView {
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

type GetDownloadsResult = Vec<(api::Video, Vec<api::Download>)>;

#[server(GetDownloads, "/api/leptos")]
pub async fn get_downloads() -> Result<GetDownloadsResult, ServerFnError> {
    use diesel::GroupedBy;

    let pool = expect_context::<crate::server_state::ServerState>().pool;
    let mut conn = pool.get().await?;

    let videos = database::models::Video::list(&mut conn).await?;
    let downloads = database::models::Download::list_for_videos(&mut conn, &videos).await?;

    let res = downloads
        .grouped_by(&videos)
        .into_iter()
        .zip(videos)
        .map(|(downloads, video)| {
            (
                video.into(),
                downloads.into_iter().map(Into::into).collect(),
            )
        })
        .collect::<Vec<(api::Video, Vec<api::Download>)>>();

    Ok(res)
}
