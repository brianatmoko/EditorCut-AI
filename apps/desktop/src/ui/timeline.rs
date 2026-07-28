use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    div, prelude::*, px, Context, IntoElement, MouseButton, MouseDownEvent, ParentElement,
    Render, SharedString, Styled, Window,
};

use crate::state::EditorState;
use crate::ui::theme::Theme;

const PIXELS_PER_SECOND: f32 = 50.0;
const TRACK_HEIGHT: f32 = 48.0;
const LABEL_WIDTH: f32 = 60.0;

pub struct Timeline {
    editor: Rc<RefCell<EditorState>>,
    is_playing: bool,
    current_time_secs: f64,
    total_duration_secs: f64,
    zoom_level: f64,
    scroll_left: f64,
}

impl Timeline {
    pub fn new(editor: Rc<RefCell<EditorState>>) -> Self {
        Self {
            editor,
            is_playing: false,
            current_time_secs: 0.0,
            total_duration_secs: 60.0,
            zoom_level: 1.0,
            scroll_left: 0.0,
        }
    }

    fn time_to_x(&self, secs: f64) -> f32 {
        (secs * PIXELS_PER_SECOND as f64 * self.zoom_level) as f32
            - self.scroll_left as f32 + LABEL_WIDTH
    }

    fn x_to_time(&self, x: f32) -> f64 {
        ((x - LABEL_WIDTH + self.scroll_left as f32)
            / (PIXELS_PER_SECOND * self.zoom_level as f32)) as f64
    }

    fn format_time(secs: f64) -> String {
        if secs < 0.0 { return "0:00".to_string(); }
        let total = secs as u32;
        let m = total / 60;
        let s = total % 60;
        format!("{}:{:02}", m, s)
    }
}

impl Render for Timeline {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut track_infos: Vec<TrackInfo> = Vec::new();

        {
            let editor = self.editor.borrow();

            let anim = editor.animator.borrow();
            self.is_playing = anim.is_playing();
            self.current_time_secs = anim.time();
            self.total_duration_secs = anim.total_duration();
            drop(anim);

            if let Some(scene) = editor.current_scene() {
                let mut track_idx = 0u32;

                for track in &scene.tracks.overlay {
                    track_idx += 1;
                    let els: Vec<ElementBlock> = track.elements().iter().map(|e| {
                        let b = e.base();
                        ElementBlock {
                            name: b.name.clone(),
                            start_x: self.time_to_x(b.start_time.to_seconds_f64()),
                            width: (b.duration.to_seconds_f64() * PIXELS_PER_SECOND as f64 * self.zoom_level) as f32,
                            color: Theme::track_video(),
                        }
                    }).collect();
                    track_infos.push(TrackInfo {
                        name: format!("O{}", track_idx),
                        kind: "overlay".to_string(),
                        color: Theme::track_video(),
                        elements: els,
                    });
                }

                {
                    let track = &scene.tracks.main;
                    let els: Vec<ElementBlock> = track.elements().iter().map(|e| {
                        let b = e.base();
                        ElementBlock {
                            name: b.name.clone(),
                            start_x: self.time_to_x(b.start_time.to_seconds_f64()),
                            width: (b.duration.to_seconds_f64() * PIXELS_PER_SECOND as f64 * self.zoom_level) as f32,
                            color: Theme::track_video(),
                        }
                    }).collect();
                    track_infos.push(TrackInfo {
                        name: "V1".to_string(),
                        kind: "video".to_string(),
                        color: Theme::track_video(),
                        elements: els,
                    });
                }

                for track in &scene.tracks.audio {
                    track_idx += 1;
                    let els: Vec<ElementBlock> = track.elements().iter().map(|e| {
                        let b = e.base();
                        ElementBlock {
                            name: b.name.clone(),
                            start_x: self.time_to_x(b.start_time.to_seconds_f64()),
                            width: (b.duration.to_seconds_f64() * PIXELS_PER_SECOND as f64 * self.zoom_level) as f32,
                            color: Theme::track_audio(),
                        }
                    }).collect();
                    track_infos.push(TrackInfo {
                        name: format!("A{}", track_idx),
                        kind: "audio".to_string(),
                        color: Theme::track_audio(),
                        elements: els,
                    });
                }
            }
        }

        if track_infos.is_empty() {
            for (_i, info) in [("V1", "video", Theme::track_video()), ("A1", "audio", Theme::track_audio()), ("FX", "effect", Theme::track_effects())].iter().enumerate() {
                track_infos.push(TrackInfo {
                    name: info.0.to_string(),
                    kind: info.1.to_string(),
                    color: info.2,
                    elements: vec![],
                });
            }
        }

        let playhead_x = self.time_to_x(self.current_time_secs);
        let track_count = track_infos.len();

        div()
            .h(px(240.0))
            .w_full()
            .bg(Theme::bg_surface())
            .border_t_1()
            .border_color(Theme::border_subtle())
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(timeline_header(track_count, self.current_time_secs, self.total_duration_secs))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .overflow_x_hidden()
                    .child(timeline_ruler(
                        self.current_time_secs,
                        self.zoom_level,
                        self.scroll_left,
                    ))
                    .children(track_infos.iter().map(|info| {
                        render_track_row(info, playhead_x)
                    }))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, event: &MouseDownEvent, _window, _cx| {
                        let time = this.x_to_time(event.position.x.into());
                        if time >= 0.0 {
                            let editor = this.editor.borrow_mut();
                            let mut anim = editor.animator.borrow_mut();
                            anim.seek(time);
                        }
                    })),
            )
    }
}

fn timeline_header(track_count: usize, current_secs: f64, total_secs: f64) -> impl IntoElement {
    div()
        .h(px(40.0))
        .bg(Theme::bg_elevated())
        .border_b_1()
        .border_color(Theme::border_subtle())
        .flex()
        .items_center()
        .px(px(12.0))
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(Theme::text_primary())
                        .child(SharedString::from("TIMELINE")),
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(Theme::text_disabled())
                        .child(SharedString::from(format!(
                            "{} tracks | {} / {}",
                            track_count,
                            Timeline::format_time(current_secs),
                            Timeline::format_time(total_secs)
                        ))),
                ),
        )
}

fn timeline_ruler(_playhead_secs: f64, zoom: f64, scroll: f64) -> impl IntoElement {
    let px_per_sec = PIXELS_PER_SECOND * zoom as f32;
    let step = if zoom < 0.5 { 10.0 } else if zoom < 1.0 { 5.0 } else { 1.0 };
    let scroll_secs = scroll as f32 / px_per_sec;
    let start_sec = (scroll_secs / step).floor() as i32 * step as i32;
    let count = 30i32;
    let start_x = start_sec as f32 * px_per_sec - scroll as f32 + LABEL_WIDTH;

    div()
        .h(px(24.0))
        .bg(Theme::bg_base())
        .border_b_1()
        .border_color(Theme::border_subtle())
        .flex()
        .items_center()
        .relative()
        .child(
            div()
                .w(px(LABEL_WIDTH))
                .h_full()
                .bg(Theme::bg_elevated())
                .border_r_1()
                .border_color(Theme::border_subtle())
                .flex()
                .items_center()
                .px(px(8.0))
                .child(
                    div()
                        .text_size(px(8.0))
                        .text_color(Theme::text_disabled())
                        .child(SharedString::from("sec")),
                ),
        )
        .children((0..count).map(|i| {
            let sec = start_sec + i as i32 * step as i32;
            let x = start_x + i as f32 * step as f32 * px_per_sec;
            if sec >= 0 && sec % 5 == 0 {
                let label = SharedString::from(format!("{}", sec));
                div()
                    .absolute()
                    .left(px(x))
                    .top(px(0.0))
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .h(px(12.0))
                            .w(px(1.0))
                            .bg(Theme::text_disabled())
                            .ml(px(4.0)),
                    )
                    .child(
                        div()
                            .mt(px(2.0))
                            .text_size(px(9.0))
                            .text_color(Theme::text_disabled())
                            .child(label),
                    )
            } else if sec >= 0 {
                div()
                    .absolute()
                    .left(px(x))
                    .top(px(4.0))
                    .h(px(6.0))
                    .w(px(1.0))
                    .bg(Theme::text_disabled())
            } else {
                div()
            }
        }))
}

fn render_track_row(info: &TrackInfo, playhead_x: f32) -> impl IntoElement {
    div()
        .h(px(TRACK_HEIGHT))
        .w_full()
        .flex()
        .border_b_1()
        .border_color(Theme::border_subtle())
        .child(
            div()
                .w(px(LABEL_WIDTH))
                .h_full()
                .bg(Theme::bg_elevated())
                .border_r_1()
                .border_color(Theme::border_subtle())
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(info.color)
                        .font_weight(gpui::FontWeight::BOLD)
                        .child(SharedString::from(info.name.clone())),
                )
                .child(
                    div()
                        .text_size(px(8.0))
                        .text_color(Theme::text_disabled())
                        .child(SharedString::from(info.kind.clone())),
                ),
        )
        .child(
            div()
                .flex_1()
                .h_full()
                .bg(Theme::bg_base())
                .relative()
                .child(
                    div()
                        .h_full()
                        .w(px(2.0))
                        .bg(Theme::playhead())
                        .absolute()
                        .left(px(playhead_x))
                        .top(px(0.0)),
                )
                .children(info.elements.iter().map(|el| {
                    render_element_block(el)
                })),
        )
}

fn render_element_block(el: &ElementBlock) -> impl IntoElement {
    let name = SharedString::from(el.name.clone());
    div()
        .absolute()
        .left(px(el.start_x))
        .top(px(4.0))
        .h(px(TRACK_HEIGHT - 8.0))
        .w(px(el.width.max(4.0)))
        .bg(el.color)
        .rounded(px(4.0))
        .flex()
        .items_center()
        .px(px(6.0))
        .overflow_hidden()
        .child(
            div()
                .text_size(px(9.0))
                .text_color(gpui::white())
                .child(name),
        )
}

struct TrackInfo {
    name: String,
    kind: String,
    color: gpui::Rgba,
    elements: Vec<ElementBlock>,
}

struct ElementBlock {
    name: String,
    start_x: f32,
    width: f32,
    color: gpui::Rgba,
}
