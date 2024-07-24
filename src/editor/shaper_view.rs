use nih_plug_vizia::vizia::{
    prelude::*,
    vg::{self, Color},
};

use crate::shaper::Shaper;

const SHAPER_RESOLUTION: usize = 512;

pub struct ShaperView {
    shape: Box<[f32]>,
}

impl ShaperView {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        let shape: Box<[f32]> = (0..SHAPER_RESOLUTION).map(Shaper::value).collect();
        Self { shape }.build(cx, |_cx| ())
    }
}

impl View for ShaperView {
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        let plot_lines_color = vg::Paint::color(Color::rgb(0, 255, 0)).with_line_width(1.0);
        let mut grid = vg::Path::new();
        grid.move_to(bounds.x + bounds.w / 2.0, bounds.y + 0.0);
        grid.line_to(bounds.x + bounds.w / 2.0, bounds.y + bounds.h);
        grid.move_to(bounds.x + 0.0, bounds.y + bounds.h / 2.0);
        grid.line_to(bounds.x + bounds.w, bounds.y + bounds.h / 2.0);

        canvas.stroke_path(&grid, &plot_lines_color);
    }
}
