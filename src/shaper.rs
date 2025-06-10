use anita::{
    compile_expression,
    jit::{compiled_function::CompiledFunction, JITError},
};

use crate::math::function_manager::MathFunctions;

pub(crate) type Shaper = CompiledFunction<fn(f32, f32, f32, f32, f32, f32) -> f32>;

pub(crate) fn compile_shaper<E: AsRef<str>>(expr: E) -> Result<Shaper, JITError> {
    compile_expression!(expr, (x, s, a, b, c, d) -> f32, MathFunctions)
}
