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
        self.duration.set(state.duration);
    }

    pub fn save(&self) -> VideoStorage {
        VideoStorage {
            selected: self.selected.get(),
            current_time: self.current_time.get(),
            duration: self.duration.get(),
        }
    }

    pub fn pause(&self) {
        self.playing.set(false);
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

    pub fn seek(&self, time: f64) {
        let Some(video) = get_element_by_id::<HtmlVideoElement>(VIDEO_PLAYER_ID) else {
            return;
        };

        let time = time.clamp(0.0, self.duration.get_untracked());

        video.set_current_time(time);
    }

    pub fn seek_relative(&self, time: f64) {
        self.current_time.update(|t| *t += time);
        self.seek(self.current_time.get_untracked());
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

    pub fn select(&self, id: VideoId) {
        self.playing.set(false);
        self.current_time.set(0.0);
        self.selected.set(Some(id));
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
