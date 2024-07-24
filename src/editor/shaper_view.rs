use nih_plug_vizia::vizia::{
    prelude::*,
    vg::{self, Color},
};

use crate::shaper::Shaper as GenericShaper;

const TABLE_SIZE: usize = 512;

type Shaper = GenericShaper<TABLE_SIZE>;

pub struct ShaperView {
    shape: Shaper,
}

impl ShaperView {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        let mut shape = Shaper::default();
        shape.prompt("Cheb(x, 20)").expect("prompt failed!");
        Self { shape }.build(cx, |_cx| ())
    }
}

impl View for ShaperView {
    fn element(&self) -> Option<&'static str> {
        Some("shaper_view")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        let grid_color = vg::Paint::color(Color::rgb(255, 255, 255)).with_line_width(1.0);
        let mut grid = vg::Path::new();
        grid.move_to(bounds.x + bounds.w / 2.0, bounds.y + 0.0);
        grid.line_to(bounds.x + bounds.w / 2.0, bounds.y + bounds.h);
        grid.move_to(bounds.x + 0.0, bounds.y + bounds.h / 2.0);
        grid.line_to(bounds.x + bounds.w, bounds.y + bounds.h / 2.0);

        canvas.stroke_path(&grid, &grid_color);
        self.shape.display(&bounds, canvas);
    }
}
