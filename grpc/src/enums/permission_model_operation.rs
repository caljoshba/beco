use strum::{ Display, EnumString };

#[derive(Debug, Clone, Display, EnumString, Eq, PartialEq, Hash)]
pub enum PermissionModelOperation {
    #[strum(serialize = "PROPOSE")]
    PROPOSE,
    #[strum(serialize = "UPDATE")]
    UPDATE,
}
