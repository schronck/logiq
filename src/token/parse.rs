use super::scan::{ScanError, Scanner};
use super::{LogicTree, Token};
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

pub fn parse(source: &str) -> Result<LogicTree, ParseError> {
    let scanned = Scanner::scan(source)?;
    parse_next(&mut scanned.iter().peekable())?.ok_or(ParseError::InvalidExpression)
}

fn parse_next(
    scanned: &mut Peekable<std::slice::Iter<'_, Token>>,
) -> Result<Option<LogicTree>, ParseError> {
    let mut current_tree: Option<LogicTree> = None;
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
                let result = if current_tree.is_some() && current_gate.is_some() {
                    Err(ParseError::InvalidGatePlacement)
                } else {
                    Ok(current_tree)
                };
                return result;
            }
            Token::Terminal(c) => {
                try_finalize_leaf(current_tree, current_gate, LogicTree::Terminal(*c))?
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
    current_tree: Option<LogicTree>,
    current_gate: Option<Gate>,
    new_leaf: LogicTree,
) -> Result<(Option<LogicTree>, Option<Gate>), ParseError> {
    match (current_tree, current_gate) {
        (None, None) => Ok((Some(new_leaf), None)),
        (None, Some(_)) => Err(ParseError::InvalidGatePlacement),
        (Some(_), None) => Err(ParseError::InvalidTerminalPlacement),
        (Some(tree), Some(gate)) => Ok((
            Some(LogicTree::Gate {
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
        let tree = parse("0").unwrap();
        match tree {
            LogicTree::Terminal(c) => assert_eq!(c, 0),
            _ => panic!("should be terminal"),
        }

        let tree = parse("(11)").unwrap();
        match tree {
            LogicTree::Terminal(c) => assert_eq!(c, 11),
            _ => panic!("should be terminal"),
        }

        let tree = parse("((((111))))").unwrap();
        match tree {
            LogicTree::Terminal(c) => assert_eq!(c, 111),
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
            &parse("0 1").err().unwrap(),
            &ParseError::InvalidTerminalPlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(1 AND 2) 3").err().unwrap(),
            &ParseError::InvalidTerminalPlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(4) 5").err().unwrap(),
            &ParseError::InvalidTerminalPlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(((6)) (7))").err().unwrap(),
            &ParseError::InvalidTerminalPlacement
        ));
    }

    #[test]
    fn parse_invalid_gates() {
        assert!(is_equal_discriminant(
            &parse("55 AND OR 44").err().unwrap(),
            &ParseError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(10 AND) OR 99").err().unwrap(),
            &ParseError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(1000 AND OR ) 12").err().unwrap(),
            &ParseError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            &parse("0 AND (OR) 1").err().unwrap(),
            &ParseError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            &parse("(1 NAND 4 OR )").err().unwrap(),
            &ParseError::InvalidGatePlacement
        ));

        assert!(is_equal_discriminant(
            &parse("0 NAND (1 OR ) XOR 2").err().unwrap(),
            &ParseError::InvalidGatePlacement
        ));
    }

    #[test]
    fn parse_valid_statement() {
        // empty parentheses are "discarded"
        assert!(parse("1 XOR () 2").is_ok());
        assert!(parse("1 NAND ( ()) 2").is_ok());

        let parsed = parse("119").unwrap();
        match parsed {
            LogicTree::Terminal(119) => {}
            _ => unreachable!(),
        }

        let parsed = parse("((15) NOR ((16)))").unwrap();
        match parsed {
            LogicTree::Gate {
                gate: Gate::Nor,
                left: terminal_15,
                right: terminal_16,
            } => match (*terminal_15, *terminal_16) {
                (LogicTree::Terminal(15), LogicTree::Terminal(16)) => {}
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }

        let parsed = parse("0 AND 1 OR 2").unwrap();
        match parsed {
            LogicTree::Gate {
                gate: Gate::Or,
                left: tree,
                right: terminal_2,
            } => {
                match *terminal_2 {
                    LogicTree::Terminal(2) => {}
                    _ => unreachable!(),
                }
                match *tree {
                    LogicTree::Gate {
                        gate: Gate::And,
                        left: terminal_0,
                        right: terminal_1,
                    } => match (*terminal_0, *terminal_1) {
                        (LogicTree::Terminal(0), LogicTree::Terminal(1)) => {}
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
        let parsed = parse("0 AND (10 OR 11) XOR 0").unwrap();
        // descending the tree is quite painful like this
        match parsed {
            LogicTree::Gate {
                gate: Gate::Xor,
                left: tree,
                right: terminal_0,
            } => {
                match *terminal_0 {
                    LogicTree::Terminal(0) => {}
                    _ => unreachable!(),
                }
                match *tree {
                    LogicTree::Gate {
                        gate: Gate::And,
                        left: terminal_0,
                        right: tree,
                    } => {
                        match *terminal_0 {
                            LogicTree::Terminal(0) => {}
                            _ => unreachable!(),
                        }
                        match *tree {
                            LogicTree::Gate {
                                gate: Gate::Or,
                                left: terminal_10,
                                right: terminal_11,
                            } => match (*terminal_10, *terminal_11) {
                                (LogicTree::Terminal(10), LogicTree::Terminal(11)) => {}
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
