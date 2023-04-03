use redis::ToRedisArgs;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub trait FlagValue:
    PartialEq + Clone + Default + ToRedisArgs + ToString + Serialize + DeserializeOwned + 'static
{
}

impl FlagValue for bool {}
impl FlagValue for i32 {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Flag<V> {
    pub name: String,
    pub value: V,
}
