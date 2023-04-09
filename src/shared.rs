use std::fmt::Formatter;

use redis::{NumericBehavior, RedisWrite, ToRedisArgs};
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, Default)]
pub enum FlagValue {
    #[default]
    Null,
    Boolean(bool),
    String(String),
    Number(f64),
}

impl Serialize for FlagValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            FlagValue::Null => serializer.serialize_none(),
            &FlagValue::Boolean(value) => serializer.serialize_bool(value),
            FlagValue::String(value) => serializer.serialize_str(value),
            &FlagValue::Number(value) => serializer.serialize_f64(value),
        }
    }
}

struct FlagValueVisitor;

impl<'de> Visitor<'de> for FlagValueVisitor {
    type Value = FlagValue;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a flag value: null, boolean, number or string")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FlagValue::Boolean(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FlagValue::Number(v as _))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FlagValue::Number(v as _))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FlagValue::Number(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FlagValue::String(v.to_string()))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FlagValue::Null)
    }
}

impl<'de> Deserialize<'de> for FlagValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(FlagValueVisitor)
    }
}

impl ToRedisArgs for FlagValue {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        match self {
            FlagValue::Null => out.write_arg(b"null:null"),
            FlagValue::Boolean(value) => out.write_arg_fmt(format_args!("boolean:{value}")),
            FlagValue::String(value) => out.write_arg_fmt(format_args!("string:{value}")),
            FlagValue::Number(value) => out.write_arg_fmt(format_args!("number:{value}")),
        }
    }

    fn describe_numeric_behavior(&self) -> NumericBehavior {
        match self {
            FlagValue::Null | FlagValue::Boolean(_) | FlagValue::String(_) => {
                NumericBehavior::NonNumeric
            }
            FlagValue::Number(_) => NumericBehavior::NumberIsFloat,
        }
    }
}

impl ToString for FlagValue {
    fn to_string(&self) -> String {
        match self {
            FlagValue::Null => "null".to_string(),
            FlagValue::Boolean(value) => value.to_string(),
            FlagValue::String(value) => value.clone(),
            FlagValue::Number(value) => value.to_string(),
        }
    }
}

impl TryFrom<&str> for FlagValue {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let [type_desc, value] = value
            .splitn(2, ':')
            .next_chunk()
            .map_err(|_| "invalid format, expected type:value")?;
        let flag_value = match type_desc {
            "null" => FlagValue::Null,
            "boolean" => FlagValue::Boolean(match value {
                "true" => true,
                "false" => false,
                _ => return Err("invalid boolean value"),
            }),
            "string" => FlagValue::String(value.to_string()),
            "number" => FlagValue::Number(value.parse().map_err(|_| "invalid number")?),
            _ => return Err("unrecognized type"),
        };
        Ok(flag_value)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Flag {
    pub name: String,
    pub value: FlagValue,
}
