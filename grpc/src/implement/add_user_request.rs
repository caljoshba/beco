use serde::{
    de::{self, Deserialize, MapAccess, SeqAccess, Visitor},
    ser::{Serialize, SerializeStruct},
    Deserialize as DeserializeDerive,
};
use std::hash::Hash;

use crate::proto::beco::AddUserRequest;

impl Hash for AddUserRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.calling_user.hash(state);
    }
}

impl Serialize for AddUserRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AddUserRequest", 4)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("calling_user", &self.calling_user)?;
        state.end()
    }
}

#[derive(DeserializeDerive)]
#[serde(field_identifier, rename_all = "snake_case")]
enum AddUserRequestFields {
    Name,
    CallingUser,
}

impl<'de> Deserialize<'de> for AddUserRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["name", "calling_user"];

        struct RequestVisitor;

        impl<'de> Visitor<'de> for RequestVisitor {
            type Value = AddUserRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct AddUserRequest")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let name = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let calling_user = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                Ok(AddUserRequest {
                    name,
                    calling_user,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut name = None;
                let mut calling_user = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        AddUserRequestFields::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        AddUserRequestFields::CallingUser => {
                            if calling_user.is_some() {
                                return Err(de::Error::duplicate_field("calling_user"));
                            }
                            calling_user = Some(map.next_value()?);
                        }
                    }
                }
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let calling_user = calling_user.ok_or_else(|| de::Error::missing_field("calling_user"))?;
                Ok(AddUserRequest {
                    name,
                    calling_user,
                })
            }
        }
        deserializer.deserialize_struct("AddUserRequest", FIELDS, RequestVisitor)
    }
}
