mod parser;
mod scanner;

use crate::gate::Gate;
use parser::{parse, ParserError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Token {
    Whitespace,
    OpeningParenthesis,
    ClosingParenthesis,
    Terminal(char),
    Gate(Gate),
}

#[derive(Clone, Debug)]
pub enum TokenTree {
    Terminal(char),
    Gate {
        gate: Gate,
        left: Box<TokenTree>,
        right: Box<TokenTree>,
    },
}

/*
impl TokenTree {
    pub fn new(source: &str) -> Result<Self, Error> {
        parse(source)
    }
}

impl FromStr for TokenTree {
    type Err = Error
}
*/
