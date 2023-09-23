use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};

pub mod xsd;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, Hash)]
pub enum Remotable<T> {
    Remote(url::Url),
    Inline(T),
}

pub trait ObjectId {
    fn object_id(&self) -> Option<&url::Url>;
}

impl<T: ObjectId> ObjectId for Remotable<T> {
    fn object_id(&self) -> Option<&url::Url> {
        match self {
            Remotable::Remote(id) => Some(id),
            Remotable::Inline(object) => object.object_id(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct Property<T>(pub Vec<T>);

impl<T: Serialize> Serialize for Property<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let [inner] = &self.0[..] {
            inner.serialize(serializer)
        } else if self.0.len() > 1 {
            self.0.serialize(serializer)
        } else {
            serializer.serialize_none()
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Property<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let content = serde::__private::de::Content::deserialize(deserializer)?;
        let deserializer = serde::__private::de::ContentRefDeserializer::<D::Error>::new(&content);
        match Vec::<T>::deserialize(deserializer) {
            Ok(inner) => Ok(Self(inner)),
            Err(seq_err) => match Option::<T>::deserialize(deserializer) {
                Ok(inner) => Ok(Self(inner.into_iter().collect())),
                Err(opt_err) => Err(serde::de::Error::custom(format!("{seq_err} & {opt_err}"))),
            },
        }
    }
}

impl<T> Default for Property<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Or<T, U> {
    Prim(T),
    Snd(U),
}

impl<'de, L: Deserialize<'de>, R: Deserialize<'de>> Deserialize<'de> for Or<L, R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let content = serde::__private::de::Content::deserialize(deserializer)?;
        let deserializer = serde::__private::de::ContentRefDeserializer::<D::Error>::new(&content);
        match L::deserialize(deserializer) {
            Ok(left) => Ok(Self::Prim(left)),
            Err(left_err) => R::deserialize(deserializer)
                .map_err(|right_err| {
                    serde::de::Error::custom(format!("{left_err} and {right_err}"))
                })
                .map(Self::Snd),
        }
    }
}

impl<T: Serialize, U: Serialize> Serialize for Or<T, U> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Prim(value) => value.serialize(serializer),
            Self::Snd(value) => value.serialize(serializer),
        }
    }
}

//pub type Remotable<T> = Or<T, Link>;
pub type Untypable<T> = Or<T, serde_json::Value>;

impl<L, R> Or<L, R> {
    pub fn prim(&self) -> Option<&L> {
        match self {
            Self::Prim(l) => Some(l),
            Self::Snd(_) => None,
        }
    }
    pub fn snd(&self) -> Option<&R> {
        match self {
            Self::Prim(_) => None,
            Self::Snd(r) => Some(r),
        }
    }
}

impl<P, S> From<P> for Or<P, S> {
    fn from(value: P) -> Self {
        Or::Prim(value)
    }
}

impl<L: Default, R> Default for Or<L, R> {
    fn default() -> Self {
        Or::Prim(L::default())
    }
}

pub trait SkipSerialization {
    fn should_skip(&self) -> bool;
}

impl<T> SkipSerialization for Option<T> {
    fn should_skip(&self) -> bool {
        self.is_none()
    }
}

impl<T> SkipSerialization for Property<T> {
    fn should_skip(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LangContainer<T> {
    pub default: Option<T>,
    pub per_lang: HashMap<String, T>,
}

pub trait MergeableProperty {
    fn merge(&mut self, other: Self);
}

impl<T> MergeableProperty for Property<T> {
    fn merge(&mut self, other: Self) {
        self.0.extend(other.0.into_iter())
    }
}

impl<K: Eq + Hash, V> MergeableProperty for HashMap<K, V> {
    fn merge(&mut self, other: Self) {
        self.extend(other.into_iter())
    }
}

impl<T: MergeableProperty> MergeableProperty for Option<T> {
    fn merge(&mut self, other: Self) {
        match (self.as_mut(), other) {
            (Some(x), Some(y)) => x.merge(y),
            (None, Some(y)) => {
                *self = Some(y);
            }
            (Some(_), None) => (),
            (None, None) => (),
        }
    }
}
