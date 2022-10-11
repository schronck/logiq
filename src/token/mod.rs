mod parse;
mod scan;

use crate::gate::Gate;
use crate::TerminalId;
use parse::parse;
pub use parse::ParseError;

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
pub enum TokenTree {
    Terminal(TerminalId),
    Gate {
        gate: Gate,
        left: Box<TokenTree>,
        right: Box<TokenTree>,
    },
}

impl TokenTree {
    pub fn new(source: &str) -> Result<Self, ParseError> {
        parse(source)
    }
}

impl FromStr for TokenTree {
    type Err = ParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::new(input)
    }
}
