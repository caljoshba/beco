use strum::{ Display, EnumString };

#[derive(Debug, Clone, Display, EnumString, Eq, PartialEq)]
pub enum EVMAlgortithm {
    #[strum(serialize = "ECDSA")]
    ECDSA
}