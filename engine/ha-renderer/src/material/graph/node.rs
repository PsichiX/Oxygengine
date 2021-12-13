use crate::{
    material::common::{
        MaterialDataPrecision, MaterialDataType, MaterialShaderType, MaterialValue,
        MaterialValueType,
    },
    math::vek::*,
};
use core::{id::ID, Ignite};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type MaterialGraphNodeId = ID<MaterialGraphNode>;

#[derive(Ignite, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialGraphInput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub data_precision: MaterialDataPrecision,
    #[serde(default)]
    pub data_type: MaterialDataType,
    #[serde(default)]
    pub value_type: MaterialValueType,
    #[serde(default)]
    pub shader_type: MaterialShaderType,
    #[serde(default)]
    pub default_value: Option<MaterialValue>,
}

impl MaterialGraphInput {
    pub fn vs_vertex_id() -> Self {
        Self {
            name: "gl_VertexID".to_owned(),
            data_precision: MaterialDataPrecision::Default,
            data_type: MaterialDataType::BuiltIn,
            value_type: MaterialValueType::Integer,
            shader_type: MaterialShaderType::Vertex,
            default_value: None,
        }
    }

    pub fn vs_instance_id() -> Self {
        Self {
            name: "gl_InstanceID".to_owned(),
            data_precision: MaterialDataPrecision::Default,
            data_type: MaterialDataType::BuiltIn,
            value_type: MaterialValueType::Integer,
            shader_type: MaterialShaderType::Vertex,
            default_value: None,
        }
    }

    pub fn fs_frag_coord() -> Self {
        Self {
            name: "gl_FragCoord".to_owned(),
            data_precision: MaterialDataPrecision::Default,
            data_type: MaterialDataType::BuiltIn,
            value_type: MaterialValueType::Vec4F,
            shader_type: MaterialShaderType::Fragment,
            default_value: None,
        }
    }

    pub fn fs_front_facing() -> Self {
        Self {
            name: "gl_FrontFacing".to_owned(),
            data_precision: MaterialDataPrecision::Default,
            data_type: MaterialDataType::BuiltIn,
            value_type: MaterialValueType::Bool,
            shader_type: MaterialShaderType::Fragment,
            default_value: None,
        }
    }

    pub fn fs_point_coord() -> Self {
        Self {
            name: "gl_PointCoord".to_owned(),
            data_precision: MaterialDataPrecision::Default,
            data_type: MaterialDataType::BuiltIn,
            value_type: MaterialValueType::Vec2F,
            shader_type: MaterialShaderType::Fragment,
            default_value: None,
        }
    }

    pub fn fs_clip_distance() -> Self {
        Self {
            name: "gl_ClipDistance".to_owned(),
            data_precision: MaterialDataPrecision::Default,
            data_type: MaterialDataType::BuiltIn,
            value_type: MaterialValueType::Array(Box::new(MaterialValueType::Scalar), None),
            shader_type: MaterialShaderType::Fragment,
            default_value: None,
        }
    }

    pub fn fs_primitive_id() -> Self {
        Self {
            name: "gl_PrimitiveID".to_owned(),
            data_precision: MaterialDataPrecision::Default,
            data_type: MaterialDataType::BuiltIn,
            value_type: MaterialValueType::Integer,
            shader_type: MaterialShaderType::Fragment,
            default_value: None,
        }
    }

    pub fn is_domain(&self) -> bool {
        self.data_type == MaterialDataType::Domain
    }

    pub fn is_vertex_input(&self) -> bool {
        self.shader_type == MaterialShaderType::Vertex
            && (self.data_type == MaterialDataType::Attribute
                || self.data_type == MaterialDataType::Uniform)
    }

    pub fn is_vertex_input_attribute(&self) -> bool {
        self.shader_type == MaterialShaderType::Vertex
            && self.data_type == MaterialDataType::Attribute
    }
}

#[derive(Ignite, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialGraphOutput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub data_type: MaterialDataType,
    #[serde(default)]
    pub value_type: MaterialValueType,
    #[serde(default)]
    pub shader_type: MaterialShaderType,
    #[serde(default)]
    pub(crate) input_connection: Option<MaterialGraphNodeId>,
}

impl MaterialGraphOutput {
    pub fn new(
        name: String,
        data_type: MaterialDataType,
        value_type: MaterialValueType,
        shader_type: MaterialShaderType,
    ) -> Self {
        Self {
            name,
            data_type,
            value_type,
            shader_type,
            input_connection: None,
        }
    }

    pub fn vs_position() -> Self {
        Self {
            name: "gl_Position".to_owned(),
            data_type: MaterialDataType::BuiltIn,
            value_type: MaterialValueType::Vec4F,
            shader_type: MaterialShaderType::Vertex,
            input_connection: None,
        }
    }

    pub fn vs_point_size() -> Self {
        Self {
            name: "gl_PointSize".to_owned(),
            data_type: MaterialDataType::BuiltIn,
            value_type: MaterialValueType::Scalar,
            shader_type: MaterialShaderType::Vertex,
            input_connection: None,
        }
    }

    pub fn vs_clip_distance() -> Self {
        Self {
            name: "gl_ClipDistance".to_owned(),
            data_type: MaterialDataType::BuiltIn,
            value_type: MaterialValueType::Array(Box::new(MaterialValueType::Scalar), None),
            shader_type: MaterialShaderType::Vertex,
            input_connection: None,
        }
    }

    pub fn fs_frag_depth() -> Self {
        Self {
            name: "gl_FragDepth".to_owned(),
            data_type: MaterialDataType::BuiltIn,
            value_type: MaterialValueType::Scalar,
            shader_type: MaterialShaderType::Fragment,
            input_connection: None,
        }
    }

    pub fn is_domain(&self) -> bool {
        self.data_type == MaterialDataType::Domain
    }

    pub fn is_fragment_output(&self) -> bool {
        self.shader_type == MaterialShaderType::Fragment
            && self.value_type == MaterialValueType::Vec4F
            && self.data_type == MaterialDataType::BufferOutput
    }

    pub fn clone_unconnected(&self) -> Self {
        Self {
            input_connection: None,
            ..self.clone()
        }
    }
}

#[derive(Ignite, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialGraphOperation {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub(crate) input_connections: HashMap<String, MaterialGraphNodeId>,
}

impl MaterialGraphOperation {
    pub fn new(name: String) -> Self {
        Self {
            name,
            input_connections: Default::default(),
        }
    }

    pub fn new_connected(
        name: String,
        input_connections: HashMap<String, MaterialGraphNodeId>,
    ) -> Self {
        Self {
            name,
            input_connections,
        }
    }

    pub fn clone_unconnected(&self) -> Self {
        Self {
            input_connections: Default::default(),
            ..self.clone()
        }
    }
}

#[derive(Ignite, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialGraphTransfer {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub(crate) input_connection: Option<MaterialGraphNodeId>,
}

impl MaterialGraphTransfer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            input_connection: None,
        }
    }

    pub fn new_connected(name: String, node: MaterialGraphNodeId) -> Self {
        Self {
            name,
            input_connection: Some(node),
        }
    }

    pub fn clone_unconnected(&self) -> Self {
        Self {
            input_connection: None,
            ..self.clone()
        }
    }
}

#[derive(Ignite, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaterialGraphNode {
    Value(MaterialValue),
    Input(MaterialGraphInput),
    Operation(MaterialGraphOperation),
    Transfer(MaterialGraphTransfer),
    Output(MaterialGraphOutput),
}

impl MaterialGraphNode {
    pub fn inputs(&self) -> HashSet<MaterialGraphNodeId> {
        match self {
            Self::Value(_) => Default::default(),
            Self::Input(_) => Default::default(),
            Self::Operation(node) => node.input_connections.values().cloned().collect(),
            Self::Transfer(node) => {
                if let Some(from) = node.input_connection {
                    let mut result = HashSet::with_capacity(1);
                    result.insert(from);
                    result
                } else {
                    Default::default()
                }
            }
            Self::Output(node) => {
                if let Some(from) = node.input_connection {
                    let mut result = HashSet::with_capacity(1);
                    result.insert(from);
                    result
                } else {
                    Default::default()
                }
            }
        }
    }

    pub fn is_domain(&self) -> bool {
        match self {
            Self::Input(v) => v.is_domain(),
            Self::Output(v) => v.is_domain(),
            _ => false,
        }
    }

    pub fn clone_unconnected(&self) -> Self {
        match self {
            Self::Value(_) | Self::Input(_) => self.clone(),
            Self::Operation(v) => Self::Operation(v.clone_unconnected()),
            Self::Transfer(v) => Self::Transfer(v.clone_unconnected()),
            Self::Output(v) => Self::Output(v.clone_unconnected()),
        }
    }
}

impl From<MaterialValue> for MaterialGraphNode {
    fn from(node: MaterialValue) -> Self {
        Self::Value(node)
    }
}

impl From<MaterialGraphInput> for MaterialGraphNode {
    fn from(node: MaterialGraphInput) -> Self {
        Self::Input(node)
    }
}

impl From<MaterialGraphOperation> for MaterialGraphNode {
    fn from(node: MaterialGraphOperation) -> Self {
        Self::Operation(node)
    }
}

impl From<MaterialGraphTransfer> for MaterialGraphNode {
    fn from(node: MaterialGraphTransfer) -> Self {
        Self::Transfer(node)
    }
}

impl From<MaterialGraphOutput> for MaterialGraphNode {
    fn from(node: MaterialGraphOutput) -> Self {
        Self::Output(node)
    }
}
