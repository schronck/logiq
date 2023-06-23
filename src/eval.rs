use crate::{
    data::{Expression, Gate, Literal, TerminalId},
    parser::*,
};

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("\"{0}\" is not a gate")]
    NotAGate(Expression),
    #[error("\"{0}\" is not a terminal id")]
    NotATerminalId(Expression),
    #[error("Thruth index out of bounds: {0}")]
    OutOfBounds(TerminalId),
    #[error("\"{0}\" is an unary operator")]
    UnaryOperator(Gate),
    #[error("\"{0}\" is a binary operator")]
    BinaryOperator(Gate),
    #[error("Invalid list: {0}")]
    InvalidList(Expression),
    #[error(transparent)]
    ParsingError(#[from] ParsingError),
}

pub fn eval(logic_str: &str, thruths: &[bool]) -> Result<bool, EvalError> {
    eval_expression(&parse(logic_str)?, thruths)
}

pub fn eval_expression(expression: &Expression, thruths: &[bool]) -> Result<bool, EvalError> {
    let res = match expression {
        Expression::Literal(literal) => match literal {
            Literal::TerminalId(id) => thruths
                .get(*id as usize)
                .cloned()
                .ok_or(EvalError::OutOfBounds(*id))?,
            Literal::Gate(_) => return Err(EvalError::NotATerminalId(expression.clone())),
        },
        Expression::List(list) => match list.len() {
            0 => true,
            1 => eval_expression(&list[0], thruths)?,
            2 => {
                let Expression::Literal(Literal::Gate(gate)) = &list[0] else {
                    return Err(EvalError::NotAGate(list[0].clone()));
                };

                if *gate == Gate::Not {
                    let right = eval_expression(&list[1], thruths)?;

                    !right
                } else {
                    return Err(EvalError::BinaryOperator(*gate));
                }
            }
            3 => {
                let left = eval_expression(&list[0], thruths)?;
                let right = eval_expression(&list[2], thruths)?;

                let Expression::Literal(Literal::Gate(gate)) = &list[1] else {
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
            _ => return Err(EvalError::InvalidList(expression.clone())),
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
