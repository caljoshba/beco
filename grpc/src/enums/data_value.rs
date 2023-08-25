use serde::{
    de::{self, Deserialize, MapAccess, SeqAccess, Visitor},
    ser::{Serialize, SerializeStruct},
    Deserialize as DeserializeDerive, Serialize as SerializeDerive,
};
use strum::{Display, EnumString};

use crate::proto::beco::{AddAccountRequest, ModifyNameRequest, ModifyOtherNamesRequest};

#[derive(Debug, Clone, Display, EnumString, Eq, PartialEq)]
pub enum DataValue {
    #[strum(serialize = "FIRST_NAME")]
    FirstName,
    #[strum(serialize = "OTHER_NAMES")]
    OtherNames,
    #[strum(serialize = "LAST_NAME")]
    LastName,
    #[strum(serialize = "LINKED_USERS")]
    LinkedUsers,
    #[strum(serialize = "CHAIN_ACCOUNTS")]
    ChainAccounts,
}

#[derive(Debug, SerializeDerive, DeserializeDerive, Clone)]
pub enum DataRequests {
    FirstName(ModifyNameRequest),
    OtherNames(ModifyOtherNamesRequest),
    LastName(ModifyNameRequest),
    AddAccount(AddAccountRequest),
    // AddLinkedUser(ModifyLinkedUserRequest),
    // RemoveLinkedUser(ModifyLinkedUserRequest),
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

impl Serialize for AddAccountRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("AddAccountRequest", 4)?;
        state.serialize_field("blockchain", &self.blockchain)?;
        state.serialize_field("alias", &self.alias)?;
        state.serialize_field("user_id", &self.user_id)?;
        state.serialize_field("calling_user", &self.calling_user)?;
        state.end()
    }
}

#[derive(DeserializeDerive)]
#[serde(field_identifier, rename_all = "snake_case")]
enum AddAccountRequestFields {
    Blockchain,
    Alias,
    UserId,
    CallingUser,
}

impl<'de> Deserialize<'de> for AddAccountRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["blockchain", "alias", "user_id", "calling_user"];

        struct RequestVisitor;

        impl<'de> Visitor<'de> for RequestVisitor {
            type Value = AddAccountRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct AddAccountRequest")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let blockchain = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let alias = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let user_id = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let calling_user = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                Ok(AddAccountRequest {
                    blockchain,
                    alias,
                    user_id,
                    calling_user,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut blockchain = None;
                let mut alias = None;
                let mut user_id = None;
                let mut calling_user = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        AddAccountRequestFields::Blockchain => {
                            if blockchain.is_some() {
                                return Err(de::Error::duplicate_field("blockchain"));
                            }
                            blockchain = Some(map.next_value()?);
                        }
                        AddAccountRequestFields::Alias => {
                            if alias.is_some() {
                                return Err(de::Error::duplicate_field("alias"));
                            }
                            alias = Some(map.next_value()?);
                        }
                        AddAccountRequestFields::UserId => {
                            if user_id.is_some() {
                                return Err(de::Error::duplicate_field("user_id"));
                            }
                            user_id = Some(map.next_value()?);
                        }
                        AddAccountRequestFields::CallingUser => {
                            if calling_user.is_some() {
                                return Err(de::Error::duplicate_field("calling_user"));
                            }
                            calling_user = Some(map.next_value()?);
                        }
                    }
                }
                let blockchain =
                    blockchain.ok_or_else(|| de::Error::missing_field("blockchain"))?;
                let alias = alias.ok_or_else(|| de::Error::missing_field("alias"))?;
                let user_id = user_id.ok_or_else(|| de::Error::missing_field("user_id"))?;
                let calling_user =
                    calling_user.ok_or_else(|| de::Error::missing_field("calling_user"))?;
                Ok(AddAccountRequest {
                    blockchain,
                    alias,
                    user_id,
                    calling_user,
                })
            }
        }
        deserializer.deserialize_struct("AddAccountRequest", FIELDS, RequestVisitor)
    }
}
