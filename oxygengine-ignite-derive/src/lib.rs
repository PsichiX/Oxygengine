extern crate proc_macro;

use lazy_static::lazy_static;
use proc_macro::TokenStream;
use quote::ToTokens;
use serde::Serialize;
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

fn store_type(item: IgniteTypeDefinition) {
    let name = match &item.variant {
        IgniteTypeVariant::StructUnit(item) => &item,
        IgniteTypeVariant::StructNamed(item) => &item.name,
        IgniteTypeVariant::StructUnnamed(item) => &item.name,
        IgniteTypeVariant::Enum(item) => &item.name,
    };
    let path = if let Some(namespace) = &item.namespace {
        TYPES_DIR
            .read()
            .unwrap()
            .join(format!("{}.{}.ignite-type.yaml", namespace, name))
    } else {
        TYPES_DIR
            .read()
            .unwrap()
            .join(format!("{}.ignite-type.yaml", name))
    };
    if let Ok(content) = serde_yaml::to_string(&item) {
        if write(&path, content).is_err() {
            println!("Could not save Ignite type definition to file: {:?}", path);
        }
    } else {
        println!("Could not serialize Ignite type definition: {:?}", path);
    }
}

#[derive(Debug, Serialize)]
struct IgniteTypeDefinition {
    #[serde(default)]
    pub namespace: Option<String>,
    pub generic_args: Vec<String>,
    pub variant: IgniteTypeVariant,
    pub meta: HashMap<String, IgniteAttribMeta>,
}

#[derive(Debug, Serialize)]
enum IgniteTypeVariant {
    StructUnit(String),
    StructNamed(IgniteNamed),
    StructUnnamed(IgniteUnnamed),
    Enum(IgniteEnum),
}

#[derive(Debug, Serialize)]
struct IgniteNamed {
    pub name: String,
    pub fields: Vec<IgniteNamedField>,
}

#[derive(Debug, Serialize)]
struct IgniteUnnamed {
    pub name: String,
    pub fields: Vec<IgniteUnnamedField>,
}

#[derive(Debug, Serialize)]
struct IgniteNamedField {
    pub name: String,
    pub typename: IgniteType,
    pub mapping: Option<String>,
    pub meta: HashMap<String, IgniteAttribMeta>,
}

#[derive(Debug, Serialize)]
struct IgniteUnnamedField {
    pub typename: IgniteType,
    pub mapping: Option<String>,
    pub meta: HashMap<String, IgniteAttribMeta>,
}

#[derive(Debug, Serialize)]
struct IgniteEnum {
    pub name: String,
    pub variants: Vec<IgniteVariant>,
}

#[derive(Debug, Serialize)]
enum IgniteVariant {
    Unit(String),
    Named(IgniteNamed),
    Unnamed(IgniteUnnamed),
}

#[derive(Debug, Serialize)]
enum IgniteType {
    Unit,
    Atom(String),
    Tuple(Vec<IgniteType>),
    Array(IgniteTypeArray),
    Generic(IgniteTypeGeneric),
}

#[derive(Debug, Serialize)]
struct IgniteTypeArray {
    pub typename: Box<IgniteType>,
    pub size: usize,
}

#[derive(Debug, Serialize)]
struct IgniteTypeGeneric {
    pub name: String,
    pub arguments: Vec<IgniteType>,
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

#[derive(Debug, Serialize)]
enum IgniteAttribMeta {
    None,
    Bool(bool),
    String(String),
    Integer(i64),
    Float(f64),
}

impl Default for IgniteAttribMeta {
    fn default() -> Self {
        Self::None
    }
}

#[proc_macro_derive(Ignite, attributes(ignite))]
pub fn derive_ignite(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let attribs = parse_type_attribs(&ast.attrs);
    let generic_args = ast
        .generics
        .params
        .iter()
        .map(|param| {
            if let GenericParam::Type(param) = param {
                param.ident.to_string()
            } else {
                unreachable!()
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
    store_type(IgniteTypeDefinition {
        namespace: attribs.namespace,
        generic_args,
        variant,
        meta: attribs.meta,
    });
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
