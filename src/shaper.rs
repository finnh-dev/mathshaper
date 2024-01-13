use evalexpr::{build_operator_tree, ContextWithMutableVariables, EvalexprError, HashMapContext};

const TABLE_SIZE: usize = 32;
const INDEX_MAX: usize = TABLE_SIZE - 1;
// const F32_RANGE: f64 = f32::MAX as f64 - f32::MIN as f64;
#[allow(unused)]
const SAMPLE_MAX: f32 = 1.0;
const SAMPLE_MIN: f32 = -1.0;
const STEP: f32 = 2.0 / INDEX_MAX as f32;

pub struct Shaper {
    baked_function: Box<[f32]>,
    context: HashMapContext,
}

impl Default for Shaper {
    fn default() -> Self {
        let table: Box<[f32]> = (0..TABLE_SIZE).map(Shaper::value).collect();
        Self {
            baked_function: table,
            context: Shaper::default_context(),
        }
    }
}

impl Shaper {
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
        (((value - SAMPLE_MIN) / STEP) as usize).min(INDEX_MAX)
    }

    fn value(index: usize) -> f32 {
        SAMPLE_MIN + (index as f32 * STEP)
    }

    fn lerp(&self, index: usize, x: f32) -> f32 {
        if index == INDEX_MAX {
            return self.baked_function[INDEX_MAX];
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
                evalexpr::Value::Float(Shaper::value(i) as f64),
            );
            *val = node.eval_float_with_context(&self.context)? as f32;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use plotly::{Plot, Scatter};
    use rand::random;

    use crate::shaper::{INDEX_MAX, SAMPLE_MAX, SAMPLE_MIN};

    use super::{Shaper, TABLE_SIZE};

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
        let index = Shaper::index(SAMPLE_MIN);
        println!("Testing if index is 0 with input SAMPE_MIN:");
        println!("\tindex:          {}", index);
        println!("\texpected index: {}", 0);
        assert_eq!(index, 0);

        let index = Shaper::index(SAMPLE_MAX);
        println!("Testing if index is max index with input SAMPLE_MAX:");
        println!("\tindex:          {}", index);
        println!("\texpected index: {}", INDEX_MAX);
        assert_eq!(index, INDEX_MAX);

        let index = Shaper::index(SAMPLE_MAX + 2.0);
        println!("Testing if index is max index with input out of range:");
        println!("\tindex:          {}", index);
        println!("\texpected index: {}", INDEX_MAX);
        assert_eq!(index, INDEX_MAX);
    }

    #[test]
    fn test_interpolate() {
        let shaper = Shaper::default();
        // let mut plot = Plot::new();
        // let lut_vec = shaper.lut.iter().map(|x| x.clone()).collect::<Vec<f32>>();
        // let x_range = 0..TABLE_SIZE;
        // let x_vec = x_range.collect::<Vec<usize>>().iter().map(|x| Shaper::value_from_index(x.to_owned())).collect::<Vec<f32>>();
        // let trace_lut = Scatter::new(x_vec.clone(), lut_vec);
        // plot.add_trace(trace_lut);

        for _ in 0..1000 {
            let x = SAMPLE_MIN + random::<f32>() + random::<f32>();
            let y = shaper.lerp(Shaper::index(x), x);
            // println!("{:<2}: x={}, y={}", i, x, y);
            // println!("inaccuracy: {}", x - y);
            assert_eq!(x, y)
        }

        // let mut shaper = Shaper::default();
        // for i in 0..shaper.lut.len() {
        //     shaper.lut[i] = f32::sin(std::f32::consts::PI * Shaper::value_from_index(i));
        // }

        // // let mut interpolated_sin: Vec<(f32, f32)> = Vec::new();
        // // let expected_sin = x_vec.iter().map(|x| f32::sin(std::f32::consts::PI * x)).collect::<Vec<f32>>();
        // // let expected_sin_trace = Scatter::new(x_vec, expected_sin).name("expected");

        // let mut inaccuracies : Vec<f32> = Vec::new();

        // let mut max_inaccuracy = 0.0;
        // let mut most_inaccurate = (0, 0.0, 0.0, 0.0);
        // for i in 0..100000000 {
        //     let x = SAMPE_MIN + random::<f32>() + random::<f32>();
        //     let y = shaper.interpolate(Shaper::nearest_lower_index(x), x);
        //     // interpolated_sin.push((x, y));
        //     let expected = f32::sin(std::f32::consts::PI * x);
        //     // println!("{:<2}:\n\tx={}\n\ty={}", i, x, y);

        //     let inaccuracy = expected - y;
        //     inaccuracies.push(inaccuracy);
        //     if inaccuracy > max_inaccuracy {
        //         max_inaccuracy = inaccuracy;
        //         most_inaccurate = (Shaper::nearest_lower_index(x), x, expected, y);
        //     };
        //     // println!("inaccuracy: {}", x - y);
        // }
        // println!("max inaccuracy: {}", max_inaccuracy);
        // println!("most inaccurate: {:?}", most_inaccurate);
        // let mean_inaccuracy = inaccuracies.iter().sum::<f32>() / inaccuracies.len() as f32;
        // println!("average: {}", mean_inaccuracy);

        // interpolated_sin.sort_by(|(x1, _), (x2, _)| x1.total_cmp(x2));
        // let (sin_x, sin_y): (Vec<f32>, Vec<f32>) = interpolated_sin.into_iter().unzip();
        // let sin_trace = Scatter::new(sin_x, sin_y).name("sin lut").mode(plotly::common::Mode::Markers);
        // plot.add_trace(sin_trace);
        // plot.add_trace(expected_sin_trace);
        // plot.write_html("plot.html");
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
            let x = SAMPLE_MIN + (2.0 * random::<f32>());
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
