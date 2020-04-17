use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct IgniteTypeDefinition {
    pub namespace: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generic_args: Vec<String>,
    pub variant: IgniteTypeVariant,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, IgniteAttribMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IgniteTypeVariant {
    StructUnit(String),
    StructNamed(IgniteNamed),
    StructUnnamed(IgniteUnnamed),
    Enum(IgniteEnum),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgniteNamed {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<IgniteNamedField>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgniteUnnamed {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<IgniteUnnamedField>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgniteNamedField {
    pub name: String,
    pub typename: IgniteType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mapping: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, IgniteAttribMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgniteUnnamedField {
    pub typename: IgniteType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mapping: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, IgniteAttribMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgniteEnum {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<IgniteVariant>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IgniteVariant {
    Unit(String),
    Named(IgniteNamed),
    Unnamed(IgniteUnnamed),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IgniteType {
    Unit,
    Atom(String),
    Tuple(Vec<IgniteType>),
    Array(IgniteTypeArray),
    Generic(IgniteTypeGeneric),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgniteTypeArray {
    pub typename: Box<IgniteType>,
    pub size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgniteTypeGeneric {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<IgniteType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IgniteAttribMeta {
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

pub trait Ignite {
    fn generate_type_definition() -> IgniteTypeDefinition;
}
