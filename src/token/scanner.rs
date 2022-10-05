use super::{ScannedTokens, Token};
use crate::gate::Gate;
use thiserror::Error;

use std::iter::Peekable;
use std::str::{Chars, FromStr};

#[derive(Error, Debug)]
pub enum ScannerError {
    #[error("input fully scanned")]
    EofReached,
    #[error("invalid token: {0}")]
    InvalidToken(char),
    #[error(transparent)]
    Transparent(#[from] anyhow::Error),
}

pub struct Scanner<'a> {
    source: Peekable<Chars<'a>>,
    lexeme: String,
    tokens: Vec<Token>,
}

impl<'a, 'b> Scanner<'a>
where
    'b: 'a,
{
    pub fn scan(source: &'b str) -> Result<ScannedTokens, ScannerError> {
        let mut scanner = Self {
            source: source.chars().peekable(),
            lexeme: String::new(),
            tokens: Vec::new(),
        };

        loop {
            match scanner.scan_next() {
                Ok(token) => scanner.tokens.push(token),
                Err(ScannerError::EofReached) => break,
                Err(e) => return Err(e),
            }
        }

        Ok(ScannedTokens(scanner.tokens))
    }

    fn scan_next(&mut self) -> Result<Token, ScannerError> {
        self.lexeme.clear();
        let next_char = self.advance().ok_or(ScannerError::EofReached)?;

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
            _ => Err(ScannerError::InvalidToken(next_char)),
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

    #[test]
    fn scan_empty() {
        let tokens = Scanner::scan("").unwrap();
        assert!(tokens.tokens().is_empty());
    }

    #[test]
    fn scan_whitespace() {
        let tokens = Scanner::scan(" ").unwrap();
        assert_eq!(tokens.tokens(), &[Token::Whitespace]);
    }

    #[test]
    fn scan_parentheses() {
        let tokens = Scanner::scan("( )").unwrap();
        assert_eq!(
            tokens.tokens(),
            &[
                Token::OpeningParenthesis,
                Token::Whitespace,
                Token::ClosingParenthesis,
            ]
        );
    }

    #[test]
    fn scan_variable() {
        let tokens = Scanner::scan("a").unwrap();
        assert_eq!(tokens.tokens(), &[Token::Terminal('a')]);
        let tokens = Scanner::scan("b c").unwrap();
        assert_eq!(
            tokens.tokens(),
            &[
                Token::Terminal('b'),
                Token::Whitespace,
                Token::Terminal('c'),
            ]
        );
        let tokens = Scanner::scan("foo bar").unwrap();
        assert_eq!(
            tokens.tokens(),
            &[
                Token::Terminal('f'),
                Token::Terminal('o'),
                Token::Terminal('o'),
                Token::Whitespace,
                Token::Terminal('b'),
                Token::Terminal('a'),
                Token::Terminal('r'),
            ]
        );
    }

    #[test]
    fn scan_gate() {
        let tokens = Scanner::scan("AND").unwrap();
        assert_eq!(tokens.tokens(), &[Token::Gate(Gate::And)]);
        let tokens = Scanner::scan("OR NAND XOR NOR").unwrap();
        assert_eq!(
            tokens.tokens(),
            &[
                Token::Gate(Gate::Or),
                Token::Whitespace,
                Token::Gate(Gate::Nand),
                Token::Whitespace,
                Token::Gate(Gate::Xor),
                Token::Whitespace,
                Token::Gate(Gate::Nor),
            ]
        );

        let tokens = Scanner::scan("AND(").unwrap();
        assert_eq!(
            tokens.tokens(),
            &[Token::Gate(Gate::And), Token::OpeningParenthesis]
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
        let tokens = Scanner::scan("aANDb)NAND XOR a b OR c)").unwrap();
        assert_eq!(
            tokens.tokens(),
            &[
                Token::Terminal('a'),
                Token::Gate(Gate::And),
                Token::Terminal('b'),
                Token::ClosingParenthesis,
                Token::Gate(Gate::Nand),
                Token::Whitespace,
                Token::Gate(Gate::Xor),
                Token::Whitespace,
                Token::Terminal('a'),
                Token::Whitespace,
                Token::Terminal('b'),
                Token::Whitespace,
                Token::Gate(Gate::Or),
                Token::Whitespace,
                Token::Terminal('c'),
                Token::ClosingParenthesis,
            ]
        );
    }
}
