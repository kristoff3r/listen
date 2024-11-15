use leptos::*;

use crate::loading::Loading;

#[component]
pub fn DownloadsPage() -> impl IntoView {
    let action = create_server_action::<GetDownloads>();
    let downloads = create_resource(move || action.version().get(), |_| get_downloads());

    view! {
        <Transition fallback=move || view! { <Loading/> }>
            <div class="flex w-full min-h-screen">
                <div class="w-[20%] bg-blue-400">
                    {move || match downloads.get() {
                        Some(Ok(downloads)) => view! { <DownloadList downloads/> }.into_view(),
                        Some(Err(e)) => {
                            view! { {format!("error loading: {e}").into_view()} }
                        }
                        _ => view! { <Loading/> }.into_view(),
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
    use database::models::downloads::Download;
    use database::models::videos::Video;
    use database::schema::videos::table as videos_table;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let pool = expect_context::<crate::server_state::ServerState>().pool;
    let mut conn = pool.get().await?;

    let videos = videos_table
        .select(Video::as_select())
        .load(&mut conn)
        .await?;

    let downloads = Download::belonging_to(&videos)
        .select(Download::as_select())
        .load(&mut conn)
        .await?;

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
