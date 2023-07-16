use strum::{ Display, EnumString };

#[derive(Debug, Clone, Display, EnumString, Eq, PartialEq)]
pub enum Blockchain {
    #[strum(serialize = "XRPL")]
    XRPL,
}