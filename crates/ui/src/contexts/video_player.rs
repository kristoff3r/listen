use api::VideoId;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::HtmlVideoElement;

use crate::util::get_element_by_id;

pub const VIDEO_STATE_KEY: &str = "video_state";
pub const VIDEO_PLAYER_ID: &str = "video_player";

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct VideoStorage {
    selected: Option<VideoId>,
    current_time: f64,
    duration: f64,
    playing: bool,
}

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
}

impl VideoPlayer {
    pub fn load(&self, state: &VideoStorage) {
        self.selected.set(state.selected);
        self.current_time.set(state.current_time);
        self.playing.set(state.playing);
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
}
