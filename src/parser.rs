use crate::{lexer::*, token::*};

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

#[derive(Debug, thiserror::Error)]
pub enum ParsingError {
    #[error("PARSER: cannot create expression from token {0}")]
    NoSuchExpression(String),
    #[error("PARSER: expected \"ParenOpen\" or \"Quote\", found {0}")]
    InvalidStart(String),
    #[error("PARSER: missing expression after quote")]
    OutOfTokens,
    #[error(transparent)]
    LexingError(#[from] LexingError),
}

pub fn parse(input: &str) -> Result<Expression, ParsingError> {
    parse_list(&mut tokenize(input)?.into_iter().rev().collect::<Vec<_>>())
}

pub fn parse_list(tokens: &mut Vec<Token>) -> Result<Expression, ParsingError> {
    let Some(token) = tokens.pop() else {
        return Ok(Expression::List(vec![]));
    };

    if tokens.is_empty() {
        return match token {
            Token::ParenOpen | Token::ParenClose => {
                Err(ParsingError::NoSuchExpression(token.to_string()))
            }
            Token::Gate(gate) => Err(ParsingError::NoSuchExpression(gate.to_string())),
            _ => Ok(Expression::Literal(Literal::try_from(token)?)),
        };
    }

    if ![Token::ParenOpen].contains(&token) {
        return Err(ParsingError::InvalidStart(token.to_string()));
    }

    let mut list: Vec<Expression> = Vec::new();

    while !tokens.is_empty() {
        let Some(token) = tokens.pop() else {
            return Err(ParsingError::OutOfTokens);
        };

        match &token {
            Token::ParenOpen => {
                tokens.push(Token::ParenOpen);

                let sub_list = parse_list(tokens)?;

                list.push(sub_list);
            }
            Token::ParenClose => return Ok(Expression::List(list)),
            _ => list.push(Expression::Literal(Literal::try_from(token)?)),
        }
    }

    Ok(Expression::List(list))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let list = parse("").unwrap();
        assert_eq!(list, Expression::List(vec![]));

        let list = parse("()").unwrap();
        assert_eq!(list, Expression::List(vec![]));
    }

    #[test]
    fn test_terminal_id() {
        let list = parse("0").unwrap();
        assert_eq!(list, Expression::Literal(Literal::TerminalId(0)));

        let list = parse("69").unwrap();
        assert_eq!(list, Expression::Literal(Literal::TerminalId(69)));
    }

    #[test]
    #[should_panic]
    fn test_gate() {
        parse("and").unwrap();
    }

    #[test]
    fn test_logic_string() {
        let list = parse("(0 and 1)").unwrap();
        assert_eq!(
            list,
            Expression::List(vec![
                Expression::Literal(Literal::TerminalId(0)),
                Expression::Literal(Literal::Gate(Gate::And)),
                Expression::Literal(Literal::TerminalId(1)),
            ])
        );

        let list = parse("((0 and 1) or 2)").unwrap();
        assert_eq!(
            list,
            Expression::List(vec![
                Expression::List(vec![
                    Expression::Literal(Literal::TerminalId(0)),
                    Expression::Literal(Literal::Gate(Gate::And)),
                    Expression::Literal(Literal::TerminalId(1))
                ]),
                Expression::Literal(Literal::Gate(Gate::Or)),
                Expression::Literal(Literal::TerminalId(2)),
            ])
        );
    }
}