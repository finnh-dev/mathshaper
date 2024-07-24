use nih_plug_vizia::vizia::{
    prelude::*,
    vg::{self, Color},
};
use rand::random;

const TABLE_SIZE: usize = 512;


pub struct ShaperView {
    shape: Box<[f32]>,
}

impl ShaperView {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        let shape: Box<[f32]> = (0..TABLE_SIZE).map(|_| random::<f32>() - 0.5).collect();
        Self { shape }.build(cx, |_cx| ())
    }
}

impl View for ShaperView {
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        let x_step = bounds.w / (TABLE_SIZE as f32 - 1.0);

        let grid_color = vg::Paint::color(Color::rgb(255, 255, 255)).with_line_width(1.0);
        let mut grid = vg::Path::new();
        grid.move_to(bounds.x + bounds.w / 2.0, bounds.y + 0.0);
        grid.line_to(bounds.x + bounds.w / 2.0, bounds.y + bounds.h);
        grid.move_to(bounds.x + 0.0, bounds.y + bounds.h / 2.0);
        grid.line_to(bounds.x + bounds.w, bounds.y + bounds.h / 2.0);

        canvas.stroke_path(&grid, &grid_color);

        let plot_color = vg::Paint::color(Color::rgb(0, 255, 0)).with_line_width(1.0);
        let mut plot = vg::Path::new();
        plot.move_to(bounds.x, bounds.y + (bounds.h / 2.0) - ((bounds.h / 2.0) * self.shape[0]));
        for (i, y) in self.shape.iter().enumerate() {
            plot.line_to(bounds.x + (i as f32 * x_step), bounds.y + (bounds.h / 2.0) - ((bounds.h / 2.0) * y));
        }
        canvas.stroke_path(&plot, &plot_color);


    }
}
