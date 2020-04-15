extern crate proc_macro;

use proc_macro::TokenStream;
use quote::ToTokens;
use serde::Serialize;
use syn::{
    parse_macro_input, Data, DeriveInput, Expr, Fields, GenericArgument, Lit, PathArguments, Type,
};

// TODO: add static wrapper struct instance that will aggregate processed types
// and save them to file on `drop()`.

#[derive(Debug, Serialize)]
struct IgniteTypeDefinition {
    pub namespace: Option<String>,
    pub variant: IgniteTypeVariant,
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
    pub fields: Vec<(String, IgniteType)>,
}

#[derive(Debug, Serialize)]
struct IgniteUnnamed {
    pub name: String,
    pub fields: Vec<IgniteType>,
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
    /// (name)
    Atom(String),
    /// (types)
    Tuple(Vec<IgniteType>),
    /// (type, size)
    Array(Box<IgniteType>, usize),
    /// (name, types)
    Generic(String, Vec<IgniteType>),
}

#[proc_macro_derive(Ignite, attributes(ignite))]
pub fn derive_ignite(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = ast.ident.to_string();
    let variant = match ast.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => {
                let fields = fields
                    .named
                    .iter()
                    .map(|field| {
                        let name = field.ident.as_ref().unwrap().to_string();
                        let type_ = parse_type(&field.ty);
                        (name, type_)
                    })
                    .collect::<Vec<_>>();
                IgniteTypeVariant::StructNamed(IgniteNamed { name, fields })
            }
            Fields::Unnamed(fields) => {
                let fields = fields
                    .unnamed
                    .iter()
                    .map(|field| parse_type(&field.ty))
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
                                .map(|field| {
                                    let name = field.ident.as_ref().unwrap().to_string();
                                    let type_ = parse_type(&field.ty);
                                    (name, type_)
                                })
                                .collect::<Vec<_>>();
                            IgniteVariant::Named(IgniteNamed { name, fields })
                        }
                        Fields::Unnamed(fields) => {
                            let fields = fields
                                .unnamed
                                .iter()
                                .map(|field| parse_type(&field.ty))
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
    let namespace = None;
    let result = IgniteTypeDefinition { namespace, variant };
    println!("=== RESULT: {:#?}", result);
    TokenStream::new()
}

fn parse_type(type_: &Type) -> IgniteType {
    match type_ {
        Type::Path(path) => {
            let segment = path.path.segments.last().unwrap();
            let name = segment.ident.to_string();
            match &segment.arguments {
                PathArguments::None => IgniteType::Atom(name),
                PathArguments::AngleBracketed(arguments) => {
                    let args = arguments.args.iter().map(|arg| {
                        match arg {
                            GenericArgument::Type(ty) => parse_type(ty),
                            _ => panic!(
                                "Ignite requires owned types of either unit, atom, tuple, array or generic. This type does not parse: {}",
                                type_.to_token_stream(),
                            ),
                        }
                    }).collect::<Vec<_>>();
                    IgniteType::Generic(name, args)
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
            let ty = parse_type(&array.elem);
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
            IgniteType::Array(Box::new(ty), size)
        }
        _ => panic!(
            "Ignite requires owned types of either unit, atom, tuple, array or generic. This type does not parse: {}",
            type_.to_token_stream(),
        ),
    }
}
