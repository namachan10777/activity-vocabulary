use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Context};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use rust_format::{Formatter, RustFmt};
use serde::Deserialize;
use syn::{LitByteStr, LitStr, Type};

#[derive(Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub enum PropertyKind {
    Required,
    Functional,
    #[default]
    Normal,
}

#[derive(Deserialize, Clone)]
pub enum PropertyDef {
    Simple {
        #[serde(default)]
        tag: Option<String>,
        #[serde(rename = "type")]
        property_type: String,
        #[serde(default)]
        aka: HashSet<String>,
        uri: String,
        doc: String,
        #[serde(default)]
        kind: PropertyKind,
    },
    LangContainer {
        #[serde(default)]
        tag: Option<String>,
        #[serde(rename = "type")]
        property_type: String,
        container_tag: String,
        #[serde(default)]
        aka: HashSet<String>,
        #[serde(default)]
        container_aka: HashSet<String>,
        uri: String,
        doc: String,
        #[serde(default)]
        kind: PropertyKind,
    },
}

#[derive(Deserialize, Clone)]
pub enum PreferredPropertyName {
    Simple(String),
    LangContainer { default: String, container: String },
}

#[derive(Deserialize, Clone)]
pub struct TypeDef {
    pub uri: String,
    #[serde(default)]
    pub extends: HashSet<String>,
    pub properties: HashMap<String, PropertyDef>,
    #[serde(default)]
    pub preferred_property_name: HashMap<String, PreferredPropertyName>,
    #[serde(default)]
    pub except_properties: HashSet<String>,
    pub doc: String,
}

impl PropertyKind {
    fn wrap_type(&self, ty: syn::Type) -> Type {
        match self {
            Self::Functional => syn::parse2(quote!(Option<#ty>)).unwrap(),
            Self::Normal => syn::parse2(quote!(::activity_vocabulary_core::Property<#ty>)).unwrap(),
            Self::Required => ty,
        }
    }

    fn serializing_stmt(
        &self,
        serializer: TokenStream,
        tag: &str,
        property: TokenStream,
    ) -> TokenStream {
        if self == &Self::Required {
            quote! {
                #serializer.serialize(#tag, #property)?;
            }
        } else {
            quote! {
                if !#property.should_skip() {
                    #serializer.serialize(#tag, #property)?;
                }
            }
        }
    }
}

impl PropertyDef {
    fn gen_type(&self) -> anyhow::Result<syn::Type> {
        match self {
            PropertyDef::Simple {
                kind,
                property_type,
                ..
            } => Ok(kind.wrap_type(
                syn::parse_str(&property_type).with_context(|| format!("parse {property_type}"))?,
            )),
            PropertyDef::LangContainer {
                property_type,
                kind,
                ..
            } => {
                let ty: syn::Type = syn::parse_str(&property_type)
                    .with_context(|| format!("parse {property_type}"))?;
                if kind == &PropertyKind::Normal {
                    Ok(
                        syn::parse2(quote!(::activity_vocabulary_core::LangContainer<::activity_vocabulary_core::Property<#ty>>))
                            .unwrap(),
                    )
                } else {
                    Ok(
                        syn::parse2(quote!(::activity_vocabulary_core::LangContainer<#ty>))
                            .unwrap(),
                    )
                }
            }
        }
    }
}

fn rename_default_name(
    type_def: &TypeDef,
    property_name: &str,
    property_def: PropertyDef,
) -> anyhow::Result<PropertyDef> {
    match (
        type_def.preferred_property_name.get(property_name),
        property_def,
    ) {
        (
            Some(PreferredPropertyName::Simple(preferred)),
            PropertyDef::Simple {
                tag,
                mut aka,
                uri,
                doc,
                kind,
                property_type,
            },
        ) => {
            let default_name = tag.unwrap_or_else(|| property_name.to_owned());
            aka.insert(default_name);
            let def = PropertyDef::Simple {
                tag: Some(preferred.clone()),
                aka,
                uri,
                doc,
                kind,
                property_type,
            };
            Ok(def)
        }
        (
            Some(PreferredPropertyName::LangContainer { default, container }),
            PropertyDef::LangContainer {
                tag,
                container_tag,
                mut aka,
                mut container_aka,
                uri,
                doc,
                kind,
                property_type,
            },
        ) => {
            let default_tag = tag.unwrap_or_else(|| property_name.to_owned());
            aka.insert(default_tag);
            container_aka.insert(container_tag);
            Ok(PropertyDef::LangContainer {
                tag: Some(default.to_owned()),
                container_tag: container.to_owned(),
                aka,
                container_aka,
                uri,
                doc,
                kind,
                property_type,
            })
        }
        (None, def) => Ok(def),
        _ => Err(anyhow!(
            "preferred name and property def type for {property_name} unmatched"
        )),
    }
}

fn collect_properties(
    type_def: &TypeDef,
    full_defs: &HashMap<String, TypeDef>,
) -> anyhow::Result<HashMap<String, PropertyDef>> {
    let properties = type_def
        .extends
        .iter()
        .map(|super_name| {
            let super_def = full_defs
                .get(super_name)
                .with_context(|| format!("type {super_name} not found"))?;
            collect_properties(super_def, full_defs)
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let properties = properties
        .into_iter()
        .chain(type_def.properties.clone().into_iter())
        .filter(|(name, _)| !type_def.except_properties.contains(name))
        .map(|(name, def)| rename_default_name(type_def, &name, def).map(|def| (name, def)))
        .collect::<anyhow::Result<HashMap<String, PropertyDef>>>()?;
    Ok(properties)
}

fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

fn gen_type(
    type_name: &str,
    type_def: &TypeDef,
    full_defs: &HashMap<String, TypeDef>,
) -> anyhow::Result<TokenStream> {
    let properties = collect_properties(type_def, full_defs)?
        .iter()
        .map(|(name, def)| {
            let ty = def.gen_type()?;
            let name = ident(name);
            Ok(quote!(pub #name: #ty, ))
        })
        .collect::<anyhow::Result<TokenStream>>()?;
    let type_name = ident(type_name);
    Ok(quote! {
        pub struct #type_name {
            #properties
        }
    })
}

fn gen_serialize_stmt(serializer: TokenStream, name: String, def: PropertyDef) -> TokenStream {
    let name_ident = ident(&name);
    match def {
        PropertyDef::Simple { tag, kind, .. } => {
            let tag = tag.unwrap_or(name);
            kind.serializing_stmt(serializer, &tag, quote!(&self.#name_ident))
        }
        PropertyDef::LangContainer {
            tag,
            container_tag,
            kind,
            ..
        } => {
            let tag = tag.unwrap_or(name);
            let default =
                kind.serializing_stmt(serializer.clone(), &tag, quote!(&self.#name_ident.default));
            let per_lang = kind.serializing_stmt(
                serializer,
                &container_tag,
                quote!(&self.#name_ident.per_lang),
            );

            quote! {
                #default
                #per_lang
            }
        }
    }
}

fn gen_serialize_impl(
    type_name: &str,
    type_def: &TypeDef,
    full_defs: &HashMap<String, TypeDef>,
) -> anyhow::Result<TokenStream> {
    let type_ident = ident(type_name);
    let properties = collect_properties(type_def, full_defs)?;
    let serializings = properties
        .into_iter()
        .map(|(name, def)| gen_serialize_stmt(quote!(serializer), name, def))
        .collect::<TokenStream>();
    Ok(quote! {
        const _: () = {
            #[allow(unused_mut)]
            impl serde::Serialize for #type_ident {
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

fn aux_conatiner_name(name: &str) -> String {
    format!("__container_{name}")
}

fn gen_label_deserialize_helper(map: HashMap<String, String>) -> TokenStream {
    let labels = map
        .values()
        .collect::<HashSet<_>>()
        .into_iter()
        .map(|v| {
            let ident = ident(v);
            quote!(#ident,)
        })
        .collect::<TokenStream>();

    let label_arms_str = map
        .iter()
        .map(|(k, v)| {
            let k = LitStr::new(k, Span::call_site());
            let v = ident(v);
            quote!(#k => Ok(__Label::#v))
        })
        .collect::<TokenStream>();

    let label_arms_bytes = map
        .iter()
        .map(|(k, v)| {
            let k = LitByteStr::new(k.as_bytes(), Span::call_site());
            let v = ident(v);
            quote!(#k => Ok(__Label::#v))
        })
        .collect::<TokenStream>();

    quote! {
        #[allow(non_camel_case_types)]
        enum __Label { #labels, __Ignore(String) }

        struct __LabelVisitor;

        impl<'de> ::serde::de::Visitor<'de> for __LabelVisitor {
            type Value = __Label;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("field identifier")
            }

            fn visit_str<E: serde::de::Error>(
                self,
                value: &str
            ) -> Result<Self::Value, E> {
                match value {
                    #label_arms_str
                    value => Ok(__Fields::__Ignore(value.to_owned())),
                }
            }

            fn visit_bytes<E: serde::de::Error>(
                self,
                value: &[u8]
            ) -> Result<Self::Value, E> {
                match value {
                    #label_arms_bytes
                    value => Ok(__Fields::__Ignore(String::from_utf8_lossy(value.to_vec()).to_string()))
                }
            }
        }
    }
}

fn gen_label_deserialize_helper_for_struct(
    properties: &HashMap<String, PropertyDef>,
) -> TokenStream {
    gen_label_deserialize_helper(
        properties
            .iter()
            .flat_map(|(name, def)| match def {
                PropertyDef::Simple { tag, aka, .. } => {
                    let tag = tag.as_ref().unwrap_or(name);
                    aka.iter()
                        .chain(std::iter::once(tag))
                        .map(|tag| (tag.to_owned(), name.to_owned()))
                        .collect::<Vec<_>>()
                }
                PropertyDef::LangContainer {
                    tag,
                    container_tag,
                    aka,
                    container_aka,
                    ..
                } => {
                    let tag = tag.as_ref().unwrap_or(name);
                    let default = aka
                        .iter()
                        .chain(std::iter::once(tag))
                        .map(|tag| (tag.to_owned(), name.to_owned()));
                    let per_lang = container_aka
                        .iter()
                        .chain(std::iter::once(container_tag))
                        .map(|tag| (tag.to_owned(), aux_conatiner_name(name)));
                    default.chain(per_lang).collect::<Vec<_>>()
                }
            })
            .collect(),
    )
}

fn gen_field_placeholder_for_struct(name: &str, def: &PropertyDef) -> anyhow::Result<TokenStream> {
    let name_ident = ident(name);
    match def {
        PropertyDef::Simple { .. } => Ok(quote! {
            let mut #name_ident = None;
        }),
        PropertyDef::LangContainer { .. } => {
            let per_lang_ident = ident(&aux_conatiner_name(name));
            Ok(quote! {
                let mut #name_ident = None;
                let mut #per_lang_ident = None;
            })
        }
    }
}

fn gen_insert_deserialized_field(
    name: Ident,
    ty: syn::Type,
    err_label: &str,
    kind: &PropertyKind,
) -> TokenStream {
    if kind == &PropertyKind::Normal {
        quote! {
            __Label::#name => {
                let value = __map.next_value::<#ty>()?;
                if let Some(occupied) = #name.as_mut() {
                    ::activity_vocabulary_core::Mergeable::merge(occupied, value);
                }
                else {
                    #name = Some(value);
                }
            }
        }
    } else {
        quote! {
            __Label::#name => {
                let value = __map.next_value::<#ty>()?;
                if #name.is_some() {
                    return Err(::serde::de::Error::duplicated(#err_label))
                }
                else {
                    #name = Some(value);
                }
            }
        }
    }
}

fn gen_deserialize_match_arm_for_struct(
    name: &str,
    def: &PropertyDef,
) -> anyhow::Result<TokenStream> {
    match def {
        PropertyDef::Simple {
            property_type,
            kind,
            ..
        } => {
            let ty = syn::parse_str(&property_type)?;
            let ty = kind.wrap_type(ty);
            Ok(gen_insert_deserialized_field(ident(name), ty, name, kind))
        }
        PropertyDef::LangContainer {
            property_type,
            kind,
            ..
        } => {
            let ty = syn::parse_str(&property_type)?;
            let ty = kind.wrap_type(ty);
            let default = gen_insert_deserialized_field(ident(name), ty.clone(), name, kind);
            let per_lang = gen_insert_deserialized_field(
                ident(&aux_conatiner_name(name)),
                syn::parse2(quote!(HashMap<String, #ty>)).unwrap(),
                name,
                kind,
            );
            Ok(quote! {#default #per_lang})
        }
    }
}

fn gen_build_field(name: &str, def: &PropertyDef) -> anyhow::Result<TokenStream> {
    let name_ident = ident(name);
    match def {
        PropertyDef::Simple { kind, .. } => {
            if kind == &PropertyKind::Required {
                Ok(quote! {
                    #name_ident: #name_ident.ok_or_else(|| serde::de::Error::missing_field(#name))?,
                })
            } else {
                Ok(quote! {
                    #name_ident: #name_ident.unwrap_or_default(),
                })
            }
        }
        PropertyDef::LangContainer {
            container_tag,
            kind,
            ..
        } => {
            let per_lang_ident = ident(&aux_conatiner_name(container_tag));
            if kind == &PropertyKind::Required {
                Ok(quote! {
                    #name_ident: {
                        if #name_ident.is_none() && #per_lang_ident.is_empty() {
                            return Err(::serde::de::Error::missing_field(#name));
                        }
                        else {
                            LangContainer {
                                default: #name_ident.unwrap_or_default(),
                                per_lang: #per_lang_ident.unwrap_or_default(),
                            }
                        }
                    },
                })
            } else {
                Ok(quote! {
                    #name_ident: LangContainer {
                        default: #name_ident.unwrap_or_default(),
                        per_lang: #per_lang_ident.unwrap_or_default(),
                    },
                })
            }
        }
    }
}

fn gen_impl_visitor_for_struct(
    type_name: &str,
    properties: &HashMap<String, PropertyDef>,
) -> anyhow::Result<TokenStream> {
    let type_ident = ident(type_name);
    let field_placeholders = properties
        .iter()
        .map(|(name, def)| {
            let placeholder = gen_field_placeholder_for_struct(name, def)?;
            Ok(quote!(#placeholder,))
        })
        .collect::<anyhow::Result<TokenStream>>()?;
    let deserialize_match_arms = properties
        .iter()
        .map(|(name, def)| {
            let arm = gen_deserialize_match_arm_for_struct(name, def)?;
            Ok(quote!(#arm,))
        })
        .collect::<anyhow::Result<TokenStream>>()?;
    let build_struct = properties
        .iter()
        .map(|(name, def)| {
            let build = gen_build_field(name, def)?;
            Ok(quote!(#build,))
        })
        .collect::<anyhow::Result<TokenStream>>()?;
    Ok(quote! {
        struct __Visitor;
        impl<'de> ::serde::de::Visitor<'de> for __Visitor {
            type Value = #type_ident;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("field identifier")
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
                Ok(Self::Value { #build_struct })
            }
        }
    })
}

fn gen_deserialize_impl(
    type_name: &str,
    type_def: &TypeDef,
    full_defs: &HashMap<String, TypeDef>,
) -> anyhow::Result<TokenStream> {
    let properties = collect_properties(type_def, full_defs)?;
    let type_ident = ident(type_name);
    let struct_key_strs = properties
        .keys()
        .map(|k| quote!(#k,))
        .collect::<TokenStream>();

    let label_helper = gen_label_deserialize_helper_for_struct(&properties);
    let visitor = gen_impl_visitor_for_struct(type_name, &properties)?;

    Ok(quote! {
        const _: () = {
            impl<'de> ::serde::Deserialize<'de> for #type_ident {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    const FIELDS: &[&str] = [ #struct_key_strs ];

                    #label_helper
                    #visitor

                    deserializer.deserialize_struct(#type_name, FIELDS, __Visitor)
                }
            }
        };
    })
}

fn collect_subtypes<'a>(
    type_name: &'a str,
    type_def: &'a TypeDef,
    full_defs: &'a HashMap<String, TypeDef>,
) -> anyhow::Result<HashMap<&'a str, &'a TypeDef>> {
    let mut names = vec![(type_name, type_def)];
    let mut subtypes = HashMap::new();
    while let Some((name, def)) = names.pop() {
        subtypes.insert(name, def);
        for (sub_name, sub_def) in full_defs {
            if sub_def.extends.contains(name) && !subtypes.contains_key(sub_name.as_str()) {
                names.push((sub_name, sub_def));
            }
        }
    }
    Ok(subtypes)
}

fn gen_upcasts(
    type_name: &str,
    type_def: &TypeDef,
    full_defs: &HashMap<String, TypeDef>,
) -> TokenStream {
    unimplemented!()
}

fn gen_subtypes(
    type_name: &str,
    type_def: &TypeDef,
    full_defs: &HashMap<String, TypeDef>,
) -> anyhow::Result<TokenStream> {
    let subtypes = collect_subtypes(type_name, type_def, full_defs)?;
    let contents = subtypes
        .keys()
        .map(|name| {
            let ident = ident(name);
            quote!(#ident(#ident),)
        })
        .collect::<TokenStream>();
    let ident = ident(&format!("{type_name}Subtypes"));
    Ok(quote! {
        #[derive(Debug, PartialEq, Clone, ::serde::Serialize)]
        pub enum #ident {
            #contents
        }
    })
}

fn gen_subtype_upcast(
    type_name: &str,
    type_def: &TypeDef,
    full_defs: &HashMap<String, TypeDef>,
) -> TokenStream {
    quote! {}
}

fn gen_subtypes_upcasts(
    type_name: &str,
    type_def: &TypeDef,
    full_defs: &HashMap<String, TypeDef>,
) -> TokenStream {
    quote! {}
}

fn gen_subtypes_deserialize(
    type_name: &str,
    type_def: &TypeDef,
    full_defs: &HashMap<String, TypeDef>,
) -> anyhow::Result<TokenStream> {
    let subtype_ident = ident(&format!("{type_name}Subtypes"));
    let subtypes = collect_subtypes(type_name, type_def, full_defs)?;
    let tag_strs = subtypes
        .keys()
        .map(|tag| quote!(#tag,))
        .collect::<TokenStream>();
    let label_helper = gen_label_deserialize_helper(
        subtypes
            .keys()
            .map(|tag| (tag.to_string(), tag.to_string()))
            .collect(),
    );
    let arms = subtypes
        .keys()
        .map(|name| {
            let ident = ident(name);
            quote! { __Label::#ident => Ok(Self::#ident(#ident::deserialize(deserializer))), }
        })
        .collect::<TokenStream>();

    Ok(quote! {
        const _:() = {
            impl<'de> ::serde::de::Deserialize<'de> for #subtype_ident {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    const FIELDS: &[&str] = [ #tag_strs ];

                    #label_helper
                    struct __Visitor;

                    impl<'de> ::serde::de::Visitor<'de> for __Visitor {
                        type Value = #subtype_ident;

                        fn visit_map<A>(
                            self,
                            mut __map: A,
                        ) -> Result<Self::Value, A::Error>
                            where
                                A: serde::de::MapAccess<'de>,
                        {
                            let (tag, container) = deserializer.deserialize_any(
                                ::activity_vocabulary_core::TaggedContentVisitor::<__Label>::new(#type_name, "type")
                            )?;
                            let deserializer = ::serde_value::ValueDeserializer::new(content);
                            match tag {
                                #arms
                                _ => unreachable!()
                            }
                        }
                        deserializer.deserialize_struct(#type_name, FIELDS, __Visitor)
                    }
                }
            }
        };
    })
}

fn gen_set(
    name: &str,
    def: &TypeDef,
    defs: &HashMap<String, TypeDef>,
) -> anyhow::Result<TokenStream> {
    let type_def = gen_type(name, def, defs)?;
    let serialize_impl = gen_serialize_impl(name, def, defs)?;
    let deserialize_impl = gen_deserialize_impl(name, def, defs)?;
    let subtypes_def = gen_subtypes(name, def, defs)?;
    let subtypes_deserialize_impl = gen_subtypes_deserialize(name, def, defs)?;
    let upcasts = gen_upcasts(name, def, defs);
    let subtypes_upcasts = gen_subtypes_upcasts(name, def, defs);
    Ok(quote! {
        #type_def
        #serialize_impl
        #deserialize_impl
        #subtypes_def
        #subtypes_deserialize_impl
        #upcasts
        #subtypes_upcasts
    })
}

pub fn gen(defs: &HashMap<String, TypeDef>) -> anyhow::Result<String> {
    let src = defs
        .iter()
        .map(|(name, def)| gen_set(name, def, defs))
        .collect::<anyhow::Result<TokenStream>>()?;
    let src = RustFmt::new().format_tokens(src)?;
    Ok(src)
}
