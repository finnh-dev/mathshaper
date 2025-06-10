use std::sync::{Arc, Mutex};

use nih_plug::prelude::AtomicF32;
use vizia_plug::vizia::{
    prelude::*,
    vg::{self, Point},
};

use crate::MathshaperParams;

use super::Shaper;

const SC_DEFAULT: f32 = 1.0; // TODO: maybe add sidechain slider for visual feedback

pub struct ShaperView {
    shaper: Arc<Mutex<Arc<Shaper>>>,
    _peak_max: Arc<AtomicF32>,
    _peak_min: Arc<AtomicF32>,
    params: Arc<MathshaperParams>,
}

impl ShaperView {
    const RESOLUTION: usize = 400;
    const RANGE_X: f32 = 3.0;
    const RANGE_Y: f32 = 3.0;

    pub fn new<LShaper, LPeakMax, LPeakMin, LParams, LString>(
        cx: &mut Context,
        shaper: LShaper,
        peak_max: LPeakMax,
        peak_min: LPeakMin,
        params: LParams,
        input_string: LString,
    ) -> Handle<Self>
    where
        LShaper: Lens<Target = Arc<Mutex<Arc<Shaper>>>>,
        LPeakMax: Lens<Target = Arc<AtomicF32>>,
        LPeakMin: Lens<Target = Arc<AtomicF32>>,
        LParams: Lens<Target = Arc<MathshaperParams>>,
        LString: Lens<Target = String>,
    {
        Self {
            shaper: shaper.get(cx),
            _peak_max: peak_max.get(cx),
            _peak_min: peak_min.get(cx),
            params: params.get(cx),
        }
        .build(cx, |_cx| ())
        .bind(
            params.map(|p| (p.a.value(), p.b.value(), p.c.value(), p.d.value())),
            |mut handle, _| handle.needs_redraw(),
        )
        .bind(input_string, |mut handle, _| handle.needs_redraw())
    }
}

impl View for ShaperView {
    fn element(&self) -> Option<&'static str> {
        Some("shaper_view")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &vizia_plug::vizia::vg::Canvas) {
        // Draw Grid
        let bounds = cx.bounds();
        let line_width = cx.scale_factor() * 1.5;
        let mut grid_paint = vg::Paint::default();
        grid_paint
            .set_color(Color::rgb(255, 255, 255))
            .set_style(vg::PaintStyle::Stroke)
            .set_stroke_width(line_width);
        let mut grid = vg::Path::new();
        grid.move_to(Point::new(bounds.x + bounds.w / 2.0, bounds.y + 0.0));
        grid.line_to(Point::new(bounds.x + bounds.w / 2.0, bounds.y + bounds.h));
        grid.move_to(Point::new(bounds.x + 0.0, bounds.y + bounds.h / 2.0));
        grid.line_to(Point::new(bounds.x + bounds.w, bounds.y + bounds.h / 2.0));
        canvas.draw_path(&grid, &grid_paint);

        // Draw Plot
        let func = self.shaper.lock().unwrap(); // TODO: Error Handling
        {
            let (a, b, c, d) = (
                self.params.a.value(),
                self.params.b.value(),
                self.params.c.value(),
                self.params.d.value(),
            );
            let bounds = cx.bounds();
            let line_width = cx.scale_factor() * 1.5;
            let x_step = bounds.w / Self::RESOLUTION as f32;

            let mut plot_paint = vg::Paint::default();
            plot_paint
                .set_color(Color::rgb(0, 255, 0))
                .set_style(vg::PaintStyle::Stroke)
                .set_stroke_width(line_width);

            let mut plot = vg::Path::new();
            plot.move_to(Point::new(
                bounds.x,
                bounds.y + (bounds.h / 2.0)
                    - ((bounds.h / Self::RANGE_Y)
                        * func(-(Self::RANGE_X / 2.0), SC_DEFAULT, a, b, c, d)),
            ));

            for i in 0..Self::RESOLUTION {
                let y = func(
                    (i as f32 / Self::RESOLUTION as f32) * Self::RANGE_X - (Self::RANGE_X / 2.0),
                    SC_DEFAULT,
                    a,
                    b,
                    c,
                    d,
                );
                plot.line_to(Point::new(
                    bounds.x + (i as f32 * x_step),
                    bounds.y + (bounds.h / 2.0) - ((bounds.h / Self::RANGE_Y) * y),
                ));
            }
            canvas.draw_path(&plot, &plot_paint);
        }

        // // Draw Peaks
        // let peaks_paint =
        //     vg::Paint::color(Color::from_argb(0, 255, 255, 64)).with_line_width(line_width);
        // let mut peaks = vg::Path::new();
        // let x_max = bounds.x + (bounds.w / 2.0) * (1.0 + self.peak_max.load(Ordering::Relaxed));
        // let x_min = bounds.x + (bounds.w / 2.0) * (1.0 + self.peak_min.load(Ordering::Relaxed));
        // let y_max = bounds.y + bounds.h;
        // let y_min = bounds.y;
        // peaks.move_to(x_max, y_min);
        // peaks.line_to(x_max, y_max);
        // peaks.move_to(x_min, y_min);
        // peaks.line_to(x_min, y_max);

        // canvas.stroke_path(&peaks, &peaks_paint);
    }
}
