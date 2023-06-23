#![deny(clippy::all)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::redundant_clone)]
#![deny(clippy::clone_on_ref_ptr)]
#![deny(clippy::cargo)]

mod data;
mod eval;
mod lexer;
mod parser;

pub use self::{data::Gate, eval::eval, parser::parse};
