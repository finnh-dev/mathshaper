use std::sync::{Arc, Mutex};

use nih_plug_vizia::vizia::{
    prelude::*,
    vg::{self, Color},
};

use crate::editor::Shaper;

pub struct ShaperView {
    shape: Arc<Mutex<Shaper>>,
}

impl ShaperView {
    pub fn new(cx: &mut Context, shape: Arc<Mutex<Shaper>>) -> Handle<Self> {
        Self { shape }.build(cx, |_cx| ())
    }
}

impl View for ShaperView {
    fn element(&self) -> Option<&'static str> {
        Some("shaper_view")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        let line_width = cx.scale_factor() * 1.5;
        let grid_paint = vg::Paint::color(Color::rgb(255, 255, 255)).with_line_width(line_width);
        let mut grid = vg::Path::new();
        grid.move_to(bounds.x + bounds.w / 2.0, bounds.y + 0.0);
        grid.line_to(bounds.x + bounds.w / 2.0, bounds.y + bounds.h);
        grid.move_to(bounds.x + 0.0, bounds.y + bounds.h / 2.0);
        grid.line_to(bounds.x + bounds.w, bounds.y + bounds.h / 2.0);

        canvas.stroke_path(&grid, &grid_paint);
        let lock = self.shape.lock().unwrap(); // TODO: Error Handling
        lock.display(cx, canvas);
    }
}
