use serde::{Deserialize, Serialize};

use crate::{Bookmark, TimelineElement, TimelineTrack};

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SceneTracks {
    pub overlay: Vec<TimelineTrack>,
    pub main: TimelineTrack,
    pub audio: Vec<TimelineTrack>,
}

impl SceneTracks {
    pub fn all_tracks(&self) -> Vec<&TimelineTrack> {
        let mut tracks: Vec<&TimelineTrack> = self.overlay.iter().collect();
        tracks.push(&self.main);
        tracks.extend(self.audio.iter());
        tracks
    }

    pub fn all_tracks_mut(&mut self) -> Vec<&mut TimelineTrack> {
        let mut tracks: Vec<&mut TimelineTrack> = self.overlay.iter_mut().collect();
        tracks.push(&mut self.main);
        tracks.extend(self.audio.iter_mut());
        tracks
    }

    pub fn find_track(&self, track_id: &str) -> Option<&TimelineTrack> {
        self.all_tracks().into_iter().find(|t| t.id() == track_id)
    }

    pub fn find_track_mut(&mut self, track_id: &str) -> Option<&mut TimelineTrack> {
        self.all_tracks_mut().into_iter().find(|t| t.id() == track_id)
    }

    pub fn add_overlay_track(&mut self, track: TimelineTrack) {
        self.overlay.push(track);
    }

    pub fn add_audio_track(&mut self, track: TimelineTrack) {
        self.audio.push(track);
    }

    pub fn remove_track(&mut self, track_id: &str) -> Option<TimelineTrack> {
        if self.main.id() == track_id {
            return None;
        }
        if let Some(pos) = self.overlay.iter().position(|t| t.id() == track_id) {
            return Some(self.overlay.remove(pos));
        }
        if let Some(pos) = self.audio.iter().position(|t| t.id() == track_id) {
            return Some(self.audio.remove(pos));
        }
        None
    }

    pub fn track_count(&self) -> usize {
        self.overlay.len() + 1 + self.audio.len()
    }
}

#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(from_wasm_abi, into_wasm_abi))]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimelineScene {
    pub id: String,
    pub name: String,
    pub is_main: bool,
    pub tracks: SceneTracks,
    pub bookmarks: Vec<Bookmark>,
    pub created_at: String,
    pub updated_at: String,
}

impl TimelineScene {
    pub fn find_track(&self, track_id: &str) -> Option<&TimelineTrack> {
        self.tracks.find_track(track_id)
    }

    pub fn find_track_mut(&mut self, track_id: &str) -> Option<&mut TimelineTrack> {
        self.tracks.find_track_mut(track_id)
    }

    pub fn insert_element(&mut self, track_id: &str, element: TimelineElement) -> bool {
        if let Some(track) = self.tracks.find_track_mut(track_id) {
            track.elements_mut().push(element);
            true
        } else {
            false
        }
    }

    pub fn find_element_mut(&mut self, element_id: &str) -> Option<&mut TimelineElement> {
        for track in self.tracks.all_tracks_mut() {
            if let Some(elem) = track.elements_mut().iter_mut().find(|e| e.base().id == element_id) {
                return Some(elem);
            }
        }
        None
    }

    pub fn remove_element(&mut self, element_id: &str) -> Option<TimelineElement> {
        for track in self.tracks.all_tracks_mut() {
            let pos = track.elements_mut().iter().position(|e| e.base().id == element_id);
            if let Some(idx) = pos {
                return Some(track.elements_mut().remove(idx));
            }
        }
        None
    }

    pub fn all_elements(&self) -> Vec<&TimelineElement> {
        let mut elements = Vec::new();
        for track in self.tracks.all_tracks() {
            elements.extend(track.elements().iter());
        }
        elements
    }
}
