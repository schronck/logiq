use crate::{
    data::{Gate, TerminalId, Token},
    parser::ParsingError,
};

#[derive(Clone, Debug, PartialEq)]
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
