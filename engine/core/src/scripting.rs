use crate::{
    app::AppBuilder,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
    prefab::PrefabValue,
};
use intuicio_essentials::{
    core::{
        function::FunctionQuery,
        registry::Registry,
        struct_type::{NativeStructBuilder, StructQuery},
    },
    data::managed::{DynamicManaged, DynamicManagedRef, DynamicManagedRefMut},
};
use pest::Parser;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, sync::RwLock};

pub use intuicio_essentials as intuicio;

pub type ScriptValueFactory =
    dyn FnMut(&PrefabValue) -> Result<DynamicManaged, Box<dyn std::error::Error>> + Send + Sync;

lazy_static! {
    static ref SCRIPT_VALUE_FACTORY: RwLock<HashMap<String, Box<ScriptValueFactory>>> =
        Default::default();
}

#[allow(clippy::upper_case_acronyms)]
mod parser {
    #[derive(Parser)]
    #[grammar = "scripting.pest"]
    pub(super) struct ScriptReferenceParser;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptStructReference {
    #[serde(default)]
    pub module_name: Option<String>,
    pub name: String,
}

impl ScriptStructReference {
    pub fn query(&self) -> StructQuery {
        StructQuery {
            name: Some(self.name.as_str().into()),
            module_name: self.module_name.as_ref().map(|name| name.as_str().into()),
            ..Default::default()
        }
    }

    pub fn parse(content: &str) -> Result<Self, String> {
        match parser::ScriptReferenceParser::parse(parser::Rule::main_struct_reference, content) {
            Ok(mut ast) => {
                let mut ast = ast.next().unwrap().into_inner();
                let mut ast = ast.next().unwrap().into_inner();
                let a = ast.next().unwrap();
                let b = ast.next();
                let (module_name, name) = if let Some(b) = b {
                    (Some(a.as_str().to_owned()), b.as_str().to_owned())
                } else {
                    (None, a.as_str().to_owned())
                };
                Ok(Self { module_name, name })
            }
            Err(error) => Err(format!("{}", error)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptFunctionReference {
    #[serde(default)]
    pub struct_type: Option<ScriptStructReference>,
    #[serde(default)]
    pub module_name: Option<String>,
    pub name: String,
}

impl ScriptFunctionReference {
    pub fn query(&self) -> FunctionQuery {
        FunctionQuery {
            struct_query: self
                .struct_type
                .as_ref()
                .map(|struct_type| struct_type.query()),
            name: Some(self.name.as_str().into()),
            module_name: self.module_name.as_ref().map(|name| name.as_str().into()),
            ..Default::default()
        }
    }

    pub fn parse(content: &str) -> Result<Self, String> {
        match parser::ScriptReferenceParser::parse(parser::Rule::main_function_reference, content) {
            Ok(mut ast) => {
                let mut ast = ast.next().unwrap().into_inner();
                let mut ast = ast.next().unwrap().into_inner();
                let a = ast.next().unwrap();
                let b = ast.next();
                let c = ast.next();
                let (struct_type, module_name, name) =
                    if a.as_rule() == parser::Rule::struct_reference {
                        let b = b.unwrap();
                        let struct_type = ScriptStructReference::parse(a.as_str())?;
                        if let Some(c) = c {
                            (
                                Some(struct_type),
                                Some(b.as_str().to_owned()),
                                c.as_str().to_owned(),
                            )
                        } else {
                            (Some(struct_type), None, b.as_str().to_owned())
                        }
                    } else if let Some(b) = b {
                        (None, Some(a.as_str().to_owned()), b.as_str().to_owned())
                    } else {
                        (None, None, a.as_str().to_owned())
                    };
                Ok(Self {
                    struct_type,
                    module_name,
                    name,
                })
            }
            Err(error) => Err(format!("{}", error)),
        }
    }
}

pub struct Scripting {
    pub registry: Registry,
}

impl Scripting {
    pub fn install(registry: &mut Registry) {
        *registry = std::mem::take(registry).with_basic_types();
        registry.add_struct(NativeStructBuilder::new_uninitialized::<DynamicManaged>().build());
        registry.add_struct(NativeStructBuilder::new_uninitialized::<DynamicManagedRef>().build());
        registry
            .add_struct(NativeStructBuilder::new_uninitialized::<DynamicManagedRefMut>().build());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptingValue {
    pub type_name: String,
    #[serde(default)]
    pub value: PrefabValue,
}

impl ScriptingValue {
    pub fn register<T: DeserializeOwned + 'static>() {
        Self::register_named::<T>(std::any::type_name::<T>())
    }

    pub fn register_named<T: DeserializeOwned + 'static>(type_name: impl ToString) {
        if let Ok(mut factory) = SCRIPT_VALUE_FACTORY.write() {
            factory.insert(
                type_name.to_string(),
                Box::new(
                    |value| match serde_json::from_value::<T>(value.to_owned()) {
                        Ok(value) => Ok(DynamicManaged::new(value)),
                        Err(error) => Err(error.into()),
                    },
                ),
            );
        }
    }

    pub fn new<T: Serialize + 'static>(value: T) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            type_name: std::any::type_name::<T>().to_owned(),
            value: serde_json::to_value(value)?,
        })
    }

    pub fn produce(&self) -> Result<DynamicManaged, Box<dyn std::error::Error>> {
        match SCRIPT_VALUE_FACTORY.write() {
            Ok(mut factory) => match factory.get_mut(&self.type_name) {
                Some(factory) => (factory)(&self.value),
                None => Err(format!(
                    "Script value factory not found for type: {}",
                    self.type_name
                )
                .into()),
            },
            Err(error) => Err(error.into()),
        }
    }
}

pub fn bundle_installer<PB>(
    builder: &mut AppBuilder<PB>,
    registry: Registry,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
{
    ScriptingValue::register::<()>();
    ScriptingValue::register::<bool>();
    ScriptingValue::register::<i8>();
    ScriptingValue::register::<i16>();
    ScriptingValue::register::<i32>();
    ScriptingValue::register::<i64>();
    ScriptingValue::register::<i128>();
    ScriptingValue::register::<isize>();
    ScriptingValue::register::<u8>();
    ScriptingValue::register::<u16>();
    ScriptingValue::register::<u32>();
    ScriptingValue::register::<u64>();
    ScriptingValue::register::<u128>();
    ScriptingValue::register::<usize>();
    ScriptingValue::register::<f32>();
    ScriptingValue::register::<f64>();
    ScriptingValue::register::<char>();
    ScriptingValue::register_named::<String>("String");
    builder.install_resource(Scripting { registry });
    Ok(())
}

pub fn scripting_installer(registry: &mut Registry) {
    registry.add_struct(
        NativeStructBuilder::new_named_uninitialized::<crate::assets::database::AssetsDatabase>(
            "AssetsDatabase",
        )
        .module_name("core")
        .build(),
    );
}
