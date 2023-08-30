use serde::{
    de::{self, Deserialize, MapAccess, SeqAccess, Visitor},
    ser::{Serialize, SerializeStruct},
    Deserialize as DeserializeDerive,
};
use std::hash::Hash;
use crate::proto::beco::ModifyNameRequest;

impl Hash for ModifyNameRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.calling_user.hash(state);
        self.user_id.hash(state);
        self.name.hash(state);
    }
}

impl Serialize for ModifyNameRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ModifyNameRequest", 3)?;
        state.serialize_field("user_id", &self.user_id)?;
        state.serialize_field("calling_user", &self.calling_user)?;
        state.serialize_field("name", &self.name)?;
        state.end()
    }
}

#[derive(DeserializeDerive)]
#[serde(field_identifier, rename_all = "snake_case")]
enum ModifyNameRequestFields {
    UserId,
    CallingUser,
    Name,
}

impl<'de> Deserialize<'de> for ModifyNameRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["user_id", "calling_user", "name"];

        struct RequestVisitor;

        impl<'de> Visitor<'de> for RequestVisitor {
            type Value = ModifyNameRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ModifyNameRequest")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let user_id = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let calling_user = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let name = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                Ok(ModifyNameRequest {
                    user_id,
                    calling_user,
                    name,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut user_id = None;
                let mut calling_user = None;
                let mut name = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        ModifyNameRequestFields::UserId => {
                            if user_id.is_some() {
                                return Err(de::Error::duplicate_field("user_id"));
                            }
                            user_id = Some(map.next_value()?);
                        }
                        ModifyNameRequestFields::CallingUser => {
                            if calling_user.is_some() {
                                return Err(de::Error::duplicate_field("calling_user"));
                            }
                            calling_user = Some(map.next_value()?);
                        }
                        ModifyNameRequestFields::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                    }
                }
                let user_id = user_id.ok_or_else(|| de::Error::missing_field("user_id"))?;
                let calling_user =
                    calling_user.ok_or_else(|| de::Error::missing_field("calling_user"))?;
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                Ok(ModifyNameRequest {
                    user_id,
                    calling_user,
                    name,
                })
            }
        }
        deserializer.deserialize_struct("ModifyNameRequest", FIELDS, RequestVisitor)
    }
}
