use evalexpr::EvalexprError;

pub(crate) fn chebychev(value: &f64, order: &i64) -> Result<f64, EvalexprError> {
    match order {
        n if *n < 0 => {
            Err(EvalexprError::CustomMessage("Chebychev order can't be negative".to_owned()))
        }
        0 => Ok(1.0),
        1 => Ok(*value),
        n => Ok(2.0 * value * chebychev(value, &(n - 1))? - chebychev(value, &(n - 2))?),
    }
}
