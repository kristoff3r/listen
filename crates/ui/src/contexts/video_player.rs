use api::VideoId;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::HtmlVideoElement;

use crate::util::get_element_by_id;

pub const VIDEO_STATE_KEY: &str = "video_state";
pub const VIDEO_PLAYER_ID: &str = "video_player";
pub const VIDEO_SOURCE_ID: &str = "video_source";

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct VideoStorage {
    selected: Option<VideoId>,
    current_time: f64,
    duration: f64,
    playing: bool,
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Loading,
    Initial,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct VideoPlayer {
    pub selected: RwSignal<Option<VideoId>>,
    pub current_time: RwSignal<f64>,
    pub duration: RwSignal<f64>,
    pub playing: RwSignal<bool>,
    is_ready: RwSignal<bool>,
}

impl VideoPlayer {
    pub fn load(&self, state: &VideoStorage) {
        self.selected.set(state.selected);
        self.current_time.set(state.current_time);
        // self.playing.set(state.playing);
        self.duration.set(state.duration);
    }

    pub fn save(&self) -> VideoStorage {
        VideoStorage {
            selected: self.selected.get(),
            current_time: self.current_time.get(),
            playing: self.playing.get(),
            duration: self.duration.get(),
        }
    }

    pub fn update_time(&self) {
        let Some(video) = get_element_by_id::<HtmlVideoElement>(VIDEO_PLAYER_ID) else {
            return;
        };

        let current_time = video.current_time();
        let total_time = video.duration();
        self.current_time.set(current_time);
        self.duration.set(total_time);
    }

    pub fn update_source(&self) {
        let Some(video) = get_element_by_id::<HtmlVideoElement>(VIDEO_PLAYER_ID) else {
            return;
        };

        video.load();
    }

    #[expect(dead_code)]
    pub fn seek(&self, time: f64) {
        let Some(video) = get_element_by_id::<HtmlVideoElement>(VIDEO_PLAYER_ID) else {
            return;
        };

        video.fast_seek(time).unwrap();
    }

    pub fn set_ready(&self, value: bool) {
        self.is_ready.set(value);
    }

    pub fn toggle_playback(&self) {
        let Some(video) = get_element_by_id::<HtmlVideoElement>(VIDEO_PLAYER_ID) else {
            return;
        };

        let playing = self.playing.get();

        if playing {
            video.pause().unwrap();
        } else {
            let _ = video.play().unwrap();
        }

        self.playing.set(!playing);
    }
}

pub fn provide_video_player(video_player: VideoPlayer) {
    provide_context(video_player);
}

pub fn use_video_player() -> VideoPlayer {
    expect_context()
}

pub fn video_src_url(id: VideoId) -> String {
    format!("/api/videos/{id}/play")
}
