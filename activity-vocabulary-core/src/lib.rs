use std::{collections::HashMap, hash::Hash};

use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize};

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

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Context {
    urls: Vec<url::Url>,
    inline: HashMap<String, String>,
}

impl Serialize for Context {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.inline.is_empty() {
            if let &[url] = &self.urls.as_slice() {
                url.serialize(serializer)
            } else {
                self.urls.serialize(serializer)
            }
        } else {
            if self.urls.is_empty() {
                self.inline.serialize(serializer)
            } else {
                let mut serializer = serializer.serialize_seq(Some(self.urls.len() + 1))?;
                for url in &self.urls {
                    serializer.serialize_element(url)?;
                }
                serializer.serialize_element(&self.inline)?;
                serializer.end()
            }
        }
    }
}

enum ContextArrayElement {
    Url(url::Url),
    Inline(HashMap<String, String>),
}

struct ContextArrayElementVisitor;
impl<'de> Visitor<'de> for ContextArrayElementVisitor {
    type Value = ContextArrayElement;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("element of @context[]")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut r = HashMap::new();
        while let Some((k, v)) = map.next_entry::<String, String>()? {
            r.insert(k, v);
        }
        Ok(ContextArrayElement::Inline(r))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ContextArrayElement::Url(
            v.parse().map_err(serde::de::Error::custom)?,
        ))
    }
}

impl<'de> Deserialize<'de> for ContextArrayElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ContextArrayElementVisitor)
    }
}

struct ContextVisitor;
impl<'de> Visitor<'de> for ContextVisitor {
    type Value = Context;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("@context")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let visitor = ContextArrayElementVisitor;
        let ContextArrayElement::Url(url) = visitor.visit_str(v)? else {
            unreachable!()
        };
        Ok(Self::Value {
            urls: vec![url],
            inline: Default::default(),
        })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut inline = HashMap::new();
        let mut urls = Vec::new();
        while let Some(element) = seq.next_element::<ContextArrayElement>()? {
            match element {
                ContextArrayElement::Inline(new) => {
                    inline.extend(new);
                }
                ContextArrayElement::Url(url) => {
                    urls.push(url);
                }
            }
        }
        Ok(Self::Value { inline, urls })
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let visitor = ContextArrayElementVisitor;
        let ContextArrayElement::Inline(inline) = visitor.visit_map(map)? else {
            unreachable!()
        };
        Ok(Self::Value {
            inline,
            urls: Default::default(),
        })
    }
}

impl<'de> Deserialize<'de> for Context {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ContextVisitor)
    }
}

#[derive(Serialize, Deserialize)]
pub struct WithContext<T> {
    #[serde(rename = "@context")]
    pub context: Context,
    #[serde(flatten)]
    pub body: T,
}
