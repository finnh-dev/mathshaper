use std::usize;

use evalexpr::{build_operator_tree, ContextWithMutableVariables, EvalexprError, HashMapContext};
use nih_plug_vizia::vizia::{layout::BoundingBox, vg::{self, Color}, view::Canvas};

pub struct Shaper<const SIZE: usize> {
    baked_function: Box<[f32]>,
    context: HashMapContext,
}

impl<const SIZE: usize> Default for Shaper<SIZE> {
    fn default() -> Self {
        let table: Box<[f32]> = (0..SIZE).map(Shaper::<SIZE>::value).collect();
        Self {
            baked_function: table,
            context: Shaper::<SIZE>::default_context(),
        }
    }
}

impl<const SIZE: usize> Shaper<SIZE> {
    const INDEX_MAX: usize = SIZE - 1;
    #[allow(unused)]
    const SAMPLE_MAX: f32 = 1.0;
    const SAMPLE_MIN: f32 = -1.0;
    const STEP: f32 = 2.0 / Self::INDEX_MAX as f32;

    fn default_context() -> HashMapContext {
        let mut context = HashMapContext::default();
        context
            .set_value(
                "PI".to_owned(),
                evalexpr::Value::Float(std::f64::consts::PI),
            )
            .expect("Default constant assignment should not panic.");

        context
    }

    #[allow(unused)]
    fn new(prompt: &str) -> Result<Self, EvalexprError> {
        let mut this = Self::default();
        this.prompt(prompt)?;
        Ok(this)
    }

    #[allow(unused)] // TODO: remove
    pub fn process(&self, x: f32) -> f32 {
        self.lerp(Self::index(x), x)
    }

    fn index(value: f32) -> usize {
        (((value - Self::SAMPLE_MIN) / Self::STEP) as usize).min(Self::INDEX_MAX)
    }

    pub fn value(index: usize) -> f32 {
        Self::SAMPLE_MIN + (index as f32 * Self::STEP)
    }

    fn lerp(&self, index: usize, x: f32) -> f32 {
        if index == Self::INDEX_MAX {
            return self.baked_function[Self::INDEX_MAX];
        };
        let higher_index = index + 1;
        let y1 = self.baked_function[index];
        let x1 = Self::value(index);
        let y2 = self.baked_function[higher_index];
        let x2 = Self::value(higher_index);

        let delta_y = y1 - y2;
        let delta_x = x1 - x2;
        let position = (x - x1) / delta_x;
        y1 + (delta_y * position)
    }

    #[allow(unused)]
    pub fn prompt(&mut self, prompt: &str) -> Result<(), EvalexprError> {
        let node = build_operator_tree(prompt)?;
        for (i, val) in self.baked_function.iter_mut().enumerate() {
            self.context.set_value(
                "x".to_owned(),
                evalexpr::Value::Float(Self::value(i) as f64),
            );
            *val = node.eval_float_with_context(&self.context)? as f32;
        }
        Ok(())
    }

    pub fn display(&self, bounds: &BoundingBox, canvas: &mut Canvas) {
        let x_step = bounds.w / Self::INDEX_MAX as f32;

        let plot_color = vg::Paint::color(Color::rgb(0, 255, 0)).with_line_width(1.0);
        let mut plot = vg::Path::new();
        plot.move_to(
            bounds.x,
            bounds.y + (bounds.h / 2.0) - ((bounds.h / 2.0) * self.baked_function[0]),
        );
        for (i, y) in self.baked_function.iter().enumerate() {
            plot.line_to(
                bounds.x + (i as f32 * x_step),
                bounds.y + (bounds.h / 2.0) - ((bounds.h / 2.0) * y),
            );
        }
        canvas.stroke_path(&plot, &plot_color);
    }
}

#[cfg(test)]
mod test {

    use plotly::{Plot, Scatter};
    use rand::random;

    use crate::shaper::Shaper as GenericShaper;

    const TABLE_SIZE: usize = 32;

    type Shaper = GenericShaper<TABLE_SIZE>;

    #[test]
    fn test_floats() {
        assert_eq!(f32::MIN, (f32::MIN as f64) as f32);
        assert_ne!((f32::MIN / 1.24436) as f64, f32::MIN as f64 / 1.24436); // NOT EQUAL!
        assert_ne!(
            (f32::MIN / 1.24436_f32) as f64,
            f32::MIN as f64 / 1.24436_f32 as f64
        ); // NOT EQUAL!
        let step = f32::from_bits(0b00000000000000000000000000000001);
        println!("{}", step);
    }

    #[test]
    fn print_default_lut() {}

    #[test]
    fn test_value_to_index() {
        let index = Shaper::index(Shaper::SAMPLE_MIN);
        println!("Testing if index is 0 with input SAMPE_MIN:");
        println!("\tindex:          {}", index);
        println!("\texpected index: {}", 0);
        assert_eq!(index, 0);

        let index = Shaper::index(Shaper::SAMPLE_MAX);
        println!("Testing if index is max index with input SAMPLE_MAX:");
        println!("\tindex:          {}", index);
        println!("\texpected index: {}", Shaper::INDEX_MAX);
        assert_eq!(index, Shaper::INDEX_MAX);

        let index = Shaper::index(Shaper::SAMPLE_MAX + 2.0);
        println!("Testing if index is max index with input out of range:");
        println!("\tindex:          {}", index);
        println!("\texpected index: {}", Shaper::INDEX_MAX);
        assert_eq!(index, Shaper::INDEX_MAX);
    }

    #[test]
    fn test_interpolate() {
        let shaper = Shaper::default();
        for _ in 0..1000 {
            let x = Shaper::SAMPLE_MIN + random::<f32>() + random::<f32>();
            let y = shaper.lerp(Shaper::index(x), x);
            assert_eq!(x, y)
        }
    }

    #[test]
    fn test_prompt() {
        let x_trace: Vec<f32> = (0..TABLE_SIZE).map(Shaper::value).collect();
        let mut shaper = Shaper::default();
        let default_trace = Scatter::new(x_trace.clone(), shaper.baked_function.clone().into())
            .mode(plotly::common::Mode::Markers)
            .name("LUT Default");
        shaper.prompt("math::sin(3 * PI * x)").unwrap();
        let prompt_trace = Scatter::new(x_trace, shaper.baked_function.clone().into())
            .mode(plotly::common::Mode::Markers)
            .name("LUT Prompt");
        let mut random_x = Vec::new();
        let mut random_y = Vec::new();
        for _ in 0..8192 {
            let x = Shaper::SAMPLE_MIN + (2.0 * random::<f32>());
            random_x.push(x);
            let y = shaper.process(x);
            random_y.push(y);
        }

        let random_trace = Scatter::new(random_x, random_y)
            .mode(plotly::common::Mode::Markers)
            .name("Random");

        let mut plot = Plot::new();
        plot.add_traces(vec![default_trace, prompt_trace, random_trace]);
        plot.write_html("plot.html");
    }
}
