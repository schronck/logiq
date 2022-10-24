mod parse;
mod scan;

use crate::gate::Gate;
use crate::TerminalId;
use parse::parse;
pub use parse::ParseError;

use std::collections::HashMap;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Token {
    Whitespace,
    OpeningParenthesis,
    ClosingParenthesis,
    Terminal(TerminalId),
    Gate(Gate),
}

#[derive(Clone, Debug)]
pub enum LogicTree {
    Terminal(TerminalId),
    Gate {
        gate: Gate,
        left: Box<LogicTree>,
        right: Box<LogicTree>,
    },
}

impl LogicTree {
    pub fn new(source: &str) -> Result<Self, ParseError> {
        parse(source)
    }

    pub fn evaluate(&self, terminals: &HashMap<TerminalId, bool>) -> Result<bool, String> {
        let eval;
        match self {
            LogicTree::Terminal(c) => {
                eval = *terminals
                    .get(c)
                    .ok_or_else(|| "Invalid terminals map".to_string())?
            }
            LogicTree::Gate { gate, left, right } => {
                let left_eval = left.evaluate(terminals)?;
                let right_eval = right.evaluate(terminals)?;
                match gate {
                    Gate::And => eval = left_eval && right_eval,
                    Gate::Or => eval = left_eval || right_eval,
                    Gate::Nand => eval = !(left_eval && right_eval),
                    Gate::Nor => eval = !(left_eval || right_eval),
                    Gate::Xor => eval = left_eval ^ right_eval,
                }
            }
        }
        Ok(eval)
    }
}

impl FromStr for LogicTree {
    type Err = ParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::new(input)
    }
}

#[test]
fn evaluate_tree() {
    let tree = LogicTree::from_str("0 AND 1 OR ((0 NAND 2) OR 3)").unwrap();
    let mut terminals = HashMap::new();
    terminals.insert(0, true);
    terminals.insert(1, true);
    terminals.insert(2, true);
    terminals.insert(3, true);
    assert!(tree.evaluate(&terminals).unwrap());
    let terminal = terminals.get_mut(&1).unwrap();
    *terminal = false;
    assert!(tree.evaluate(&terminals).unwrap());
    let terminal = terminals.get_mut(&3).unwrap();
    *terminal = false;
    assert!(!tree.evaluate(&terminals).unwrap());
    let terminal = terminals.get_mut(&2).unwrap();
    *terminal = false;
    assert!(tree.evaluate(&terminals).unwrap());
    let terminal = terminals.get_mut(&0).unwrap();
    *terminal = false;
    assert!(tree.evaluate(&terminals).unwrap());
}
