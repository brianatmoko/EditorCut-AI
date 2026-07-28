use std::fs;
use std::path::Path;

use chrono::Utc;
use project::Project;
use uuid::Uuid;

use super::EditorState;

impl EditorState {
    pub fn create_new_project(name: &str) -> Project {
        let now = Utc::now().to_rfc3339();
        let scene_id = Uuid::new_v4().to_string();
        let track_id = Uuid::new_v4().to_string();

        use timeline::*;

        Project {
            version: 1,
            metadata: project::ProjectMetadata {
                id: Uuid::new_v4().to_string(),
                name: name.to_string(),
                thumbnail: None,
                duration: time::MediaTime::ZERO,
                created_at: now.clone(),
                updated_at: now,
            },
            current_scene_id: scene_id.clone(),
            scenes: vec![TimelineScene {
                id: scene_id,
                name: "Scene 1".to_string(),
                is_main: true,
                tracks: SceneTracks {
                    overlay: Vec::new(),
                    main: TimelineTrack::Video(VideoTrack {
                        id: track_id,
                        name: "V1".to_string(),
                        elements: Vec::new(),
                        muted: false,
                        hidden: false,
                    }),
                    audio: Vec::new(),
                },
                bookmarks: Vec::new(),
                created_at: Utc::now().to_rfc3339(),
                updated_at: Utc::now().to_rfc3339(),
            }],
            settings: project::ProjectSettings {
                fps: time::FrameRate::FPS_30,
                canvas_size: project::CanvasSize {
                    width: 1920,
                    height: 1080,
                },
                canvas_size_mode: Some("preset".to_string()),
                last_custom_canvas_size: None,
                original_canvas_size: None,
                background: project::Background::Color {
                    color: "#000000".to_string(),
                },
            },
            timeline_view_state: Some(project::TimelineViewState {
                zoom_level: 1.0,
                scroll_left: 0.0,
                playhead_time: time::MediaTime::ZERO,
            }),
        }
    }

    pub fn save_project(&self, file_path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.project)
            .map_err(|e| format!("Failed to serialize project: {}", e))?;
        fs::write(file_path, json)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    }

    pub fn load_project(file_path: &Path) -> Result<Project, String> {
        let json = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        let project: Project = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse project: {}", e))?;
        Ok(project)
    }
}
