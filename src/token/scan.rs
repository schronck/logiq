use super::Token;
use crate::gate::Gate;
use thiserror::Error;

use std::iter::Peekable;
use std::str::{Chars, FromStr};

#[derive(Error, Debug)]
pub enum ScanError {
    #[error("parsed an empty expression")]
    EmptyExpression,
    #[error("input fully scanned")]
    EofReached,
    #[error("missing on mangled parentheses in expression")]
    InvalidParentheses,
    #[error("invalid token: {0}")]
    InvalidToken(char),
    #[error("{0}")]
    Transparent(#[from] anyhow::Error),
}

pub struct Scanner<'a> {
    source: Peekable<Chars<'a>>,
    lexeme: String,
}

impl<'a, 'b> Scanner<'a>
where
    'b: 'a,
{
    pub fn scan(source: &'b str) -> Result<Vec<Token>, ScanError> {
        let mut scanner = Self {
            source: source.chars().peekable(),
            lexeme: String::new(),
        };

        let mut tokens = Vec::new();
        let mut parentheses = 0_u32; // for detecting invalid parentheses early

        loop {
            match scanner.scan_next() {
                Ok(token) => {
                    match token {
                        Token::Whitespace => continue,
                        Token::OpeningParenthesis => parentheses = parentheses.saturating_add(1),
                        Token::ClosingParenthesis => parentheses = parentheses.wrapping_sub(1),
                        _ => {}
                    }
                    tokens.push(token);
                }
                Err(ScanError::EofReached) => break,
                Err(e) => return Err(e),
            }
        }

        if tokens.is_empty() {
            Err(ScanError::EmptyExpression)
        } else if parentheses != 0 {
            Err(ScanError::InvalidParentheses)
        } else {
            Ok(tokens)
        }
    }

    fn scan_next(&mut self) -> Result<Token, ScanError> {
        self.lexeme.clear();
        let next_char = self.advance().ok_or(ScanError::EofReached)?;

        match next_char {
            c if matches!(c, ' ') => Ok(Token::Whitespace),
            '(' => Ok(Token::OpeningParenthesis),
            ')' => Ok(Token::ClosingParenthesis),
            c if c.is_ascii_lowercase() => Ok(Token::Terminal(c)),
            c if c.is_ascii_uppercase() => {
                self.advance_while(|x| x.is_ascii_uppercase());
                let boolean_gate = Gate::from_str(&self.lexeme)?;
                Ok(Token::Gate(boolean_gate))
            }
            _ => Err(ScanError::InvalidToken(next_char)),
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.source.next().map(|c| {
            self.lexeme.push(c);
            c
        })
    }

    fn peek_check<F>(&mut self, condition: F) -> bool
    where
        F: Fn(&char) -> bool,
    {
        if let Some(c) = self.source.peek() {
            condition(c)
        } else {
            false
        }
    }

    fn advance_while<F>(&mut self, condition: F)
    where
        F: Fn(&char) -> bool,
    {
        while self.peek_check(&condition) {
            self.advance();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn is_equal_discriminant(this: &ScanError, that: &ScanError) -> bool {
        std::mem::discriminant(this) == std::mem::discriminant(that)
    }

    #[test]
    fn scan_empty() {
        assert!(is_equal_discriminant(
            &Scanner::scan("").err().unwrap(),
            &ScanError::EmptyExpression
        ));
    }

    #[test]
    fn scan_whitespace() {
        assert!(is_equal_discriminant(
            &Scanner::scan(" ").err().unwrap(),
            &ScanError::EmptyExpression
        ));
    }

    #[test]
    fn scan_parentheses() {
        assert_eq!(
            &Scanner::scan("( )").unwrap(),
            &[Token::OpeningParenthesis, Token::ClosingParenthesis]
        );

        let error = Scanner::scan("(()").err().unwrap();
        assert!(is_equal_discriminant(
            &error,
            &ScanError::InvalidParentheses,
        ));

        let error = Scanner::scan("    )").err().unwrap();
        assert!(is_equal_discriminant(
            &error,
            &ScanError::InvalidParentheses,
        ));

        let error = Scanner::scan("(())(").err().unwrap();
        assert!(is_equal_discriminant(
            &error,
            &ScanError::InvalidParentheses,
        ));

        let error = Scanner::scan("())))))))))))))").err().unwrap();
        assert!(is_equal_discriminant(
            &error,
            &ScanError::InvalidParentheses,
        ));
    }

    #[test]
    fn scan_variable() {
        assert_eq!(&Scanner::scan("a").unwrap(), &[Token::Terminal('a')]);
        assert_eq!(
            &Scanner::scan("b c").unwrap(),
            &[Token::Terminal('b'), Token::Terminal('c'),]
        );
        assert_eq!(
            &Scanner::scan("foo bar").unwrap(),
            &[
                Token::Terminal('f'),
                Token::Terminal('o'),
                Token::Terminal('o'),
                Token::Terminal('b'),
                Token::Terminal('a'),
                Token::Terminal('r'),
            ]
        );
    }

    #[test]
    fn scan_gate() {
        assert_eq!(&Scanner::scan("AND").unwrap(), &[Token::Gate(Gate::And)]);
        assert_eq!(
            &Scanner::scan("OR NAND XOR NOR").unwrap(),
            &[
                Token::Gate(Gate::Or),
                Token::Gate(Gate::Nand),
                Token::Gate(Gate::Xor),
                Token::Gate(Gate::Nor),
            ]
        );

        assert_eq!(
            &Scanner::scan("(AND)").unwrap(),
            &[
                Token::OpeningParenthesis,
                Token::Gate(Gate::And),
                Token::ClosingParenthesis
            ]
        );
    }

    #[test]
    fn scan_invalid_gate() {
        assert!(Scanner::scan("A").is_err());
        assert!(Scanner::scan("ANDOR").is_err());
        assert!(Scanner::scan("NANd").is_err());
        assert!(Scanner::scan("XXR").is_err());
        assert!(Scanner::scan("Or").is_err());
    }

    #[test]
    fn scan_expression() {
        assert_eq!(
            &Scanner::scan("((aANDb)NAND XOR a b OR c)").unwrap(),
            &[
                Token::OpeningParenthesis,
                Token::OpeningParenthesis,
                Token::Terminal('a'),
                Token::Gate(Gate::And),
                Token::Terminal('b'),
                Token::ClosingParenthesis,
                Token::Gate(Gate::Nand),
                Token::Gate(Gate::Xor),
                Token::Terminal('a'),
                Token::Terminal('b'),
                Token::Gate(Gate::Or),
                Token::Terminal('c'),
                Token::ClosingParenthesis,
            ]
        );
    }
}
