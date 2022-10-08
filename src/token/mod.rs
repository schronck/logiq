mod parser;
mod scanner;

use crate::gate::Gate;
pub use parser::{Parser, ParserError};

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
