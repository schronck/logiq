use crate::{
    data::{Expression, Gate, List, Literal, Value},
    parser::*,
};

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("\"{0}\" is not a gate")]
    NotAGate(Value),
    #[error("\"{0}\" is not a terminal id")]
    NotATerminalId(Value),
    #[error("\"{0}\" is an unary operator")]
    UnaryOperator(Gate),
    #[error("\"{0}\" is a binary operator")]
    BinaryOperator(Gate),
    #[error("Invalid list: {0}")]
    InvalidList(Value),
    #[error(transparent)]
    ParsingError(#[from] ParsingError),
}

pub fn eval(logic_str: &str, thruths: &[bool]) -> Result<bool, EvalError> {
    let parsed = parse(logic_str)?;
    let value = eval_expression(&parsed, thruths)?;

    eval_logic(&value)
}

pub fn eval_expression(expression: &Expression, thruths: &[bool]) -> Result<Value, EvalError> {
    let value = match expression {
        Expression::Literal(literal) => match literal {
            Literal::TerminalId(id) => Value::Thruth(thruths[*id as usize]),
            Literal::Gate(gate) => Value::Gate(*gate),
        },
        Expression::List(list) => Value::List(List(
            list.iter()
                .map(|expr| eval_expression(expr, thruths))
                .collect::<Result<Vec<_>, _>>()?,
        )),
    };

    Ok(value)
}

pub fn eval_logic(value: &Value) -> Result<bool, EvalError> {
    let res = match value {
        Value::Thruth(thruth) => *thruth,
        Value::Gate(_) => return Err(EvalError::NotATerminalId(value.clone())),
        Value::List(List(list)) => match list.len() {
            0 => true,
            1 => eval_logic(&list[0])?,
            2 => {
                let Value::Gate(gate) = &list[0] else {
                    return Err(EvalError::NotAGate(list[0].clone()));
                };

                if *gate == Gate::Not {
                    let right = eval_logic(&list[1])?;

                    !right
                } else {
                    return Err(EvalError::BinaryOperator(*gate));
                }
            }
            3 => {
                let left = eval_logic(&list[0])?;
                let right = eval_logic(&list[2])?;

                let Value::Gate(gate) = &list[1] else {
                    return Err(EvalError::NotAGate(list[1].clone()));
                };

                match &gate {
                    Gate::And => left && right,
                    Gate::Or => left || right,
                    Gate::Nand => !(left && right),
                    Gate::Nor => !(left || right),
                    Gate::Xor => left ^ right,
                    _ => return Err(EvalError::UnaryOperator(*gate)),
                }
            }
            _ => return Err(EvalError::InvalidList(value.clone())),
        },
    };

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::eval;

    const THRUTHS: &[bool] = &[true, false, true, false, true, false];

    #[test]
    fn test_eval() {
        let res = eval("", THRUTHS).unwrap();
        assert!(res);

        let res = eval("()", THRUTHS).unwrap();
        assert!(res);

        let res = eval("0", THRUTHS).unwrap();
        assert!(res);

        let res = eval("1", THRUTHS).unwrap();
        assert!(!res);

        let res = eval("(not 0)", THRUTHS).unwrap();
        assert!(!res);

        let res = eval("(0 and 1)", THRUTHS).unwrap();
        assert!(!res);

        let res = eval("((0 and 1) or 2)", THRUTHS).unwrap();
        assert!(res);
    }
}
