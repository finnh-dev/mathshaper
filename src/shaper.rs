use std::usize;

use evalexpr::{
    build_operator_tree, context_map, ContextWithMutableVariables, EvalexprError, HashMapContext,
    Value,
};
use nih_plug_vizia::vizia::{
    context::DrawContext,
    vg::{self, Color},
    view::Canvas,
};

use crate::math::chebychev::chebychev;

pub struct Shaper<const SIZE: usize> {
    table: Box<[f32]>,
    context: HashMapContext,
}

impl<const SIZE: usize> Default for Shaper<SIZE> {
    fn default() -> Self {
        let table: Box<[f32]> = (0..SIZE).map(Shaper::<SIZE>::value).collect();
        Self {
            table,
            context: Shaper::<SIZE>::default_context(),
        }
    }
}

impl<const SIZE: usize> Shaper<SIZE> {
    const INDEX_MAX: usize = SIZE - 1;
    const INPUT_SAMPLE_MAX: f32 = 1.0;
    const INPUT_SAMPLE_MIN: f32 = -Self::INPUT_SAMPLE_MAX;
    const STEP: f32 = 2.0 / Self::INDEX_MAX as f32;

    fn default_context() -> HashMapContext {
        context_map! {
            "PI" => evalexpr::Value::Float(std::f64::consts::PI),
            "Cheb" => Function::new(|args| {
                let args = args.as_tuple()?;
                if args.len() != 2 {
                    return Err(EvalexprError::expected_fixed_len_tuple(2, Value::Int(args.len() as i64)));
                }
                let Value::Float(x) = &args[0] else {
                    return Err(EvalexprError::expected_float(args[0].clone()));
                };
                let Value::Int(n) = &args[1] else {
                    return Err(EvalexprError::expected_int(args[1].clone()));
                };

                Ok(Value::Float(chebychev(x, n)?))
            }),
        }
        .expect("Failed to initialize contex map!")
    }

    #[allow(unused)]
    fn new(prompt: &str) -> Result<Self, EvalexprError> {
        let mut this = Self::default();
        this.prompt(prompt)?;
        Ok(this)
    }

    pub fn process(&self, x: f32) -> f32 {
        self.get_interpolated(x)
    }

    pub fn value(index: usize) -> f32 {
        Self::INPUT_SAMPLE_MIN + (index as f32 * Self::STEP)
    }

    fn get_interpolated(&self, x: f32) -> f32 {
        if x >= 1.0 {
            return self.table[Self::INDEX_MAX];
        }
        if x <= 1.0 {
            return self.table[0];
        }

        let true_index = (x + 1.0) * 0.5 * Self::INDEX_MAX as f32;
        let index = true_index.floor();
        let lerp_factor = true_index - index;
        let index = index as usize;
        let value_1 = self.table[index];
        let value_2 = self.table[index + 1];

        value_1 + lerp_factor * (value_2 - value_1)
    }

    pub fn normalize(&mut self) {
        let max_abs = self
            .table
            .iter()
            .map(|&value| value.abs())
            .max_by(|a, b| a.partial_cmp(b).expect("NaN Error"))
            .expect("Table can't be empty");

        for value in self.table.iter_mut() {
            *value = *value / max_abs;
        }
    }

    pub fn prompt(&mut self, prompt: &str) -> Result<(), EvalexprError> {
        let node = build_operator_tree(prompt)?;
        for (i, val) in self.table.iter_mut().enumerate() {
            self.context
                .set_value(
                    "x".to_owned(),
                    evalexpr::Value::Float(Self::value(i) as f64),
                )
                .expect("Failed to set context!");
            *val = node.eval_float_with_context(&self.context)? as f32;
        }
        Ok(())
    }

    pub fn display(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();
        let line_width = cx.scale_factor() * 1.5;
        let x_step = bounds.w / Self::INDEX_MAX as f32;

        let plot_paint = vg::Paint::color(Color::rgb(0, 255, 0)).with_line_width(line_width);
        let mut plot = vg::Path::new();
        plot.move_to(
            bounds.x,
            bounds.y + (bounds.h / 2.0) - ((bounds.h / 2.0) * self.table[0]),
        );
        for (i, y) in self.table.iter().enumerate() {
            plot.line_to(
                bounds.x + (i as f32 * x_step),
                bounds.y + (bounds.h / 2.0) - ((bounds.h / 2.0) * y),
            );
        }
        canvas.stroke_path(&plot, &plot_paint);
    }
}

// #[cfg(test)]
// mod test {

//     use plotly::{Plot, Scatter};
//     use rand::random;

//     use crate::shaper::Shaper as GenericShaper;

//     const TABLE_SIZE: usize = 32;

//     type Shaper = GenericShaper<TABLE_SIZE>;

//     #[test]
//     fn test_floats() {
//         assert_eq!(f32::MIN, (f32::MIN as f64) as f32);
//         assert_ne!((f32::MIN / 1.24436) as f64, f32::MIN as f64 / 1.24436); // NOT EQUAL!
//         assert_ne!(
//             (f32::MIN / 1.24436_f32) as f64,
//             f32::MIN as f64 / 1.24436_f32 as f64
//         ); // NOT EQUAL!
//         let step = f32::from_bits(0b00000000000000000000000000000001);
//         println!("{}", step);
//     }

//     #[test]
//     fn print_default_lut() {}

//     #[test]
//     fn test_value_to_index() {
//         let index = Shaper::index(Shaper::INPUT_SAMPLE_MIN);
//         println!("Testing if index is 0 with input SAMPE_MIN:");
//         println!("\tindex:          {}", index);
//         println!("\texpected index: {}", 0);
//         assert_eq!(index, 0);

//         let index = Shaper::index(Shaper::INPUT_SAMPLE_MAX);
//         println!("Testing if index is max index with input SAMPLE_MAX:");
//         println!("\tindex:          {}", index);
//         println!("\texpected index: {}", Shaper::INDEX_MAX);
//         assert_eq!(index, Shaper::INDEX_MAX);

//         let index = Shaper::index(Shaper::INPUT_SAMPLE_MAX + 2.0);
//         println!("Testing if index is max index with input out of range:");
//         println!("\tindex:          {}", index);
//         println!("\texpected index: {}", Shaper::INDEX_MAX);
//         assert_eq!(index, Shaper::INDEX_MAX);
//     }

//     #[test]
//     fn test_interpolate() {
//         let shaper = Shaper::default();
//         for _ in 0..1000 {
//             let x = Shaper::INPUT_SAMPLE_MIN + random::<f32>() + random::<f32>();
//             let y = shaper.lerp(Shaper::index(x), x);
//             assert_eq!(x, y)
//         }
//     }

//     #[test]
//     fn test_prompt() {
//         let x_trace: Vec<f32> = (0..TABLE_SIZE).map(Shaper::value).collect();
//         let mut shaper = Shaper::default();
//         let default_trace = Scatter::new(x_trace.clone(), shaper.table.clone().into())
//             .mode(plotly::common::Mode::Markers)
//             .name("LUT Default");
//         shaper.prompt("math::sin(3 * PI * x)").unwrap();
//         let prompt_trace = Scatter::new(x_trace, shaper.table.clone().into())
//             .mode(plotly::common::Mode::Markers)
//             .name("LUT Prompt");
//         let mut random_x = Vec::new();
//         let mut random_y = Vec::new();
//         for _ in 0..8192 {
//             let x = Shaper::INPUT_SAMPLE_MIN + (2.0 * random::<f32>());
//             random_x.push(x);
//             let y = shaper.process(x);
//             random_y.push(y);
//         }

//         let random_trace = Scatter::new(random_x, random_y)
//             .mode(plotly::common::Mode::Markers)
//             .name("Random");

//         let mut plot = Plot::new();
//         plot.add_traces(vec![default_trace, prompt_trace, random_trace]);
//         plot.write_html("plot.html");
//     }
// }
