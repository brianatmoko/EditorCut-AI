use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    div, prelude::*, px, ClickEvent, Context, IntoElement, ParentElement,
    Render, SharedString, Styled, Window,
};

use crate::state::EditorState;
use crate::ui::theme::Theme;

pub struct Toolbar {
    editor: Rc<RefCell<EditorState>>,
}

impl Toolbar {
    pub fn new(editor: Rc<RefCell<EditorState>>) -> Self {
        Self { editor }
    }

    fn import_media(editor: &Rc<RefCell<EditorState>>) {
        let files = rfd::FileDialog::new()
            .add_filter("Media", &["mp4", "mov", "avi", "mkv", "png", "jpg", "jpeg", "gif", "mp3", "wav", "flac", "ogg"])
            .pick_files();

        if let Some(paths) = files {
            for path in paths {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let ext = path.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    let media_type = match ext.as_str() {
                        "mp4" | "mov" | "avi" | "mkv" => media::MediaType::Video,
                        "png" | "jpg" | "jpeg" | "gif" => media::MediaType::Image,
                        "mp3" | "wav" | "flac" | "ogg" => media::MediaType::Audio,
                        _ => continue,
                    };

                    let metadata = match std::fs::metadata(&path) {
                        Ok(m) => m,
                        _ => continue,
                    };

                    let mut editor = editor.borrow_mut();
                    editor.media_pool.assets.push(media::MediaAsset {
                        id: uuid::Uuid::new_v4().to_string(),
                        name: name.to_string(),
                        media_type,
                        file_path: path.to_string_lossy().to_string(),
                        thumbnail: None,
                        duration: None,
                        width: None,
                        height: None,
                        size: metadata.len(),
                    });
                    editor.mark_modified();
                }
            }
        }
    }

    fn save_project(editor: &Rc<RefCell<EditorState>>) {
        let file = rfd::FileDialog::new()
            .add_filter("OpenCut Project", &["opencut.json"])
            .set_file_name("project.opencut.json")
            .save_file();

        if let Some(path) = file {
            let state = editor.borrow();
            match state.save_project(&path) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Failed to save project: {}", e);
                }
            }
            drop(state);
            let mut state = editor.borrow_mut();
            state.project_file_path = Some(path.to_string_lossy().to_string());
            state.mark_saved();
        }
    }
}

impl Render for Toolbar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.request_animation_frame();

        let editor = self.editor.borrow();
        let project_name = SharedString::from(editor.project.metadata.name.clone());
        let is_modified = editor.is_modified;
        drop(editor);

        let modified_indicator = if is_modified { " •" } else { "" };

        div()
            .w_full()
            .h(px(48.0))
            .bg(Theme::bg_surface())
            .border_b_1()
            .border_color(Theme::border_subtle())
            .flex()
            .items_center()
            .gap(px(4.0))
            .px(px(12.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child(toolbar_btn("tb-import", "📁", "Import")
                        .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                            Self::import_media(&this.editor);
                        }))
                    )
                    .child(toolbar_btn("tb-save", "💾", "Save")
                        .on_click(cx.listener(|this, _: &ClickEvent, _w, _cx| {
                            Self::save_project(&this.editor);
                        }))
                    )
                    .child(divider()),
            )
            .child(
                div()
                    .flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(Theme::text_primary())
                            .child(SharedString::from(format!("{}{}", project_name, modified_indicator))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child(divider())
                    .child(
                        toolbar_btn("tb-export", "⬆", "Export")
                            .on_click(cx.listener(|_this, _: &ClickEvent, _w, _cx| {
                                eprintln!("[Export] Not implemented yet");
                            })),
                    )
            )
    }
}

fn toolbar_btn(id: &'static str, icon: &str, label: &str) -> gpui::Stateful<gpui::Div> {
    let icon_str = SharedString::from(icon.to_string());
    let label_str = SharedString::from(label.to_string());
    let has_label = !label.is_empty();

    div()
        .id(id)
        .h(px(32.0))
        .px(px(8.0))
        .rounded(px(6.0))
        .flex()
        .items_center()
        .gap(px(4.0))
        .text_size(px(13.0))
        .text_color(Theme::text_secondary())
        .hover(|s| s.bg(Theme::bg_hover()).text_color(Theme::text_primary()))
        .cursor_pointer()
        .child(div().text_size(px(14.0)).child(icon_str))
        .when(has_label, |e| {
            e.child(div().text_size(px(12.0)).child(label_str))
        })
}

fn divider() -> impl IntoElement {
    div()
        .w(px(1.0))
        .h(px(24.0))
        .bg(Theme::border_default())
        .mx(px(4.0))
}
