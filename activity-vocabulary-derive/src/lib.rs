use std::{collections::HashMap, fs, path::Path};

use anyhow::Context;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use rust_format::{Formatter, RustFmt};
use serde::{Deserialize, Serialize};
use syn::LitByteStr;

const W3C_DOC_BASE: &str = "https://www.w3.org/TR/activitystreams-vocabulary";

fn doc_link(name: &str) -> String {
    format!("{W3C_DOC_BASE}/#dfn-{}", name.to_lowercase())
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PropertyDef {
    #[serde(rename = "type")]
    property_type: String,
    rename: Option<String>,
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
        if let Some(container) = &self.container {
            match container.container_type {
                ContainerType::Language => Ok(PropertyType::LanguageContainer {
                    ty: quote!(LangContainer<#ty>),
                    default: quote!(Option<#ty>),
                    per_lang: quote!(std::collections::HashMap<String, #ty>),
                }),
            }
        } else {
            let ty = match self.kind {
                PropertyKind::Functional => quote!(Option<#ty>),
                PropertyKind::Normal => quote!(Property<#ty>),
                PropertyKind::Required => ty,
            };
            Ok(PropertyType::Simple(ty))
        }
    }

    fn tag(&self, name: &str) -> String {
        self.rename.clone().unwrap_or_else(|| name.to_owned())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct TypeDef {
    extends: Vec<String>,
    properties: HashMap<String, PropertyDef>,
    #[serde(default = "Default::default")]
    as_object_id: bool,
}

fn collect_properties(
    name: &str,
    defs: &HashMap<String, TypeDef>,
) -> anyhow::Result<Vec<(String, PropertyDef)>> {
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
        .collect::<Vec<_>>();
    properties.append(
        &mut def
            .properties
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>(),
    );
    Ok(properties)
}

fn generate_properties(properties: &[(String, PropertyDef)]) -> anyhow::Result<TokenStream> {
    let tokens = properties
        .iter()
        .map(|(name, def)| {
            let quoted_name = Ident::new(name, Span::call_site());
            let quoted_type = match def.property_type()? {
                PropertyType::LanguageContainer { ty, .. } => ty,
                PropertyType::Simple(ty) => ty,
            };
            let tag = def.tag(name);
            let doc_comment = format!(
                "`{}`\n\n[W3C recommendation]({})\n]n{}",
                def.uri,
                doc_link(&tag),
                def.doc
            );
            let quoted = quote! {
                #[doc = #doc_comment]
                #quoted_name: #quoted_type,
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
    properties: &[(String, PropertyDef)],
) -> anyhow::Result<TokenStream> {
    let name_ident = Ident::new(name, Span::call_site());
    let serializings = properties
        .iter()
        .flat_map(|(name, def)| {
            let name_ident = Ident::new(name, Span::call_site());
            let tag = def.tag(name);
            if let Some(container) = &def.container {
                match container.container_type {
                    ContainerType::Language => {
                        let map_name = &container.name;
                        quote! {
                            if self.#name_ident.default.is_some() {
                                serializer.serialize_entry(#tag, &self.#name_ident.default)?;
                            }
                            if !self.#name_ident.per_lang.is_empty() {
                                serializer.serialize_entry(#map_name, &self.#name_ident.per_lang)?;
                            }
                        }
                    }
                }
            } else {
                quote! { serializer.serialize_entry(#tag, &self.#name_ident)?; }
            }
        })
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
    properties: &[(String, PropertyDef)],
) -> anyhow::Result<TokenStream> {
    let name_ident = Ident::new(name, Span::call_site());

    let field_tags = properties
        .iter()
        .map(|(name, def)| {
            let tag = def.tag(name);
            if let Some(container) = &def.container {
                match container.container_type {
                    ContainerType::Language => {
                        let map_tag = &container.name;
                        quote! {#tag, #map_tag, }
                    }
                }
            } else {
                quote! {#tag, }
            }
        })
        .collect::<TokenStream>();

    let fields_enum_defs = properties
        .iter()
        .map(|(name, def)| {
            let ident = Ident::new(name, Span::call_site());
            if let Some(container) = &def.container {
                match container.container_type {
                    ContainerType::Language => {
                        let map_ident = Ident::new(
                            &format!("_prop_map_{}", &container.name),
                            Span::call_site(),
                        );
                        quote!(#ident, #map_ident,)
                    }
                }
            } else {
                quote!(#ident,)
            }
        })
        .collect::<TokenStream>();

    let field_enum_match_arms = |bytes: bool| {
        properties
        .iter()
        .map(|(name, def)| {
            let tag = def.tag(name);
            let tag = if bytes {
                LitByteStr::new(tag.as_bytes(), Span::call_site()).into_token_stream()
            } else {
                tag.into_token_stream()
            };
            let name = Ident::new(name, Span::call_site());
            if let Some(container) = &def.container {
                let map_ident =
                    Ident::new(&format!("_prop_map_{}", &container.name), Span::call_site());

                let map_tag = if bytes {
                    LitByteStr::new(container.name.as_bytes(), Span::call_site()).into_token_stream()
                } else {
                    container.name.clone().into_token_stream()
                };
                match container.container_type {
                    ContainerType::Language => {
                        quote!(#tag => Ok(__Fields::#name), #map_tag => Ok(__Fields::#map_ident),)
                    }
                }
            } else {
                quote!(#tag => Ok(__Fields::#name),)
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
                        let name_map =
                            Ident::new(&format!("_prop_map_{}", container.name), Span::call_site());
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
                    unreachable!()
                };
                let map_name = format!("_prop_map_{}", container.name);
                let map_name_ident = Ident::new(&map_name, Span::call_site());
                Ok(quote!{
                    __Fields::#name_ident => {
                        if #name_ident.is_some() {
                            return Err(<A::Error as serde::de::Error>::duplicate_field(#name));
                        }
                        #name_ident = Some(__map.next_value::<#default>()?);
                    }
                    __Fields::#map_name_ident => {
                        if #map_name_ident.is_some() {
                            return Err(<A::Error as serde::de::Error>::duplicate_field(#map_name));
                        }
                        #map_name_ident = Some(__map.next_value::<#per_lang>()?);
                    }
                })
            } else {
                let PropertyType::Simple(ty) = def.property_type()? else {
                    unreachable!()
                };
                Ok(quote!(
                    __Fields::#name_ident => {
                        if #name_ident.is_some() {
                            return Err(<A::Error as serde::de::Error>::duplicate_field(#name));
                        }
                        #name_ident = Some(__map.next_value::<#ty>()?);
                    }
                ))
            }
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<TokenStream>();

    let build_type = properties
        .iter()
        .map(|(name, def)| {
            let name_ident = Ident::new(name, Span::call_site());
            let not_found_msg = format!("{} not found", name);
            if let Some(container) = &def.container {
                let map_name = format!("_prop_map_{}", container.name);
                let map_name_ident = Ident::new(&map_name, Span::call_site());
                Ok(quote! {
                    #name_ident: LangContainer {
                        default: #name_ident.ok_or_else(|| <A::Error as serde::de::Error>::missing_field(#not_found_msg))?,
                        per_lang: #map_name_ident.ok_or_else(|| <A::Error as serde::de::Error>::missing_field(#not_found_msg))?,
                    },
                })
            } else {
                Ok(quote! {
                    #name_ident: #name_ident.ok_or_else(|| <A::Error as serde::de::Error>::missing_field(#not_found_msg))?,
                })
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

                    deserializer.deserialize_map(__Visitor)
                }
            }
        };
    })
}

fn generate_types(defs: HashMap<String, TypeDef>) -> anyhow::Result<TokenStream> {
    let mut token = TokenStream::new();
    for name in defs.keys() {
        let properties = collect_properties(name, &defs)?;
        let quote_properties = generate_properties(&properties)?;
        let quote_name = Ident::new(name, Span::call_site());
        token.append_all(quote! {
            pub struct #quote_name {
                #quote_properties
            }
        });
        token.append_all(generate_serialize_impl(name, &properties)?);
        token.append_all(generate_deserialize_impl(name, &properties)?);
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
