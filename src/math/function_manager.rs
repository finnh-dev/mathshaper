use anita::function_manager;

pub(crate) struct MathFunctions;

#[function_manager]
impl MathFunctions {
    #[name = "pmap"]
    fn param_range(param: f32, min: f32, max: f32) -> f32 {
        let range = max - min;
        param * range + min
    }

    #[name = "min"]
    fn math_min(x: f32, y: f32) -> f32 {
        x.min(y)
    }

    #[name = "max"]
    fn math_max(x: f32, y: f32) -> f32 {
        x.max(y)
    }

    #[name = "floor"]
    fn math_floor(x: f32) -> f32 {
        x.floor()
    }

    #[name = "round"]
    fn math_round(x: f32) -> f32 {
        x.round()
    }

    #[name = "ceil"]
    fn math_ceil(x: f32) -> f32 {
        x.ceil()
    }

    #[name = "is_nan"]
    fn math_is_nan(x: f32) -> f32 {
        x.is_nan() as u8 as f32
    }

    #[name = "is_finite"]
    fn math_is_finite(x: f32) -> f32 {
        x.is_finite() as u8 as f32
    }

    #[name = "is_infinite"]
    fn math_is_infinite(x: f32) -> f32 {
        x.is_infinite() as u8 as f32
    }

    #[name = "is_normal"]
    fn math_is_normal(x: f32) -> f32 {
        x.is_normal() as u8 as f32
    }

    #[name = "pow"]
    fn math_pow(a: f32, x: f32) -> f32 {
        a.powf(x)
    }

    #[name = "mod"]
    fn math_mod(x: f32, y: f32) -> f32 {
        x % y
    }

    #[name = "ln"]
    fn math_ln(x: f32) -> f32 {
        x.ln()
    }

    #[name = "log2"]
    fn math_log2(x: f32) -> f32 {
        x.log2()
    }

    #[name = "log10"]
    fn math_log10(x: f32) -> f32 {
        x.log10()
    }

    #[name = "exp"]
    fn math_exp(x: f32) -> f32 {
        x.exp()
    }

    #[name = "exp2"]
    fn math_exp2(x: f32) -> f32 {
        x.exp2()
    }

    #[name = "cos"]
    fn math_cos(x: f32) -> f32 {
        x.cos()
    }

    #[name = "acos"]
    fn math_acos(x: f32) -> f32 {
        x.acos()
    }

    #[name = "cosh"]
    fn math_cosh(x: f32) -> f32 {
        x.cosh()
    }

    #[name = "acosh"]
    fn math_acosh(x: f32) -> f32 {
        x.acosh()
    }

    #[name = "sin"]
    fn math_sin(x: f32) -> f32 {
        x.sin()
    }

    #[name = "asin"]
    fn math_asin(x: f32) -> f32 {
        x.asin()
    }

    #[name = "sinh"]
    fn math_sinh(x: f32) -> f32 {
        x.sinh()
    }

    #[name = "asinh"]
    fn math_asinh(x: f32) -> f32 {
        x.asinh()
    }

    #[name = "tan"]
    fn math_tan(x: f32) -> f32 {
        x.tan()
    }

    #[name = "atan"]
    fn math_atan(x: f32) -> f32 {
        x.atan()
    }

    #[name = "atan2"]
    fn math_atan2(x: f32, y: f32) -> f32 {
        x.atan2(y)
    }

    #[name = "tanh"]
    fn math_tanh(x: f32) -> f32 {
        match x {
            f32::INFINITY => 1.0,
            f32::NEG_INFINITY => -1.0,
            x if x.is_nan() => 0.0,
            x => x.tanh(),
        }
    }

    #[name = "atanh"]
    fn math_atanh(x: f32) -> f32 {
        x.atanh()
    }

    #[name = "sqrt"]
    fn math_sqrt(x: f32) -> f32 {
        x.sqrt()
    }

    #[name = "cbrt"]
    fn math_cbrt(x: f32) -> f32 {
        x.cbrt()
    }

    #[name = "abs"]
    fn math_abs(x: f32) -> f32 {
        x.abs()
    }

    #[name = "hypot"]
    fn math_hypot(x: f32, y: f32) -> f32 {
        x.hypot(y)
    }

    #[name = "if"]
    fn math_if(cond: f32, a: f32, b: f32) -> f32 {
        if cond.is_normal() && cond != 0.0 {
            a
        } else {
            b
        }
    }
}

#[cfg(test)]
mod test {
    use super::MathFunctions;

    #[test]
    fn test_param_range() {
        // Test unipolar
        assert_eq!(MathFunctions::param_range(0.0, 10.0, 20.0), 10.0);
        assert_eq!(MathFunctions::param_range(0.5, 10.0, 20.0), 15.0);
        assert_eq!(MathFunctions::param_range(1.0, 10.0, 20.0), 20.0);

        // Test unipolar inverted
        assert_eq!(MathFunctions::param_range(0.0, 20.0, 10.0), 20.0);
        assert_eq!(MathFunctions::param_range(0.5, 20.0, 10.0), 15.0);
        assert_eq!(MathFunctions::param_range(1.0, 20.0, 10.0), 10.0);

        // Test bipolar
        assert_eq!(MathFunctions::param_range(0.0, -1.0, 1.0), -1.0);
        assert_eq!(MathFunctions::param_range(0.5, -1.0, 1.0), 0.0);
        assert_eq!(MathFunctions::param_range(1.0, -1.0, 1.0), 1.0);
    }
}
