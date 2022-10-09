use super::scanner::{ScanError, Scanner};
use super::{Token, TokenTree};
use crate::gate::Gate;
use thiserror::Error;

use std::iter::Peekable;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("the resulting expression has dangling terminals")]
    InvalidExpression,
    #[error("two consecutive terminals in expression")]
    InvalidTerminalPlacement,
    #[error("gates must come between terminals")]
    InvalidGatePlacement,
    #[error("{0}")]
    ScanError(#[from] ScanError),
}

pub fn parse(source: &str) -> Result<TokenTree, ParseError> {
    let scanned = Scanner::scan(source)?;
    parse_next(&mut scanned.iter().peekable())?.ok_or(ParseError::InvalidExpression)
}

fn parse_next(
    scanned: &mut Peekable<std::slice::Iter<'_, Token>>,
) -> Result<Option<TokenTree>, ParseError> {
    let mut current_tree: Option<TokenTree> = None;
    let mut current_gate: Option<Gate> = None;
    while let Some(token) = scanned.next() {
        (current_tree, current_gate) = match token {
            Token::Whitespace => unreachable!("use with pre-scanned input"),
            Token::OpeningParenthesis => {
                if let Some(new_leaf) = parse_next(scanned)? {
                    try_finalize_leaf(current_tree, current_gate, new_leaf)?
                } else {
                    (current_tree, current_gate)
                }
            }
            Token::ClosingParenthesis => {
                if current_tree.is_some() && current_gate.is_some() {
                    return Err(ParseError::InvalidGatePlacement);
                } else {
                    return Ok(current_tree);
                }
            }
            Token::Terminal(c) => {
                try_finalize_leaf(current_tree, current_gate, TokenTree::Terminal(*c))?
            }
            Token::Gate(gate) => {
                if current_tree.is_some() && current_gate.is_none() {
                    (current_tree, Some(*gate))
                } else {
                    return Err(ParseError::InvalidGatePlacement);
                }
            }
        };
    }
    // error if there's an unfinalized leaf (gate)
    if current_gate.is_some() {
        Err(ParseError::InvalidGatePlacement)
    } else {
        Ok(current_tree)
    }
}

fn try_finalize_leaf(
    current_tree: Option<TokenTree>,
    current_gate: Option<Gate>,
    new_leaf: TokenTree,
) -> Result<(Option<TokenTree>, Option<Gate>), ParseError> {
    match (current_tree, current_gate) {
        (None, None) => Ok((Some(new_leaf), None)),
        (None, Some(_)) => Err(ParseError::InvalidGatePlacement),
        (Some(_), None) => Err(ParseError::InvalidTerminalPlacement),
        (Some(tree), Some(gate)) => Ok((
            Some(TokenTree::Gate {
                gate,
                left: Box::new(tree),
                right: Box::new(new_leaf),
            }),
            None,
        )),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn is_equal_discriminant(this: &ParseError, that: &ParseError) -> bool {
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
            ParseError::ScanError(ScanError::EmptyExpression) => {}
            _ => panic!("should be scanner error, empty expression"),
        }
    }

    #[test]
    fn parse_invalid_terminals() {
        assert!(is_equal_discriminant(
            &parse("a b").err().unwrap(),
            &ParseError::InvalidTerminalPlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(a AND c) b").err().unwrap(),
            &ParseError::InvalidTerminalPlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(a) b").err().unwrap(),
            &ParseError::InvalidTerminalPlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(((a)) (b))").err().unwrap(),
            &ParseError::InvalidTerminalPlacement
        ));
    }

    #[test]
    fn parse_invalid_gates() {
        assert!(is_equal_discriminant(
            &parse("a AND OR b").err().unwrap(),
            &ParseError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(a AND) OR b").err().unwrap(),
            &ParseError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(a AND OR ) b").err().unwrap(),
            &ParseError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            &parse("a AND (OR) b").err().unwrap(),
            &ParseError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(a NAND b OR )").err().unwrap(),
            &ParseError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            &parse("a NAND (b OR ) XOR c").err().unwrap(),
            &ParseError::InvalidGatePlacement
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
        // descending the tree is quite painful like this
        match parsed {
            TokenTree::Gate {
                gate: Gate::Xor,
                left: tree,
                right: terminal_d,
            } => {
                match *terminal_d {
                    TokenTree::Terminal('d') => {}
                    _ => unreachable!(),
                }
                match *tree {
                    TokenTree::Gate {
                        gate: Gate::And,
                        left: terminal_a,
                        right: tree,
                    } => {
                        match *terminal_a {
                            TokenTree::Terminal('a') => {}
                            _ => unreachable!(),
                        }
                        match *tree {
                            TokenTree::Gate {
                                gate: Gate::Or,
                                left: terminal_b,
                                right: terminal_c,
                            } => match (*terminal_b, *terminal_c) {
                                (TokenTree::Terminal('b'), TokenTree::Terminal('c')) => {}
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}
