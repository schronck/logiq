#[derive(Clone, Copy, Debug, strum::EnumString, PartialEq, strum::Display)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Gate {
    And,
    Or,
    Not,
    Nand,
    Nor,
    Xor,
}
