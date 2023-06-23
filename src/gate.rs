use strum::{Display, EnumString};

#[derive(Debug, EnumString, PartialEq, Display)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Gate {
    And,
    Or,
    Not,
    Nand,
    Nor,
    Xor,
}
