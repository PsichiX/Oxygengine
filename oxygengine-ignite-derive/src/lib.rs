extern crate proc_macro;

use lazy_static::lazy_static;
use oxygengine_ignite_types::*;
use proc_macro::TokenStream;
use quote::ToTokens;
use std::{
    collections::HashMap,
    fs::{create_dir_all, write},
    path::PathBuf,
    sync::{Arc, RwLock},
};
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Expr, Fields, GenericArgument, GenericParam,
    Lit, Meta, NestedMeta, PathArguments, Type,
};

lazy_static! {
    static ref TYPES_DIR: Arc<RwLock<PathBuf>> = {
        let path = std::env::current_dir()
            .unwrap()
            .join("target")
            .join("ignite")
            .join("types");
        if create_dir_all(&path).is_err() {
            println!(
                "Could not create Ignite type definitions directory: {:?}",
                path
            );
        }
        Arc::new(RwLock::new(path))
    };
}

fn store_type(item: &IgniteTypeDefinition) {
    #[cfg(feature = "target-yaml")]
    store_type_yaml(&item);
    #[cfg(feature = "target-json")]
    store_type_json(&item);
    #[cfg(feature = "target-ron")]
    store_type_ron(&item);
    #[cfg(feature = "target-binary")]
    store_type_binary(&item);
}

#[cfg(feature = "target-yaml")]
fn store_type_yaml(item: &IgniteTypeDefinition) {
    let name = match &item.variant {
        IgniteTypeVariant::StructUnit(item) => &item,
        IgniteTypeVariant::StructNamed(item) => &item.name,
        IgniteTypeVariant::StructUnnamed(item) => &item.name,
        IgniteTypeVariant::Enum(item) => &item.name,
    };
    let path = TYPES_DIR
        .read()
        .unwrap()
        .join(format!("{}.{}.ignite-type.yaml", item.namespace, name));
    if let Ok(content) = serde_yaml::to_string(&item) {
        if write(&path, content).is_err() {
            println!("Could not save Ignite type definition to file: {:?}", path);
        }
    } else {
        println!("Could not serialize Ignite type definition: {:?}", path);
    }
}

#[cfg(feature = "target-json")]
fn store_type_json(item: &IgniteTypeDefinition) {
    let name = match &item.variant {
        IgniteTypeVariant::StructUnit(item) => &item,
        IgniteTypeVariant::StructNamed(item) => &item.name,
        IgniteTypeVariant::StructUnnamed(item) => &item.name,
        IgniteTypeVariant::Enum(item) => &item.name,
    };
    let path = TYPES_DIR
        .read()
        .unwrap()
        .join(format!("{}.{}.ignite-type.json", item.namespace, name));
    #[cfg(feature = "pretty")]
    let result = serde_json::to_string_pretty(&item);
    #[cfg(not(feature = "pretty"))]
    let result = serde_json::to_string(&item);
    if let Ok(content) = result {
        if write(&path, content).is_err() {
            println!("Could not save Ignite type definition to file: {:?}", path);
        }
    } else {
        println!("Could not serialize Ignite type definition: {:?}", path);
    }
}

#[cfg(feature = "target-ron")]
fn store_type_ron(item: &IgniteTypeDefinition) {
    let name = match &item.variant {
        IgniteTypeVariant::StructUnit(item) => &item,
        IgniteTypeVariant::StructNamed(item) => &item.name,
        IgniteTypeVariant::StructUnnamed(item) => &item.name,
        IgniteTypeVariant::Enum(item) => &item.name,
    };
    let path = TYPES_DIR
        .read()
        .unwrap()
        .join(format!("{}.{}.ignite-type.ron", item.namespace, name));
    #[cfg(feature = "pretty")]
    let result = ron::ser::to_string_pretty(&item, Default::default());
    #[cfg(not(feature = "pretty"))]
    let result = ron::ser::to_string(&item);
    if let Ok(content) = result {
        if write(&path, content).is_err() {
            println!("Could not save Ignite type definition to file: {:?}", path);
        }
    } else {
        println!("Could not serialize Ignite type definition: {:?}", path);
    }
}

#[cfg(feature = "target-binary")]
fn store_type_binary(item: &IgniteTypeDefinition) {
    let name = match &item.variant {
        IgniteTypeVariant::StructUnit(item) => &item,
        IgniteTypeVariant::StructNamed(item) => &item.name,
        IgniteTypeVariant::StructUnnamed(item) => &item.name,
        IgniteTypeVariant::Enum(item) => &item.name,
    };
    let path = TYPES_DIR
        .read()
        .unwrap()
        .join(format!("{}.{}.ignite-type.bin", item.namespace, name));
    if let Ok(content) = bincode::serialize(&item) {
        if write(&path, content).is_err() {
            println!("Could not save Ignite type definition to file: {:?}", path);
        }
    } else {
        println!("Could not serialize Ignite type definition: {:?}", path);
    }
}

#[derive(Debug, Default)]
struct IgniteFieldAttribs {
    pub ignore: bool,
    pub mapping: Option<String>,
    pub meta: HashMap<String, IgniteAttribMeta>,
}

#[derive(Debug, Default)]
struct IgniteTypeAttribs {
    pub namespace: Option<String>,
    pub meta: HashMap<String, IgniteAttribMeta>,
}

#[proc_macro]
pub fn ignite_proxy(input: TokenStream) -> TokenStream {
    derive_ignite_inner(input, true)
}

#[proc_macro]
pub fn ignite_alias(input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_derive(Ignite, attributes(ignite))]
pub fn derive_ignite(input: TokenStream) -> TokenStream {
    derive_ignite_inner(input, false)
}

fn derive_ignite_inner(input: TokenStream, is_proxy: bool) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let attribs = parse_type_attribs(&ast.attrs);
    let generic_args = ast
        .generics
        .params
        .iter()
        .filter_map(|param| {
            if let GenericParam::Type(param) = param {
                Some(param.ident.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let name = ast.ident.to_string();
    let variant = match ast.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let fields = fields
                    .named
                    .iter()
                    .filter_map(|field| {
                        let attribs = parse_field_attribs(&field.attrs);
                        if attribs.ignore {
                            return None;
                        }
                        let name = field.ident.as_ref().unwrap().to_string();
                        let type_ = parse_type(&field.ty);
                        Some(IgniteNamedField {
                            name,
                            typename: type_,
                            mapping: attribs.mapping,
                            meta: attribs.meta,
                        })
                    })
                    .collect::<Vec<_>>();
                IgniteTypeVariant::StructNamed(IgniteNamed { name, fields })
            }
            Fields::Unnamed(fields) => {
                let fields = fields
                    .unnamed
                    .iter()
                    .filter_map(|field| {
                        let attribs = parse_field_attribs(&field.attrs);
                        if attribs.ignore {
                            return None;
                        }
                        let type_ = parse_type(&field.ty);
                        Some(IgniteUnnamedField {
                            typename: type_,
                            mapping: attribs.mapping,
                            meta: attribs.meta,
                        })
                    })
                    .collect::<Vec<_>>();
                IgniteTypeVariant::StructUnnamed(IgniteUnnamed { name, fields })
            }
            Fields::Unit => IgniteTypeVariant::StructUnit(name),
        },
        Data::Enum(data) => {
            let variants = data
                .variants
                .iter()
                .map(|variant| {
                    let name = variant.ident.to_string();
                    match &variant.fields {
                        Fields::Named(fields) => {
                            let fields = fields
                                .named
                                .iter()
                                .filter_map(|field| {
                                    let attribs = parse_field_attribs(&field.attrs);
                                    if attribs.ignore {
                                        return None;
                                    }
                                    let name = field.ident.as_ref().unwrap().to_string();
                                    let type_ = parse_type(&field.ty);
                                    Some(IgniteNamedField {
                                        name,
                                        typename: type_,
                                        mapping: attribs.mapping,
                                        meta: attribs.meta,
                                    })
                                })
                                .collect::<Vec<_>>();
                            IgniteVariant::Named(IgniteNamed { name, fields })
                        }
                        Fields::Unnamed(fields) => {
                            let fields = fields
                                .unnamed
                                .iter()
                                .filter_map(|field| {
                                    let attribs = parse_field_attribs(&field.attrs);
                                    if attribs.ignore {
                                        return None;
                                    }
                                    let type_ = parse_type(&field.ty);
                                    Some(IgniteUnnamedField {
                                        typename: type_,
                                        mapping: attribs.mapping,
                                        meta: attribs.meta,
                                    })
                                })
                                .collect::<Vec<_>>();
                            IgniteVariant::Unnamed(IgniteUnnamed { name, fields })
                        }
                        Fields::Unit => IgniteVariant::Unit(name),
                    }
                })
                .collect::<Vec<_>>();
            IgniteTypeVariant::Enum(IgniteEnum { name, variants })
        }
        _ => panic!("Ignite can be derived only for structs and enums"),
    };
    let definition = IgniteTypeDefinition {
        namespace: if let Some(namespace) = attribs.namespace {
            namespace
        } else {
            std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_owned())
        },
        generic_args,
        variant,
        meta: attribs.meta,
        is_proxy,
    };
    store_type(&definition);
    TokenStream::new()
}

fn parse_type_attribs(attrs: &[Attribute]) -> IgniteTypeAttribs {
    let mut result = IgniteTypeAttribs::default();
    for attrib in attrs {
        match attrib.parse_meta() {
            Err(error) => panic!(
                "Could not parse ignite attribute `{}`: {:?}",
                attrib.to_token_stream().to_string(),
                error
            ),
            Ok(Meta::List(meta)) => {
                if meta.path.is_ident("ignite") {
                    for meta in meta.nested {
                        if let NestedMeta::Meta(Meta::NameValue(meta)) = &meta {
                            if meta.path.is_ident("namespace") {
                                if let Lit::Str(value) = &meta.lit {
                                    result.namespace = Some(value.value());
                                }
                            } else if let Some(ident) = meta.path.get_ident() {
                                let value = match &meta.lit {
                                    Lit::Str(value) => IgniteAttribMeta::String(value.value()),
                                    Lit::Int(value) => IgniteAttribMeta::Integer(
                                        value.base10_parse::<i64>().unwrap(),
                                    ),
                                    Lit::Float(value) => IgniteAttribMeta::Float(
                                        value.base10_parse::<f64>().unwrap(),
                                    ),
                                    Lit::Bool(value) => IgniteAttribMeta::Bool(value.value),
                                    _ => IgniteAttribMeta::None,
                                };
                                result.meta.insert(ident.to_string(), value);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    result
}

fn parse_field_attribs(attrs: &[Attribute]) -> IgniteFieldAttribs {
    let mut result = IgniteFieldAttribs::default();
    for attrib in attrs {
        match attrib.parse_meta() {
            Err(error) => panic!(
                "Could not parse ignite attribute `{}`: {:?}",
                attrib.to_token_stream().to_string(),
                error
            ),
            Ok(Meta::List(meta)) => {
                if meta.path.is_ident("ignite") {
                    for meta in meta.nested {
                        if let NestedMeta::Meta(meta) = &meta {
                            if let Meta::Path(path) = meta {
                                if path.is_ident("ignore") {
                                    result.ignore = true;
                                } else if let Some(ident) = path.get_ident() {
                                    result
                                        .meta
                                        .insert(ident.to_string(), IgniteAttribMeta::None);
                                }
                            } else if let Meta::NameValue(meta) = meta {
                                if meta.path.is_ident("mapping") {
                                    if let Lit::Str(value) = &meta.lit {
                                        result.mapping = Some(value.value());
                                    }
                                } else if let Some(ident) = meta.path.get_ident() {
                                    let value = match &meta.lit {
                                        Lit::Str(value) => IgniteAttribMeta::String(value.value()),
                                        Lit::Int(value) => IgniteAttribMeta::Integer(
                                            value.base10_parse::<i64>().unwrap(),
                                        ),
                                        Lit::Float(value) => IgniteAttribMeta::Float(
                                            value.base10_parse::<f64>().unwrap(),
                                        ),
                                        Lit::Bool(value) => IgniteAttribMeta::Bool(value.value),
                                        _ => IgniteAttribMeta::None,
                                    };
                                    result.meta.insert(ident.to_string(), value);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    result
}

fn parse_type(type_: &Type) -> IgniteType {
    match type_ {
        Type::Path(path) => {
            let segment = path.path.segments.last().unwrap();
            let name = segment.ident.to_string();
            match &segment.arguments {
                PathArguments::None => IgniteType::Atom(name),
                PathArguments::AngleBracketed(arguments) => {
                    let arguments = arguments.args.iter().filter_map(|arg| {
                        match arg {
                            GenericArgument::Type(ty) => Some(parse_type(ty)),
                            _ => None,
                        }
                    }).collect::<Vec<_>>();
                    IgniteType::Generic(IgniteTypeGeneric{name, arguments})
                }
                _ => panic!(
                    "Ignite requires owned types of either unit, atom, tuple, array or generic. This type does not parse: {}",
                    type_.to_token_stream(),
                ),
            }
        }
        Type::Tuple(tuple) => {
            if tuple.elems.is_empty() {
                IgniteType::Unit
            } else {
                let elems = tuple.elems.iter().map(|elem| parse_type(elem)).collect::<Vec<_>>();
                IgniteType::Tuple(elems)
            }
        }
        Type::Array(array) => {
            let typename = Box::new(parse_type(&array.elem));
            let size = match &array.len {
                Expr::Lit(lit) => {
                    match &lit.lit {
                        Lit::Int(lit) => lit.base10_parse::<usize>().unwrap(),
                        _ => panic!(
                            "Ignite requires owned types of either unit, atom, tuple, array or generic. This type does not parse: {}",
                            type_.to_token_stream(),
                        ),
                    }
                }
                _ => panic!(
                    "Ignite requires owned types of either unit, atom, tuple, array or generic. This type does not parse: {}",
                    type_.to_token_stream(),
                ),
            };
            IgniteType::Array(IgniteTypeArray{typename,size})
        }
        _ => panic!(
            "Ignite requires owned types of either unit, atom, tuple, array or generic. This type does not parse: {}",
            type_.to_token_stream(),
        ),
    }
}
