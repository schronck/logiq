use super::{ParsedTokens, ScannedTokens, Token};
use crate::gate::Gate;
use thiserror::Error;

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

pub struct Parser<'a, I: Iterator<Item = Token>> {
    source: Peekable<I>,
    current_tree: Option<TokenTree>,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_single_variable() {
        let scanned = ScannedTokens::from_str("p").unwrap();
        let parsed = parse(&scanned).unwrap();
        assert_eq!(parsed.tokens(), &[Token::Terminal('p')]);
        // NOTE the parser is not smart enough to accept these as
        // valid inputs
        let scanned = ScannedTokens::from_str("(p)").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidExpression,
        ));
        let scanned = ScannedTokens::from_str("((((p))))").unwrap();
        assert!(is_equal_discriminant(
            &parse(&scanned).err().unwrap(),
            &ParserError::InvalidExpression,
        ));
    }

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
