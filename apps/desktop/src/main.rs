#![allow(unused_imports, unused_variables, dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Entity, Render, Window,
    WindowBounds, WindowOptions,
};

mod state;
mod ui;

use state::EditorState;
use ui::preview::Preview;
use ui::sidebar::Sidebar;
use ui::theme::Theme;
use ui::timeline::Timeline;
use ui::toolbar::Toolbar;

struct EditorApp {
    _editor: Rc<RefCell<EditorState>>,
    toolbar: Entity<Toolbar>,
    sidebar: Entity<Sidebar>,
    preview: Entity<Preview>,
    timeline: Entity<Timeline>,
}

impl EditorApp {
    fn new(cx: &mut Context<Self>) -> Self {
        let editor = Rc::new(RefCell::new(EditorState::new()));
        Self {
            _editor: editor.clone(),
            toolbar: cx.new(|_| Toolbar::new(editor.clone())),
            sidebar: cx.new(|sidebar_cx| Sidebar::new(editor.clone(), sidebar_cx)),
            preview: cx.new(|preview_cx| Preview::new(editor.clone(), preview_cx)),
            timeline: cx.new(|_| Timeline::new(editor)),
        }
    }
}

impl Render for EditorApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(Theme::bg_base())
            .flex()
            .flex_col()
            .child(self.toolbar.clone())
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    .child(self.sidebar.clone())
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .child(self.preview.clone())
                            .child(self.timeline.clone()),
                    ),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1280.), px(720.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| EditorApp::new(cx)),
        )
        .unwrap();
        cx.activate(true);
    });
}
