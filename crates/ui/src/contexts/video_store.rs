use api::{Video, VideoId};
use leptos::prelude::*;
use log::error;
use reactive_stores::Store;

use crate::contexts::backend::use_backend;

#[derive(Store, Debug, Clone, Default)]
pub struct VideoStore {
    #[store(key: VideoId = |v| v.video_id)]
    pub videos: Vec<Video>,
}

#[component]
pub fn VideoStoreProvider() -> impl IntoView {
    let backend = use_backend();

    LocalResource::new(move || {
        let backend = backend.clone();
        async move {
            match backend.list_videos().await.unwrap() {
                Ok(videos) => {
                    update_context::<Store<VideoStore>, ()>(|store| store.write().videos = videos);
                }
                Err(e) => error!("{e:?}"),
            }
        }
    });

    provide_context(Store::new(VideoStore::default()));

    view! {}
}

pub fn use_video_store() -> Store<VideoStore> {
    expect_context()
}
