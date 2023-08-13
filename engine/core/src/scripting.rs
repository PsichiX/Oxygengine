use crate::{
    app::AppBuilder,
    ecs::pipeline::{PipelineBuilder, PipelineBuilderError},
};
pub use intuicio_essentials as intuicio;
use intuicio_essentials::core::{
    function::FunctionQuery, registry::Registry, struct_type::StructQuery,
};
use pest::Parser;
use serde::{Deserialize, Serialize};

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
    }
}

pub fn bundle_installer<PB>(
    builder: &mut AppBuilder<PB>,
    registry: Registry,
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
{
    builder.install_resource(Scripting { registry });
    Ok(())
}
