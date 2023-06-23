pub use crate::{gate::Gate, lexer::LexingError};
use logos::Logos;
use std::str::FromStr;
use strum::Display;

pub type TerminalId = u16;

#[derive(Debug, Logos, PartialEq, Display)]
#[logos(error = LexingError)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("(")]
    ParenOpen,

    #[token(")")]
    ParenClose,

    #[regex(
        "[0-9]+",
        |lex| TerminalId::from_str_radix(lex.slice(), 10),
        priority = 2
    )]
    TerminalId(TerminalId),

    #[regex(
        r"[a-zA-Z]+",
        |lex| {
            let gate_str = lex.slice().to_uppercase();
            Gate::from_str(&gate_str)
                .map_err(|_| LexingError::NoSuchGate(gate_str.to_string()))
        }
    )]
    Gate(Gate),
}
