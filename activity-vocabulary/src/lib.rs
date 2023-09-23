use std::{fmt::Display, str::FromStr};

use serde::{de::Visitor, Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/vocab.rs"));

#[derive(Clone, PartialEq, Eq, Debug, Hash, Default)]
pub enum Unit {
    Cm,
    Feet,
    Inches,
    Km,
    #[default]
    M,
    Miles,
    Uri(url::Url),
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cm => f.write_str("cm"),
            Self::Feet => f.write_str("feet"),
            Self::Inches => f.write_str("inches"),
            Self::Km => f.write_str("km"),
            Self::M => f.write_str("m"),
            Self::Miles => f.write_str("miles"),
            Self::Uri(uri) => uri.fmt(f),
        }
    }
}

impl FromStr for Unit {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "cm" => Self::Cm,
            "feet" => Self::Feet,
            "inches" => Self::Inches,
            "km" => Self::Km,
            "m" => Self::M,
            "miles" => Self::Miles,
            uri => Self::Uri(uri.parse()?),
        })
    }
}

impl Serialize for Unit {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct UnitVisitor;

impl<'de> Visitor<'de> for UnitVisitor {
    type Value = Unit;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Unit")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::default())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self::Value::default())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(serde::de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Unit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(UnitVisitor)
    }
}
