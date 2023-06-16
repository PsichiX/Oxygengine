use crate::{
    material::{
        common::{MaterialCompilationState, MaterialCompile, MaterialDataType, MaterialValueType},
        graph::MaterialGraph,
        MaterialError,
    },
    math::vek::*,
    resources::material_library::MaterialLibrary,
};
use core::utils::StringBuffer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaterialFunctionContent {
    Graph(MaterialGraph),
    Code(String),
    BuiltIn(String),
}

impl Default for MaterialFunctionContent {
    fn default() -> Self {
        Self::Graph(Default::default())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialFunctionInput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value_type: MaterialValueType,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialFunction {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub inputs: Vec<MaterialFunctionInput>,
    #[serde(default)]
    pub output: MaterialValueType,
    #[serde(default)]
    pub content: MaterialFunctionContent,
}

impl MaterialFunction {
    pub fn call_name(&self) -> &str {
        match &self.content {
            MaterialFunctionContent::BuiltIn(name) => name,
            _ => &self.name,
        }
    }

    pub fn validate(&self, library: &MaterialLibrary) -> Result<(), MaterialError> {
        if let MaterialFunctionContent::Graph(graph) = &self.content {
            graph.validate(library)?;
            if !graph.outputs().any(|(_, node)| {
                node.name == "result"
                    && node.value_type == self.output
                    && node.data_type == MaterialDataType::BuiltIn
            }) {
                return Err(MaterialError::FunctionOutputHasNoNode);
            }
            let inputs = graph
                .inputs()
                .map(|(_, node)| (&node.name, &node.value_type, node.data_type))
                .collect::<Vec<_>>();
            for input in &self.inputs {
                if !inputs.iter().any(|(n, t, d)| {
                    *n == &input.name && *t == &input.value_type && *d == MaterialDataType::BuiltIn
                }) {
                    return Err(MaterialError::FunctionInputHasNoNode(input.name.to_owned()));
                }
            }
        }
        Ok(())
    }

    pub fn can_be_compiled(&self) -> bool {
        !matches!(self.content, MaterialFunctionContent::BuiltIn(_))
    }

    fn compile_declaration(&self, output: &mut StringBuffer) -> std::io::Result<()> {
        if matches!(self.content, MaterialFunctionContent::BuiltIn(_)) {
            return Ok(());
        }
        output.write_str(self.output.to_string())?;
        output.write_space()?;
        output.write_str(&self.name)?;
        output.write_str("(")?;
        for (index, input) in self.inputs.iter().enumerate() {
            if index > 0 {
                output.write_str(", ")?;
            }
            output.write_str(input.value_type.to_string())?;
            output.write_space()?;
            output.write_str(&input.name)?;
        }
        output.write_str(");")?;
        output.write_new_line()?;
        Ok(())
    }

    fn compile_definition(
        &self,
        output: &mut StringBuffer,
        library: &MaterialLibrary,
    ) -> std::io::Result<()> {
        output.write_str(self.output.to_string())?;
        output.write_space()?;
        output.write_str(&self.name)?;
        output.write_str("(")?;
        for (index, input) in self.inputs.iter().enumerate() {
            if index > 0 {
                output.write_str(", ")?;
            }
            output.write_str(input.value_type.to_string())?;
            output.write_space()?;
            output.write_str(&input.name)?;
        }
        output.write_str(") {")?;
        match &self.content {
            MaterialFunctionContent::Graph(graph) => {
                output.push_level();
                output.write_new_line()?;
                output.write_str(self.output.to_string())?;
                output.write_space()?;
                output.write_str("result;")?;
                output.write_indented_lines(
                    graph.compile(MaterialCompilationState::FunctionBody { library })?,
                )?;
                output.write_new_line()?;
                output.write_str("return result;")?;
                output.pop_level();
                output.write_new_line()?;
            }
            MaterialFunctionContent::Code(content) => {
                output.push_level();
                output.write_indented_lines(content)?;
                output.pop_level();
                output.write_new_line()?;
            }
            MaterialFunctionContent::BuiltIn(_) => {}
        }
        output.write_str("}")?;
        output.write_new_line()?;
        Ok(())
    }
}

impl MaterialCompile<StringBuffer, String, MaterialCompilationState<'_>> for MaterialFunction {
    fn compile_to(
        &self,
        output: &mut StringBuffer,
        state: MaterialCompilationState,
    ) -> std::io::Result<()> {
        match state {
            MaterialCompilationState::FunctionDeclaration => {
                self.compile_declaration(output)?;
            }
            MaterialCompilationState::FunctionDefinition { library } => {
                self.compile_definition(output, library)?;
            }
            _ => {}
        }
        Ok(())
    }
}
