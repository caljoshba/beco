use serde::{
    de::{self, Deserialize, MapAccess, SeqAccess, Visitor},
    ser::{Serialize, SerializeStruct},
    Deserialize as DeserializeDerive,
};
use std::hash::Hash;

use crate::proto::beco::ModifyLinkedUserRequest;

impl Hash for ModifyLinkedUserRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.user_id.hash(state);
        self.calling_user.hash(state);
    }
}

impl Serialize for ModifyLinkedUserRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ModifyLinkedUserRequest", 4)?;
        state.serialize_field("user_id", &self.user_id)?;
        state.serialize_field("calling_user", &self.calling_user)?;
        state.end()
    }
}

#[derive(DeserializeDerive)]
#[serde(field_identifier, rename_all = "snake_case")]
enum ModifyLinkedUserRequestFields {
    UserId,
    CallingUser,
}

impl<'de> Deserialize<'de> for ModifyLinkedUserRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["user_id", "calling_user"];

        struct RequestVisitor;

        impl<'de> Visitor<'de> for RequestVisitor {
            type Value = ModifyLinkedUserRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ModifyLinkedUserRequest")
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
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                Ok(ModifyLinkedUserRequest {
                    user_id,
                    calling_user,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut user_id = None;
                let mut calling_user = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        ModifyLinkedUserRequestFields::UserId => {
                            if user_id.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            user_id = Some(map.next_value()?);
                        }
                        ModifyLinkedUserRequestFields::CallingUser => {
                            if calling_user.is_some() {
                                return Err(de::Error::duplicate_field("calling_user"));
                            }
                            calling_user = Some(map.next_value()?);
                        }
                    }
                }
                let user_id = user_id.ok_or_else(|| de::Error::missing_field("user_id"))?;
                let calling_user = calling_user.ok_or_else(|| de::Error::missing_field("calling_user"))?;
                Ok(ModifyLinkedUserRequest {
                    user_id,
                    calling_user,
                })
            }
        }
        deserializer.deserialize_struct("ModifyLinkedUserRequest", FIELDS, RequestVisitor)
    }
}
