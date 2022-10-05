mod parser;
mod scanner;

use crate::gate::Gate;
use parser::{parse, ParserError};
use scanner::{Scanner, ScannerError};

use std::str::FromStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Token {
    Whitespace,
    OpeningParenthesis,
    ClosingParenthesis,
    Terminal(char),
    Gate(Gate),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScannedTokens(Vec<Token>);

impl ScannedTokens {
    pub fn new(source: &str) -> Result<Self, ScannerError> {
        Scanner::scan(source)
    }

    pub fn tokens(&self) -> &[Token] {
        &self.0
    }
}

impl FromStr for ScannedTokens {
    type Err = ScannerError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::new(input)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedTokens(Vec<Token>);

impl ParsedTokens {
    pub fn new(scanned_tokens: &ScannedTokens) -> Result<Self, ParserError> {
        parse(scanned_tokens)
    }

    pub fn tokens(&self) -> &[Token] {
        &self.0
    }
}

impl FromStr for ParsedTokens {
    type Err = ParserError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let scanned_tokens = ScannedTokens::new(input)?;
        Self::new(&scanned_tokens)
    }
}

impl TryFrom<ScannedTokens> for ParsedTokens {
    type Error = ParserError;
    fn try_from(input: ScannedTokens) -> Result<Self, Self::Error> {
        Self::new(&input)
    }
}
