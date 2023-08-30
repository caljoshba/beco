use serde::{
    de::{self, Deserialize, MapAccess, SeqAccess, Visitor},
    ser::{Serialize, SerializeStruct},
    Deserialize as DeserializeDerive,
};
use std::hash::Hash;
use crate::proto::beco::ModifyOtherNamesRequest;

impl Hash for ModifyOtherNamesRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.calling_user.hash(state);
        self.user_id.hash(state);
        self.other_names.hash(state);
    }
}

impl Serialize for ModifyOtherNamesRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ModifyOtherNamesRequest", 3)?;
        state.serialize_field("user_id", &self.user_id)?;
        state.serialize_field("calling_user", &self.calling_user)?;
        state.serialize_field("other_names", &self.other_names)?;
        state.end()
    }
}

#[derive(DeserializeDerive)]
#[serde(field_identifier, rename_all = "snake_case")]
enum ModifyOtherNamesRequestFields {
    UserId,
    CallingUser,
    OtherNames,
}

impl<'de> Deserialize<'de> for ModifyOtherNamesRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["user_id", "calling_user", "other_names"];

        struct RequestVisitor;

        impl<'de> Visitor<'de> for RequestVisitor {
            type Value = ModifyOtherNamesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ModifyOtherNamesRequest")
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
                let other_names = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                Ok(ModifyOtherNamesRequest {
                    user_id,
                    calling_user,
                    other_names,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut user_id = None;
                let mut calling_user = None;
                let mut other_names = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        ModifyOtherNamesRequestFields::UserId => {
                            if user_id.is_some() {
                                return Err(de::Error::duplicate_field("user_id"));
                            }
                            user_id = Some(map.next_value()?);
                        }
                        ModifyOtherNamesRequestFields::CallingUser => {
                            if calling_user.is_some() {
                                return Err(de::Error::duplicate_field("calling_user"));
                            }
                            calling_user = Some(map.next_value()?);
                        }
                        ModifyOtherNamesRequestFields::OtherNames => {
                            if other_names.is_some() {
                                return Err(de::Error::duplicate_field("other_names"));
                            }
                            other_names = Some(map.next_value()?);
                        }
                    }
                }
                let user_id = user_id.ok_or_else(|| de::Error::missing_field("user_id"))?;
                let calling_user =
                    calling_user.ok_or_else(|| de::Error::missing_field("calling_user"))?;
                let other_names =
                    other_names.ok_or_else(|| de::Error::missing_field("other_names"))?;
                Ok(ModifyOtherNamesRequest {
                    user_id,
                    calling_user,
                    other_names,
                })
            }
        }
        deserializer.deserialize_struct("ModifyOtherNamesRequest", FIELDS, RequestVisitor)
    }
}