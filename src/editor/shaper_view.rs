use std::sync::{atomic::Ordering, Arc, Mutex};

use nih_plug::prelude::AtomicF32;
use nih_plug_vizia::vizia::{
    prelude::*,
    vg::{self, Color},
};

use crate::editor::Shaper;

pub struct ShaperView {
    shaper: Arc<Mutex<Shaper>>,
    peak_max: Arc<AtomicF32>,
    peak_min: Arc<AtomicF32>,
}

impl ShaperView {
    pub fn new<LShaper, LPeakMax, LPeakMin>(
        cx: &mut Context,
        shaper: LShaper,
        peak_max: LPeakMax,
        peak_min: LPeakMin,
    ) -> Handle<Self>
    where
        LShaper: Lens<Target = Arc<Mutex<Shaper>>>,
        LPeakMax: Lens<Target = Arc<AtomicF32>>,
        LPeakMin: Lens<Target = Arc<AtomicF32>>,
    {
        Self {
            shaper: shaper.get(cx),
            peak_max: peak_max.get(cx),
            peak_min: peak_min.get(cx),
        }
        .build(cx, |_cx| ())
    }
}

impl View for ShaperView {
    fn element(&self) -> Option<&'static str> {
        Some("shaper_view")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        // Draw Grid
        let bounds = cx.bounds();
        let line_width = cx.scale_factor() * 1.5;
        let grid_paint = vg::Paint::color(Color::rgb(255, 255, 255)).with_line_width(line_width);
        let mut grid = vg::Path::new();
        grid.move_to(bounds.x + bounds.w / 2.0, bounds.y + 0.0);
        grid.line_to(bounds.x + bounds.w / 2.0, bounds.y + bounds.h);
        grid.move_to(bounds.x + 0.0, bounds.y + bounds.h / 2.0);
        grid.line_to(bounds.x + bounds.w, bounds.y + bounds.h / 2.0);
        canvas.stroke_path(&grid, &grid_paint);

        // Draw Plot
        let lock = self.shaper.lock().unwrap(); // TODO: Error Handling
        lock.display(cx, canvas);

        // Draw Peaks
        let peaks_paint =
            vg::Paint::color(Color::rgba(0, 255, 255, 64)).with_line_width(line_width);
        let mut peaks = vg::Path::new();
        let x_max = bounds.x + (bounds.w / 2.0) * (1.0 + self.peak_max.load(Ordering::Relaxed));
        let x_min = bounds.x + (bounds.w / 2.0) * (1.0 + self.peak_min.load(Ordering::Relaxed));
        let y_max = bounds.y + bounds.h;
        let y_min = bounds.y;
        peaks.move_to(x_max, y_min);
        peaks.line_to(x_max, y_max);
        peaks.move_to(x_min, y_min);
        peaks.line_to(x_min, y_max);

        canvas.stroke_path(&peaks, &peaks_paint);
    }
}
