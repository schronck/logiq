use super::scanner::Scanner;
use super::{Token, TokenTree};
use crate::gate::Gate;
use thiserror::Error;

use std::iter::Peekable;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("the resulting expression has dangling terminals")]
    InvalidExpression,
    #[error("two consecutive terminals in expression")]
    InvalidTerminalPlacement,
    #[error("gates must come between terminals")]
    InvalidGatePlacement,
    #[error("{0}")]
    ScannerError(#[from] super::scanner::ScannerError),
    #[error(transparent)]
    Transparent(#[from] anyhow::Error),
}

pub struct Parser<'a> {
    source: Peekable<std::slice::Iter<'a, Token>>,
}

impl<'a, 'b> Parser<'a>
where
    'b: 'a,
{
    pub fn parse(source: &'b str) -> Result<TokenTree, ParserError> {
        let scanned = Scanner::scan(source)?;
        let mut parser = Self {
            source: scanned.iter().peekable(),
        };
        parser.parse_next()?.ok_or(ParserError::InvalidExpression)
    }

    fn parse_next(&mut self) -> Result<Option<TokenTree>, ParserError> {
        let mut current_tree: Option<TokenTree> = None;
        let mut current_gate: Option<Gate> = None;
        while let Some(token) = self.source.next() {
            current_tree = match token {
                Token::Whitespace => unreachable!("use with pre-scanned input"),
                Token::OpeningParenthesis => self.parse_next()?,
                Token::ClosingParenthesis => return Ok(current_tree),
                Token::Terminal(c) => match (current_tree, current_gate) {
                    (None, None) => Some(TokenTree::Terminal(*c)),
                    (None, Some(_)) => return Err(ParserError::InvalidGatePlacement),
                    (Some(_), None) => return Err(ParserError::InvalidTerminalPlacement),
                    (Some(tree), Some(gate)) => {
                        current_gate = None;
                        Some(TokenTree::Gate {
                            gate,
                            left: Box::new(tree),
                            right: Box::new(TokenTree::Terminal(*c)),
                        })
                    }
                },
                Token::Gate(gate) => match (current_tree.as_ref(), current_gate) {
                    (Some(_), None) => {
                        current_gate = Some(*gate);
                        current_tree
                    }
                    _ => return Err(ParserError::InvalidGatePlacement),
                },
            }
        }
        // is there a gate without a second terminal?
        if current_gate.is_some() {
            Err(ParserError::InvalidGatePlacement)
        } else {
            Ok(current_tree)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    fn is_equal_discriminant(this: &ScannerError, that: &ScannerError) -> bool {
        std::mem::discriminant(this) == std::mem::discriminant(that)
    }

    #[test]
    fn parse_single_variable() {
        let scanned = ScannedTokens::from_str("p").unwrap();
        let tree = Parser::parse(&scanned).unwrap();
        match tree {
            TokenTree::Terminal(c) => assert_eq!(c, 'p'),
            _ => panic!("should be terminal"),
        }

        let scanned = ScannedTokens::from_str("(p)").unwrap();
        let tree = Parser::parse(&scanned).unwrap();
        match tree {
            TokenTree::Terminal(c) => assert_eq!(c, 'p'),
            _ => panic!("should be terminal"),
        }

        let scanned = ScannedTokens::from_str("((((p))))").unwrap();
        let tree = Parser::parse(&scanned).unwrap();
        match tree {
            TokenTree::Terminal(c) => assert_eq!(c, 'p'),
            _ => panic!("should be terminal"),
        }
    }
}
/*

    #[test]
    fn parse_single_whitespace() {
        let scanned = ScannedTokens::from_str(" ").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::EmptyExpression
        ));
    }

    #[test]
    fn parse_invalid_parentheses() {
        let scanned = ScannedTokens::from_str("( )").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::EmptyExpression,
        ));

        let scanned = ScannedTokens::from_str("(()").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidParentheses,
        ));

        let scanned = ScannedTokens::from_str("    )").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidParentheses,
        ));

        let scanned = ScannedTokens::from_str("(())(").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidParentheses,
        ));

        let scanned = ScannedTokens::from_str("())))))))))))))").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidParentheses,
        ));
    }

    #[test]
    fn parse_invalid_terminals() {
        let scanned = ScannedTokens::from_str("a b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidTerminalPlacement
        ));

        let scanned = ScannedTokens::from_str("(a)").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidExpression
        ));

        let scanned = ScannedTokens::from_str("(a AND c) b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidTerminalPlacement
        ));

        let scanned = ScannedTokens::from_str("(a) b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidTerminalPlacement
        ));
    }

    #[test]
    fn parse_invalid_gates() {
        let scanned = ScannedTokens::from_str("a AND OR b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidGatePlacement
        ));

        let scanned = ScannedTokens::from_str("(a AND) OR b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidGatePlacement
        ));

        let scanned = ScannedTokens::from_str("(a AND OR) b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidGatePlacement
        ));

        let scanned = ScannedTokens::from_str("a AND (OR) b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidGatePlacement
        ));

        let scanned = ScannedTokens::from_str("a AND () b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidExpression
        ));

        let scanned = ScannedTokens::from_str("a AND ( ()) b").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidExpression
        ));
    }

    #[test]
    fn parse_valid_statement() {
        let scanned = ScannedTokens::from_str("a AND b OR c").unwrap();
        let parsed = parse(&scanned).unwrap();
        assert_eq!(
            parsed.tokens(),
            &[
                Token::Terminal('a'),
                Token::Gate(Gate::And),
                Token::Terminal('b'),
                Token::Gate(Gate::Or),
                Token::Terminal('c')
            ]
        );
        let scanned = ScannedTokens::from_str("a AND (b OR c) XOR d").unwrap();
        let parsed = parse(&scanned).unwrap();
        assert_eq!(
            parsed.tokens(),
            &[
                Token::Terminal('a'),
                Token::Gate(Gate::And),
                Token::OpeningParenthesis,
                Token::Terminal('b'),
                Token::Gate(Gate::Or),
                Token::Terminal('c'),
                Token::ClosingParenthesis,
                Token::Gate(Gate::Xor),
                Token::Terminal('d')
            ]
        );
    }
}
*/
