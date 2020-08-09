use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct IgniteTypeDefinition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generic_args: Vec<String>,
    pub variant: IgniteTypeVariant,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, IgniteAttribMeta>,
}

impl IgniteTypeDefinition {
    pub fn name(&self) -> String {
        self.variant.name()
    }

    pub fn referenced(&self) -> HashSet<String> {
        self.variant.referenced()
    }
}

impl Hash for IgniteTypeDefinition {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.namespace.hash(state);
        self.generic_args.hash(state);
        self.variant.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub enum IgniteTypeVariant {
    StructUnit(String),
    StructNamed(IgniteNamed),
    StructUnnamed(IgniteUnnamed),
    Enum(IgniteEnum),
}

impl IgniteTypeVariant {
    pub fn name(&self) -> String {
        match self {
            Self::StructUnit(name) => name.clone(),
            Self::StructNamed(value) => value.name.clone(),
            Self::StructUnnamed(value) => value.name.clone(),
            Self::Enum(value) => value.name.clone(),
        }
    }

    pub fn referenced(&self) -> HashSet<String> {
        match self {
            Self::StructUnit(_) => HashSet::new(),
            Self::StructNamed(value) => value.referenced(),
            Self::StructUnnamed(value) => value.referenced(),
            Self::Enum(value) => value.referenced(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct IgniteNamed {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<IgniteNamedField>,
}

impl IgniteNamed {
    pub fn referenced(&self) -> HashSet<String> {
        self.fields
            .iter()
            .flat_map(|field| field.referenced())
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct IgniteUnnamed {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<IgniteUnnamedField>,
}

impl IgniteUnnamed {
    pub fn referenced(&self) -> HashSet<String> {
        self.fields
            .iter()
            .flat_map(|field| field.referenced())
            .collect()
    }
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

impl IgniteNamedField {
    pub fn referenced(&self) -> HashSet<String> {
        if let Some(mapping) = &self.mapping {
            let mut result = HashSet::new();
            if let Some(mapping) = mapping.split('.').last() {
                result.insert(mapping.to_owned());
            }
            result
        } else {
            self.typename.referenced()
        }
    }
}

impl Hash for IgniteNamedField {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name.hash(state);
        self.typename.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgniteUnnamedField {
    pub typename: IgniteType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mapping: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, IgniteAttribMeta>,
}

impl IgniteUnnamedField {
    pub fn referenced(&self) -> HashSet<String> {
        if let Some(mapping) = &self.mapping {
            let mut result = HashSet::new();
            if let Some(mapping) = mapping.split('.').last() {
                result.insert(mapping.to_owned());
            }
            result
        } else {
            self.typename.referenced()
        }
    }
}

impl Hash for IgniteUnnamedField {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.typename.hash(state);
    }
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct IgniteEnum {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<IgniteVariant>,
}

impl IgniteEnum {
    pub fn referenced(&self) -> HashSet<String> {
        self.variants
            .iter()
            .flat_map(|variant| variant.referenced())
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub enum IgniteVariant {
    Unit(String),
    Named(IgniteNamed),
    Unnamed(IgniteUnnamed),
}

impl IgniteVariant {
    pub fn referenced(&self) -> HashSet<String> {
        match self {
            Self::Unit(name) => {
                let mut result = HashSet::new();
                result.insert(name.clone());
                result
            }
            Self::Named(value) => value.referenced(),
            Self::Unnamed(value) => value.referenced(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub enum IgniteType {
    Unit,
    Atom(String),
    Tuple(Vec<IgniteType>),
    Array(IgniteTypeArray),
    Generic(IgniteTypeGeneric),
}

impl IgniteType {
    pub fn referenced(&self) -> HashSet<String> {
        match self {
            Self::Unit => HashSet::new(),
            Self::Atom(name) => {
                let mut result = HashSet::new();
                result.insert(name.clone());
                result
            }
            Self::Tuple(value) => value.iter().flat_map(|item| item.referenced()).collect(),
            Self::Array(value) => value.referenced(),
            Self::Generic(value) => value.referenced(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct IgniteTypeArray {
    pub typename: Box<IgniteType>,
    pub size: usize,
}

impl IgniteTypeArray {
    pub fn referenced(&self) -> HashSet<String> {
        self.typename.referenced()
    }
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct IgniteTypeGeneric {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<IgniteType>,
}

impl IgniteTypeGeneric {
    pub fn referenced(&self) -> HashSet<String> {
        std::iter::once(self.name.clone())
            .chain(self.arguments.iter().flat_map(|arg| arg.referenced()))
            .collect()
    }
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
