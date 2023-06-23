use crate::data::Gate;
use std::fmt;

#[derive(Clone, Debug, strum::Display, PartialEq)]
pub enum Value {
    Thruth(bool),
    Gate(Gate),
    List(List),
}

#[derive(Clone, Debug, PartialEq)]
pub struct List(pub Vec<Value>);

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "({})",
            self.0
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}
