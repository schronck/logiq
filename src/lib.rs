#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]

mod gate;
mod token;

pub use gate::Gate;
pub use token::{LogicTree, ParseError};

pub type TerminalId = u32;
