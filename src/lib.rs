#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]

mod evaluator;
mod gate;
mod requirement;
mod token;

pub use evaluator::Evaluator;
pub use requirement::Requirement;

pub type TerminalId = usize;
