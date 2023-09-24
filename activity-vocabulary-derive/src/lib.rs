use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{bail, Context};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use rust_format::{Formatter, RustFmt};
use serde::{Deserialize, Serialize};
use syn::{LitByteStr, LitStr};

const W3C_DOC_BASE: &str = "https://www.w3.org/TR/activitystreams-vocabulary";

fn doc_link(name: &str) -> String {
    format!("{W3C_DOC_BASE}/#dfn-{}", name.to_lowercase())
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
enum PropertyKind {
    Required,
    Functional,
    #[default]
    Normal,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum ContainerType {
    Language,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ContainerDef {
    #[serde(rename = "type")]
    container_type: ContainerType,
    tag: String,
    #[serde(default = "Default::default")]
    // another names of property
    aka: HashSet<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PropertyDef {
    #[serde(rename = "type")]
    property_type: String,
    tag: Option<String>,
    // another names of property
    #[serde(default = "Default::default")]
    aka: HashSet<String>,
    uri: String,
    doc: String,
    container: Option<ContainerDef>,
    #[serde(default = "PropertyKind::default")]
    kind: PropertyKind,
}

enum PropertyType {
    Simple(TokenStream),
    LanguageContainer {
        ty: TokenStream,
        default: TokenStream,
        per_lang: TokenStream,
    },
}

impl PropertyDef {
    fn property_type(&self) -> anyhow::Result<PropertyType> {
        let ty: TokenStream = syn::parse_str(&self.property_type)?;
        let ty = match self.kind {
            PropertyKind::Functional => quote!(Option<#ty>),
            PropertyKind::Normal => quote!(Property<#ty>),
            PropertyKind::Required => ty,
        };
        if let Some(container) = &self.container {
            match container.container_type {
                ContainerType::Language => Ok(PropertyType::LanguageContainer {
                    ty: quote!(LangContainer<#ty>),
                    default: quote!(Option<#ty>),
                    per_lang: quote!(std::collections::HashMap<String, #ty>),
                }),
            }
        } else {
            Ok(PropertyType::Simple(ty))
        }
    }

    fn tags(&self, name: &str) -> String {
        self.tag.clone().unwrap_or_else(|| name.to_owned())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum PreferredPropertyName {
    Simple(String),
    LangContainer { default: String, container: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TypeDef {
    uri: String,
    #[serde(default = "Default::default")]
    extends: HashSet<String>,
    #[serde(default = "Default::default")]
    properties: HashMap<String, PropertyDef>,
    subtype_name: String,
    #[serde(default = "Default::default")]
    as_object_id: bool,
    #[serde(default = "Default::default")]
    preferred_property_name: HashMap<String, PreferredPropertyName>,
    #[serde(default = "Default::default")]
    except_properties: HashSet<String>,
    doc: String,
}

fn collect_properties(
    name: &str,
    defs: &HashMap<String, TypeDef>,
) -> anyhow::Result<HashMap<String, PropertyDef>> {
    let def = defs
        .get(name)
        .with_context(|| format!("{name} not found"))?;
    let mut properties = def
        .extends
        .iter()
        .map(|super_type| collect_properties(super_type, defs))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .filter(|(name, _)| !def.except_properties.contains(name))
        .collect::<HashMap<_, _>>();
    properties.extend(def.properties.iter().map(|(k, v)| (k.clone(), v.clone())));
    Ok(properties)
}

fn generate_properties(properties: &HashMap<String, PropertyDef>) -> anyhow::Result<TokenStream> {
    let tokens = properties
        .iter()
        .map(|(name, def)| {
            let quoted_name = Ident::new(name, Span::call_site());
            let quoted_type = match def.property_type()? {
                PropertyType::LanguageContainer { ty, .. } => ty,
                PropertyType::Simple(ty) => ty,
            };
            let tag = def.tags(name);
            let doc_comment = format!(
                "`{}`\n\n[W3C recommendation]({})\n\n{}",
                def.uri,
                doc_link(&tag),
                def.doc
            );
            let doc_lit = LitStr::new(&doc_comment, Span::call_site());
            let quoted = quote! {
                #[doc = #doc_lit]
                pub #quoted_name: #quoted_type,
            };
            Ok::<_, anyhow::Error>(quoted)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();
    Ok(tokens)
}

fn generate_serialize_impl(
    name: &str,
    def: &TypeDef,
    properties: &HashMap<String, PropertyDef>,
) -> anyhow::Result<TokenStream> {
    let name_ident = Ident::new(name, Span::call_site());
    let serializings = properties
        .iter()
        .map(|(name, property_def)| {
            let name_ident = Ident::new(name, Span::call_site());
            let tag = property_def.tags(name);
            if let Some(container) = &property_def.container {
                match container.container_type {
                    ContainerType::Language => {
                        let (tag, map_tag) = def.preferred_property_name.get(name).map(|prop| {
                            let PreferredPropertyName::LangContainer { default, container } = prop else {
                                bail!("container expected")
                            };
                            Ok((default, container))
                        }).unwrap_or_else(|| Ok((name, &container.tag)))?;
                        Ok(quote! {
                            if self.#name_ident.default.is_some() {
                                serializer.serialize_entry(#tag, &self.#name_ident.default)?;
                            }
                            if !self.#name_ident.per_lang.is_empty() {
                                serializer.serialize_entry(#map_tag, &self.#name_ident.per_lang)?;
                            }
                        })
                    }
                }
            } else {
                let tag = def.preferred_property_name.get(name).map(|prop| {
                    let PreferredPropertyName::Simple(tag) = prop else {
                        bail!("simple property expected")
                    };
                    Ok(tag)
                }).unwrap_or_else(|| Ok(&tag))?;
                if property_def.kind != PropertyKind::Required {
                    Ok(quote! {
                        if !activity_vocabulary_core::SkipSerialization::should_skip(&self.#name_ident) {
                            serializer.serialize_entry(#tag, &self.#name_ident)?;
                        }
                    })
                }
                else {
                    Ok(quote! { serializer.serialize_entry(#tag, &self.#name_ident)?; })
                }
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<TokenStream>();
    Ok(quote! {
        #[allow(non_snake_case, unused_mut, unused_imports, dead_code, unused_attributes, clippy::match_single_binding)]
        const _: () = {
            use serde::{Serialize, Deserialize};
            impl Serialize for #name_ident {
                #[automatically_derived]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    use serde::ser::SerializeMap;
                    let mut serializer = serializer.serialize_map(None)?;
                    #serializings
                    serializer.end()
                }
            }
        };
    })
}

fn generate_deserialize_impl(
    name: &str,
    properties: &HashMap<String, PropertyDef>,
) -> anyhow::Result<TokenStream> {
    let name_ident = Ident::new(name, Span::call_site());
    let field_tags = properties
        .iter()
        .map(|(name, prop_def)| {
            let tag = prop_def.tags(name);
            let aka = prop_def
                .aka
                .iter()
                .map(|tag| quote!(#tag,))
                .collect::<TokenStream>();
            if let Some(container) = &prop_def.container {
                let container_aka = container
                    .aka
                    .iter()
                    .map(|tag| quote!(#tag, ))
                    .collect::<TokenStream>();
                match container.container_type {
                    ContainerType::Language => {
                        let map_tag = &container.tag;
                        Ok(quote! {#aka #container_aka #tag, #map_tag, })
                    }
                }
            } else {
                Ok(quote! {#aka #tag, })
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<TokenStream>();

    let fields_enum_defs = properties
        .iter()
        .map(|(name, def)| {
            let ident = Ident::new(name, Span::call_site());
            let aka = def
                .aka
                .iter()
                .map(|name| quote!(#name, ))
                .collect::<TokenStream>();
            if let Some(container) = &def.container {
                match container.container_type {
                    ContainerType::Language => {
                        let map_ident = Ident::new(&container.tag, Span::call_site());
                        let container_aka = container
                            .aka
                            .iter()
                            .map(|tag| quote!(#tag, ))
                            .collect::<TokenStream>();
                        quote!(#aka #container_aka #ident, #map_ident,)
                    }
                }
            } else {
                quote!(#aka #ident,)
            }
        })
        .collect::<TokenStream>();

    let field_enum_match_arms = |bytes: bool| {
        properties
            .iter()
            .map(|(name, def)| {
                let tag = def.tags(name);
                let name = Ident::new(name, Span::call_site());
                let arms = def
                    .aka
                    .iter()
                    .chain(std::iter::once(&tag))
                    .map(|tag| {
                        let tag = if bytes {
                            LitByteStr::new(tag.as_bytes(), Span::call_site()).into_token_stream()
                        } else {
                            tag.into_token_stream()
                        };
                        quote!(#tag => Ok(__Fields::#name),)
                    })
                    .collect::<TokenStream>();
                if let Some(container) = &def.container {
                    let map_ident = Ident::new(&container.tag, Span::call_site());
                    let container_arms = container
                        .aka
                        .iter()
                        .chain(std::iter::once(&container.tag))
                        .map(|tag| {
                            let tag = if bytes {
                                LitByteStr::new(container.tag.as_bytes(), Span::call_site())
                                    .into_token_stream()
                            } else {
                                quote!(#tag)
                            };
                            quote!(#tag => Ok(__Fields::#map_ident),)
                        })
                        .collect::<TokenStream>();
                    quote!(#arms #container_arms)
                } else {
                    arms
                }
            })
            .collect::<TokenStream>()
    };

    let field_enum_match_arms_str = field_enum_match_arms(false);
    let field_enum_match_arms_bytes = field_enum_match_arms(true);

    let field_placeholders = properties
        .iter()
        .map(|(name, def)| {
            let name = Ident::new(name, Span::call_site());
            Ok::<_, anyhow::Error>(match def.property_type()? {
                PropertyType::LanguageContainer {
                    default, per_lang, ..
                } => {
                    if let Some(container) = &def.container {
                        let name_map = Ident::new(&container.tag, Span::call_site());
                        quote! {
                            let mut #name = Option::<#default>::None;
                            let mut #name_map = Option::<#per_lang>::None;
                        }
                    } else {
                        unreachable!()
                    }
                }
                PropertyType::Simple(ty) => quote! {
                    let mut #name = Option::<#ty>::None;
                },
            })
        })
        .collect::<Result<TokenStream, _>>()?;

    let deserialize_match_arms = properties
        .iter()
        .map(|(name, def)| {
            let name_ident = Ident::new(name, Span::call_site());
            if let Some(container) = &def.container {
                let PropertyType::LanguageContainer { default, per_lang, .. } = def.property_type()? else {
                    bail!("language container expected")
                };
                let map_name = &container.tag;
                let map_name_ident = Ident::new(map_name, Span::call_site());
                if PropertyKind::Normal == def.kind {
                    Ok(quote!{
                        __Fields::#name_ident => {
                            if let Some(occupied) = #name_ident.as_mut() {
                                occupied.merge(__map.next_value::<#default>()?);
                            }
                            else {
                                #name_ident = Some(__map.next_value::<#default>()?);
                            }
                        }
                        __Fields::#map_name_ident => {
                            if let Some(occupied) = #map_name_ident.as_mut() {
                                occupied.merge(__map.next_value::<#per_lang>()?);
                            }
                            else {
                                #map_name_ident = Some(__map.next_value::<#per_lang>()?);
                            }
                        }
                    })
                }
                else {
                    Ok(quote!{
                        __Fields::#name_ident => {
                            if #name_ident.is_some() {
                                return Err(<A::Error as serde::de::Error>::duplicate_field(#name));
                            }
                            else {
                                #name_ident = Some(__map.next_value::<#default>()?);
                            }
                        }
                        __Fields::#map_name_ident => {
                            if #map_name_ident.is_some() {
                                return Err(<A::Error as serde::de::Error>::duplicate_field(#map_name));
                            }
                            else {
                                #map_name_ident = Some(__map.next_value::<#per_lang>()?);
                            }
                        }
                    })
                }
            } else {
                let PropertyType::Simple(ty) = def.property_type()? else {
                    bail!("simple type expected")
                };
                if PropertyKind::Normal == def.kind {
                    Ok(quote!(
                        __Fields::#name_ident => {
                            if let Some(occupied) = #name_ident.as_mut() {
                                occupied.merge(__map.next_value::<#ty>()?);
                            }
                            else {
                                #name_ident = Some(__map.next_value::<#ty>()?);
                            }
                        }
                    ))
                }
                else {
                    Ok(quote!(
                        __Fields::#name_ident => {
                            if #name_ident.is_some() {
                                return Err(<A::Error as serde::de::Error>::duplicate_field(#name));
                            }
                            else {
                                #name_ident = Some(__map.next_value::<#ty>()?);
                            }
                        }
                    ))
                }
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<TokenStream>();

    let build_type = properties
        .iter()
        .map(|(name, def)| {
            if def.kind == PropertyKind::Required {
                let name_ident = Ident::new(name, Span::call_site());
                if let Some(container) = &def.container {
                    let map_name = &container.tag;
                    let map_name_ident = Ident::new(&map_name, Span::call_site());
                    Ok(quote! {
                        #name_ident: LangContainer {
                            default: #name_ident.ok_or_else(|| serde::de::Error::missing_field(#name))?,
                            per_lang: #map_name_ident.ok_or_else(|| serde::de::Error::missing_field(#map_name))?,
                        },
                    })
                } else {
                    Ok(quote! {
                        #name_ident: #name_ident.ok_or_else(|| serde::de::Error::missing_field(#name))?,
                    })
                }
            } else {
                let name_ident = Ident::new(name, Span::call_site());
                if let Some(container) = &def.container {
                    let map_name_ident = Ident::new(&container.tag, Span::call_site());
                    Ok(quote! {
                        #name_ident: LangContainer {
                            default: #name_ident.unwrap_or_default(),
                            per_lang: #map_name_ident.unwrap_or_default(),
                        },
                    })
                } else {
                    Ok(quote! {
                        #name_ident: #name_ident.unwrap_or_default(),
                    })
                }
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<TokenStream>();

    Ok(quote! {
        #[allow(non_snake_case, unused_mut, unused_imports, dead_code, unused_attributes, clippy::match_single_binding)]
        const _:() = {
            use serde::de::{Deserialize, Visitor, Deserializer};

            impl<'de> Deserialize<'de> for #name_ident {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    const FIELDS: &[&str] = &[#field_tags];

                    #[derive(Debug)]
                    #[allow(non_camel_case_types)]
                    enum __Fields {
                        #fields_enum_defs
                        __Ignore,
                    }

                    struct __FieldsVisitor;

                    impl<'de> Visitor<'de> for __FieldsVisitor {
                        type Value = __Fields;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str("field identifier")
                        }

                        fn visit_str<E: serde::de::Error>(
                            self,
                            value: &str
                        ) -> Result<Self::Value, E> {
                            match value {
                                #field_enum_match_arms_str
                                _ => Ok(__Fields::__Ignore),
                            }
                        }

                        fn visit_bytes<E: serde::de::Error>(
                            self,
                            value: &[u8]
                        ) -> Result<Self::Value, E> {
                            match value {
                                #field_enum_match_arms_bytes
                                _ => Ok(__Fields::__Ignore)
                            }
                        }
                    }

                    impl<'de> Deserialize<'de> for __Fields {
                        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                            deserializer.deserialize_identifier(__FieldsVisitor)
                        }
                    }

                    struct __Visitor;

                    impl<'de> Visitor<'de> for __Visitor {
                        type Value = #name_ident;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str(#name)
                        }

                        fn visit_map<A>(
                            self,
                            mut __map: A,
                        ) -> Result<Self::Value, A::Error>
                            where
                                A: serde::de::MapAccess<'de>,
                        {
                            #field_placeholders
                            while let Some(__key) = __map.next_key::<__Fields>()? {
                                match __key {
                                    #deserialize_match_arms
                                    _ => {
                                        let _ = __map.next_value::<serde::de::IgnoredAny>();
                                    }
                                }
                            }
                            Ok(Self::Value { #build_type })
                        }
                    }

                    deserializer.deserialize_struct(#name, FIELDS, __Visitor)
                }
            }
        };
    })
}

fn collect_subtypes(
    name: &str,
    defs: &HashMap<String, TypeDef>,
) -> anyhow::Result<HashMap<String, TypeDef>> {
    let mut subtypes = HashMap::new();
    let mut queue = vec![name];
    while let Some(name) = queue.pop() {
        let def = defs
            .get(name)
            .with_context(|| format!("{name} not found"))?;
        subtypes.insert(name.to_owned(), def.clone());
        for (subtype_name, subtype_def) in defs {
            if !subtypes.contains_key(subtype_name) && subtype_def.extends.contains(name) {
                queue.push(&subtype_name);
            }
        }
    }
    Ok(subtypes)
}

fn generate_subtypes(name: &str, defs: &HashMap<String, TypeDef>) -> anyhow::Result<TokenStream> {
    let def = defs
        .get(name)
        .with_context(|| format!("{name} not found"))?;
    let subtypes = collect_subtypes(name, defs)?;
    let name_ident = Ident::new(name, Span::call_site());

    let subtype_arms = subtypes
        .iter()
        .map(|(name, _)| {
            let subtype_ident = Ident::new(&name, Span::call_site());
            quote! {
                #subtype_ident(#subtype_ident),
            }
        })
        .collect::<TokenStream>();
    let subtype_ident = Ident::new(&def.subtype_name, Span::call_site());

    let subtypes_upcast_arms = subtypes
        .iter()
        .map(|(name, _)| {
            let ident = Ident::new(name, Span::call_site());
            quote! { #subtype_ident::#ident(x) => x.into(), }
        })
        .collect::<TokenStream>();

    let subtype_upcasts = subtypes
        .iter()
        .filter(|(subtype_name, _)| name != *subtype_name)
        .map(|(subtype_name, def)| {
            let super_properties = collect_properties(name, defs)?;
            let sub_properties = collect_properties(&subtype_name, defs)?;
            let sub_ident = Ident::new(subtype_name, Span::call_site());
            let common_property_into = super_properties
                .iter()
                .filter(|(name, _)| sub_properties.iter().any(|(sub_name, _)| sub_name == *name))
                .map(|(name, _)| {
                    let ident = Ident::new(name, Span::call_site());
                    quote! {
                        #ident: value.#ident.into(),
                    }
                })
                .collect::<TokenStream>();
            let fill_properties = super_properties
                .iter()
                .filter(|(name, _)| !sub_properties.iter().any(|(sub_name, _)| sub_name == *name))
                .map(|(name, _)| {
                    let ident = Ident::new(name, Span::call_site());
                    quote! {
                        #ident: Default::default(),
                    }
                })
                .collect::<TokenStream>();
            let sub_subtypes_ident = Ident::new(&def.subtype_name, Span::call_site());
            Ok(quote! {
                impl From<#sub_subtypes_ident> for #name_ident {
                    fn from(value: #sub_subtypes_ident) -> Self {
                        Into::<#sub_ident>::into(value).into()
                    }
                }

                impl From<#sub_ident> for #name_ident {
                    fn from(value: #sub_ident) -> Self {
                        Self {
                            #common_property_into
                            #fill_properties
                        }
                    }
                }
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<TokenStream>();
    let doc_comment = format!(
        "`{}`\n\n[W3C recommendation]({})\n\n{}",
        def.uri,
        doc_link(name),
        def.doc
    );
    let doc_lit = LitStr::new(&doc_comment, Span::call_site());
    Ok(quote! {
        #[derive(serde::Serialize, Debug, Clone, PartialEq)]
        #[serde(tag = "type")]
        #[doc = #doc_lit]
        pub enum #subtype_ident {
            #subtype_arms
        }

        impl From<#subtype_ident> for #name_ident {
            fn from(value: #subtype_ident) -> Self {
                match value {
                    #subtypes_upcast_arms
                }
            }
        }

        #subtype_upcasts
    })
}

fn generate_subtypes_deserialize(
    name: &str,
    defs: &HashMap<String, TypeDef>,
) -> anyhow::Result<TokenStream> {
    let name_ident = Ident::new(name, Span::call_site());
    let def = defs.get(name).with_context(|| format!("missing {name}"))?;
    let subtype_ident = Ident::new(&def.subtype_name, Span::call_site());
    let subtypes = collect_subtypes(name, defs)?;
    let fields_contents = subtypes
        .keys()
        .map(|name| {
            let ident = Ident::new(name, Span::call_site());
            quote!(#ident, )
        })
        .collect::<TokenStream>();
    let match_arms = subtypes
        .keys()
        .map(|name| {
            let ident = Ident::new(name, Span::call_site());
            quote!(
                __Fields::#ident => Ok(Self::#ident(#ident::deserialize(deserializer)?)),
            )
        })
        .collect::<TokenStream>();
    let field_enum_match_arms = |bytes: bool| {
        subtypes
            .keys()
            .map(|subtype| {
                let subtype_ident = Ident::new(subtype, Span::call_site());
                let v = if bytes {
                    LitByteStr::new(subtype.as_bytes(), Span::call_site()).into_token_stream()
                } else {
                    LitStr::new(subtype, Span::call_site()).into_token_stream()
                };
                quote!(#v => Ok(__Fields::#subtype_ident), )
            })
            .collect::<TokenStream>()
    };
    let field_enum_match_arms_str = field_enum_match_arms(false);
    let field_enum_match_arms_bytes = field_enum_match_arms(true);

    Ok(quote! {
        const _:() = {
            impl<'de> Deserialize<'de> for #subtype_ident {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    use activity_vocabulary_core::TaggedContentVisitor;
                    #[derive(Debug)]
                    enum __Fields {
                        #fields_contents
                    }

                    struct __FieldsVisitor;

                    impl<'de> Visitor<'de> for __FieldsVisitor {
                        type Value = __Fields;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str("type")
                        }

                        fn visit_str<E: serde::de::Error>(
                            self,
                            value: &str
                        ) -> Result<Self::Value, E> {
                            match value {
                                #field_enum_match_arms_str
                                _ => Err(serde::de::Error::missing_field("type"))
                            }
                        }

                        fn visit_bytes<E: serde::de::Error>(
                            self,
                            value: &[u8]
                        ) -> Result<Self::Value, E> {
                            match value {
                                #field_enum_match_arms_bytes
                                _ => Err(serde::de::Error::missing_field("type")),
                            }
                        }
                    }

                    impl<'de> Deserialize<'de> for __Fields {
                        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                        where
                            D: serde::Deserializer<'de>,
                        {
                            deserializer.deserialize_any(__FieldsVisitor)
                        }
                    }

                    let (tag, content) = deserializer.deserialize_any(TaggedContentVisitor::<__Fields>::new(#name, "type"))?;
                    let deserializer = serde_value::ValueDeserializer::new(content);
                    match tag {
                        #match_arms
                    }
                }
            }
        };

    })
}

fn generate_types(defs: HashMap<String, TypeDef>) -> anyhow::Result<TokenStream> {
    let mut token = TokenStream::new();
    for (name, def) in &defs {
        let properties = collect_properties(name, &defs)?;
        let quote_properties = generate_properties(&properties)?;
        let quote_name = Ident::new(name, Span::call_site());
        let quote_subtype = generate_subtypes(name, &defs)?;
        let doc_comment = format!(
            "`{}`\n\n[W3C recommendation]({})\n\n{}",
            def.uri,
            doc_link(name),
            def.doc
        );
        let doc_lit = LitStr::new(&doc_comment, Span::call_site());
        token.append_all(quote! {
            #[derive(Debug, Clone, PartialEq)]
            #[doc = #doc_lit]
            pub struct #quote_name {
                #quote_properties
            }
        });
        token.append_all(generate_serialize_impl(name, def, &properties)?);
        token.append_all(generate_deserialize_impl(name, &properties)?);
        token.append_all(quote_subtype);
        token.append_all(generate_subtypes_deserialize(name, &defs)?);
    }
    Ok(token)
}

pub fn define_types<P: AsRef<Path>>(path: P) -> anyhow::Result<String> {
    let mut reader = fs::File::open(path)?;
    let def = serde_yaml::from_reader(&mut reader)?;
    let src = generate_types(def)?;
    let src = quote! {
        use activity_vocabulary_core::*;
        #src
    };
    let src = RustFmt::new().format_tokens(src)?;
    Ok(src)
}
