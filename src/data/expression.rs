use crate::{
    data::{Gate, TerminalId, Token},
    parser::ParsingError,
};
use std::fmt;

#[derive(Clone, Debug, strum::Display, PartialEq)]
pub enum Literal {
    TerminalId(TerminalId),
    Gate(Gate),
}

impl TryFrom<Token> for Literal {
    type Error = ParsingError;

    fn try_from(token: Token) -> Result<Self, Self::Error> {
        let value = match token {
            Token::TerminalId(id) => Literal::TerminalId(id),
            Token::Gate(gate) => Literal::Gate(gate),
            _ => return Err(ParsingError::NoSuchExpression(token.to_string())),
        };

        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Literal(Literal),
    List(Vec<Expression>),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Expression::Literal(literal) => write!(f, "{literal}"),
            Expression::List(list) => write!(
                f,
                "({})",
                list.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        }
    }
}
