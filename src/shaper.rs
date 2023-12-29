const TABLE_SIZE: usize = 128;  
const INDEX_MAX: usize = TABLE_SIZE - 1;
const F32_RANGE: f64 = f32::MAX as f64 - f32::MIN as f64;
const STEP: f64 = F32_RANGE / INDEX_MAX as f64;


fn default_lut() -> [f32; TABLE_SIZE] {
    let mut lut = [0.0; TABLE_SIZE];
    for i in 0..lut.len() {
        lut[i] = (f32::MIN as f64 + (STEP * i as f64)) as f32;
    };
    lut
}

struct Shaper {
    lut: [f32; TABLE_SIZE],
}

impl Shaper {
    pub fn calc(x: f32) -> f32 {
        todo!()
    }

    fn nearest_lower_index(value: f32) -> usize {
        ((value as f64 - f32::MIN as f64) / STEP) as usize
    }

    fn value_from_index(index: usize) -> f32 {
        f32::MIN + (index as f64 * STEP) as f32
    }

    fn interpolate(&self, lower_index: usize, x: f32) -> f32 {
        println!("index: {}", lower_index);
        if lower_index == INDEX_MAX {
            return self.lut[INDEX_MAX];
        };
        let y1 = self.lut[lower_index];
        let x1 = Shaper::value_from_index(lower_index);
        let y2 = self.lut[lower_index + 1];
        let x2 = Shaper::value_from_index(lower_index + 1);
        println!("y1: {}\nx1: {}\ny2: {}\nx2: {}", y1, x1, y2, x2);

        let delta_y = y1 - y2;
        println!("delta_y: {}", delta_y);
        let delta_x = x1 - x2;
        println!("delta_x: {}", delta_x);
        let position = (x - x1) / delta_x;
        println!("pos: {}", position);
        y1 + (delta_y * position)
    }
}

impl Default for Shaper {
    fn default() -> Self {
        Self { lut: default_lut()  }
    }
}

#[cfg(test)]
mod test {

    use rand::Rng;

    use crate::shaper::INDEX_MAX;

    use super::Shaper;

    #[test]
    fn test_floats() {
        assert_eq!(f32::MIN, (f32::MIN as f64) as f32);
        assert_ne!((f32::MIN / 1.24436) as f64, f32::MIN as f64 / 1.24436); // NOT EQUAL!
        assert_ne!((f32::MIN / 1.24436_f32) as f64, f32::MIN as f64 / 1.24436_f32 as f64); // NOT EQUAL!
    }

    #[test]
    fn print_default_lut() {
        let shaper = Shaper::default();
        for i in 0..shaper.lut.len() {
            println!("{:<4}:{}", i, shaper.lut[i])
        };
        assert_eq!(shaper.lut[0], f32::MIN);
        assert_eq!(shaper.lut[shaper.lut.len()-1], f32::MAX);
    }

    #[test]
    fn test_value_to_index() {
        let index = Shaper::nearest_lower_index(f32::MIN);
        println!("Testing if index is 0 with input f32::MIN:");
        println!("\tindex:          {}", index);
        println!("\texpected index: {}", 0);
        assert_eq!(index, 0);

        let index = Shaper::nearest_lower_index(f32::MAX);
        println!("Testing if index is max index with input f32::MAX:");
        println!("\tindex:          {}", index);
        println!("\texpected index: {}", INDEX_MAX);
        assert_eq!(index, INDEX_MAX);
    }

    #[test]
    fn test_interpolate() {
        let shaper = Shaper::default();
        let mut rng = rand::thread_rng();
        
        for i in 0..100 {
            let bytes: [u8; 4] = rng.gen();
            let x = f32::from_ne_bytes(bytes);
            let y = shaper.interpolate(Shaper::nearest_lower_index(x), x);
            println!("{:<2}: x={}, y={}", i, x, y);
            println!("inaccuracy: {}", x - y);
        }
        // let x = f32::MIN + (f32::MAX / 6.0);
        // let y = shaper.interpolate(Shaper::nearest_lower_index(x), x);
        // println!("x={}, y={}", x, y);
        // println!("inaccuracy: {}", x - y);
    }
}