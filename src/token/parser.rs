use super::scanner::{Scanner, ScannerError};
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
    ScannerError(#[from] ScannerError),
    #[error(transparent)]
    Transparent(#[from] anyhow::Error),
}

pub fn parse(source: &str) -> Result<TokenTree, ParserError> {
    let scanned = Scanner::scan(source)?;
    parse_next(&mut scanned.iter().peekable())?.ok_or(ParserError::InvalidExpression)
}

fn parse_next(
    scanned: &mut Peekable<std::slice::Iter<'_, Token>>,
) -> Result<Option<TokenTree>, ParserError> {
    let mut current_tree: Option<TokenTree> = None;
    let mut current_gate: Option<Gate> = None;
    while let Some(token) = scanned.next() {
        current_tree = match token {
            Token::Whitespace => unreachable!("use with pre-scanned input"),
            Token::OpeningParenthesis => {
                if let Some(new_tree) = parse_next(scanned)? {
                    match (current_tree, current_gate) {
                        (None, None) => Some(new_tree),
                        (None, Some(_)) => return Err(ParserError::InvalidGatePlacement),
                        (Some(_), None) => return Err(ParserError::InvalidTerminalPlacement),
                        (Some(tree), Some(gate)) => {
                            current_gate = None;
                            Some(TokenTree::Gate {
                                gate,
                                left: Box::new(tree),
                                right: Box::new(new_tree),
                            })
                        }
                    }
                } else {
                    current_tree
                }
            }
            Token::ClosingParenthesis => {
                if current_tree.is_some() && current_gate.is_some() {
                    return Err(ParserError::InvalidGatePlacement);
                } else {
                    return Ok(current_tree);
                }
            }
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
            Token::Gate(gate) => {
                if current_tree.is_some() && current_gate.is_none() {
                    current_gate = Some(*gate);
                    current_tree
                } else {
                    return Err(ParserError::InvalidGatePlacement);
                }
            }
        };
    }
    // is there a gate without a second terminal?
    if current_gate.is_some() {
        Err(ParserError::InvalidGatePlacement)
    } else {
        Ok(current_tree)
    }
}

//fn finalize_leaf(current_tree: Option<TokenTree>, current_gate: Option<Gate>, new_leaf: TokenTree) {
//}

#[cfg(test)]
mod test {
    use super::*;

    fn is_equal_discriminant(this: &ParserError, that: &ParserError) -> bool {
        std::mem::discriminant(this) == std::mem::discriminant(that)
    }

    #[test]
    fn parse_single_variable() {
        let tree = parse("p").unwrap();
        match tree {
            TokenTree::Terminal(c) => assert_eq!(c, 'p'),
            _ => panic!("should be terminal"),
        }

        let tree = parse("(p)").unwrap();
        match tree {
            TokenTree::Terminal(c) => assert_eq!(c, 'p'),
            _ => panic!("should be terminal"),
        }

        let tree = parse("((((p))))").unwrap();
        match tree {
            TokenTree::Terminal(c) => assert_eq!(c, 'p'),
            _ => panic!("should be terminal"),
        }
    }

    #[test]
    fn parse_single_whitespace() {
        match parse(" ").err().unwrap() {
            ParserError::ScannerError(ScannerError::EmptyExpression) => {}
            _ => panic!("should be scanner error, empty expression"),
        }
    }

    #[test]
    fn parse_invalid_terminals() {
        assert!(is_equal_discriminant(
            &parse("a b").err().unwrap(),
            &ParserError::InvalidTerminalPlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(a AND c) b").err().unwrap(),
            &ParserError::InvalidTerminalPlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(a) b").err().unwrap(),
            &ParserError::InvalidTerminalPlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(((a)) (b))").err().unwrap(),
            &ParserError::InvalidTerminalPlacement
        ));
    }

    #[test]
    fn parse_invalid_gates() {
        assert!(is_equal_discriminant(
            &parse("a AND OR b").err().unwrap(),
            &ParserError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(a AND) OR b").err().unwrap(),
            &ParserError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(a AND OR ) b").err().unwrap(),
            &ParserError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            dbg!(&parse("a AND (OR) b").err().unwrap()),
            &ParserError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(a NAND b OR )").err().unwrap(),
            &ParserError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            &parse("a NAND (b OR ) XOR c").err().unwrap(),
            &ParserError::InvalidGatePlacement
        ));
    }

    #[test]
    fn parse_valid_statement() {
        // empty parentheses are "discarded"
        assert!(parse("a XOR () b").is_ok());
        assert!(parse("a NAND ( ()) b").is_ok());

        let parsed = parse("a AND b OR c").unwrap();
        match parsed {
            TokenTree::Gate {
                gate: Gate::Or,
                left: tree,
                right: terminal_c,
            } => {
                match *terminal_c {
                    TokenTree::Terminal('c') => {}
                    _ => unreachable!(),
                }
                match *tree {
                    TokenTree::Gate {
                        gate: Gate::And,
                        left: terminal_a,
                        right: terminal_b,
                    } => match (*terminal_a, *terminal_b) {
                        (TokenTree::Terminal('a'), TokenTree::Terminal('b')) => {}
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
        let parsed = parse("a AND (b OR c) XOR d").unwrap();
        // TODO
    }
}
