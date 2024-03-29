pub mod common;
pub mod domains;
pub mod graph;

use crate::{
    ha_renderer::{RenderStageResources, RenderStats},
    material::{
        common::{BakedMaterialShaders, MaterialSignature, MaterialValue, MaterialValueType},
        graph::{node::MaterialGraphNodeId, MaterialGraph},
    },
    render_target::RenderTargetError,
    resources::resource_mapping::ResourceMapping,
    HasContextResources, ResourceReference,
};
use core::id::ID;
use glow::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type MaterialId = ID<Material>;
pub type MaterialReference = ResourceReference<MaterialId>;
pub type MaterialResourceMapping = ResourceMapping<Material>;
type CompilationResultValue = (
    <Context as HasContext>::Program,
    HashMap<String, (<Context as HasContext>::UniformLocation, MaterialValueType)>,
    HashMap<String, <Context as HasContext>::Sampler>,
);

#[derive(Debug, Clone)]
pub enum MaterialError {
    NoResources,
    Internal(String),
    ShaderCompilation {
        vertex_errors: Option<String>,
        fragment_errors: Option<String>,
        vertex_shader: String,
        fragment_shader: String,
        link: Option<String>,
    },
    InvalidName {
        node: MaterialGraphNodeId,
        name: String,
    },
    NodeDoesNotExists(MaterialGraphNodeId),
    InvalidConnection {
        from: MaterialGraphNodeId,
        to: MaterialGraphNodeId,
    },
    InvalidConnectionParam {
        target: MaterialGraphNodeId,
        name: String,
    },
    InvalidConnectionSource {
        target: MaterialGraphNodeId,
        source: MaterialGraphNodeId,
    },
    MissingConnection {
        id: MaterialGraphNodeId,
        param: Option<String>,
    },
    TargetNodeRequiresParamName(MaterialGraphNodeId),
    AttributeInputHasNoDefaultValue {
        node: MaterialGraphNodeId,
        name: String,
    },
    FunctionNotFoundInLibrary {
        node: MaterialGraphNodeId,
        name: String,
    },
    CannotConnectNodeToItself(MaterialGraphNodeId),
    MismatchingConnectionTypes {
        from: MaterialGraphNodeId,
        from_value_type: Option<MaterialValueType>,
        to: MaterialGraphNodeId,
        to_value_type: Option<MaterialValueType>,
        param: Option<String>,
    },
    GraphIsCyclic(Vec<MaterialGraphNodeId>),
    NoTransferFound(MaterialGraphNodeId),
    CouldNotCompileVertexShader(String),
    CouldNotCompileFragmentShader(String),
    FunctionOutputHasNoNode,
    FunctionInputHasNoNode(String),
    NoShaderVersion(Box<MaterialSignature>),
    InvalidUniformTypeToSubmit(MaterialValueType),
    CouldNotBuildSubgraphForSignature(Box<MaterialSignature>),
    SubgraphInputsDoesNotMatchSignature(HashSet<String>, Box<MaterialSignature>),
    Baking(MaterialGraph, Box<MaterialError>),
    CouldNotCreateRenderTarget(Box<RenderTargetError>),
    FunctionIsNotValidMiddleware(String),
    FunctionDoesNotExists(String),
    MiddlewareDoesNotExists(String),
}

#[derive(Debug)]
pub struct MaterialResourceHandles {
    pub program: <Context as HasContext>::Program,
    pub uniforms: HashMap<String, (<Context as HasContext>::UniformLocation, MaterialValueType)>,
    pub samplers: HashMap<String, <Context as HasContext>::Sampler>,
}

#[cfg(feature = "web")]
unsafe impl Send for MaterialResourceHandles {}
#[cfg(feature = "web")]
unsafe impl Sync for MaterialResourceHandles {}

#[derive(Debug)]
struct MaterialResources(HashMap<MaterialSignature, MaterialResourceHandles>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaterialContent {
    Graph(MaterialGraph),
    Baked,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialBlending {
    None,
    Alpha,
    Additive,
}

impl Default for MaterialBlending {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDrawOptions {
    #[serde(default = "MaterialDrawOptions::default_color_mask")]
    pub color_mask: [bool; 4],
    #[serde(default = "MaterialDrawOptions::default_depth_mask")]
    pub depth_mask: bool,
    #[serde(default)]
    pub blending: MaterialBlending,
}

impl Default for MaterialDrawOptions {
    fn default() -> Self {
        Self {
            color_mask: Self::default_color_mask(),
            depth_mask: Self::default_depth_mask(),
            blending: Default::default(),
        }
    }
}

impl MaterialDrawOptions {
    fn default_color_mask() -> [bool; 4] {
        [true, true, true, true]
    }

    fn default_depth_mask() -> bool {
        true
    }

    pub fn transparent() -> Self {
        Self {
            blending: MaterialBlending::Alpha,
            ..Default::default()
        }
    }
}

impl MaterialDrawOptions {
    pub fn apply(&self, context: &Context, render_stats: &mut RenderStats) {
        let context = context;
        unsafe {
            context.color_mask(
                self.color_mask[0],
                self.color_mask[1],
                self.color_mask[2],
                self.color_mask[3],
            );
            context.depth_mask(self.depth_mask);
            match self.blending {
                MaterialBlending::None => context.blend_func(ONE, ZERO),
                MaterialBlending::Alpha => context.blend_func(SRC_ALPHA, ONE_MINUS_SRC_ALPHA),
                MaterialBlending::Additive => context.blend_func(ONE, ONE),
            }
        }
        render_stats.state_changes += 3;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaterialDetailedInfo {
    pub versions: HashMap<MaterialSignature, BakedMaterialShaders>,
    pub default_values: HashMap<String, MaterialValue>,
    pub draw_options: MaterialDrawOptions,
}

#[derive(Debug)]
pub struct Material {
    content: MaterialContent,
    versions: HashMap<MaterialSignature, BakedMaterialShaders>,
    pub default_values: HashMap<String, MaterialValue>,
    pub draw_options: MaterialDrawOptions,
    resources: Option<MaterialResources>,
}

impl Drop for Material {
    fn drop(&mut self) {
        if self.resources.is_some() {
            panic!(
                "Dropping {} without calling `context_release` to release resources first!",
                std::any::type_name::<Self>()
            );
        }
    }
}

impl HasContextResources<Context> for Material {
    type Error = MaterialError;

    fn has_resources(&self) -> bool {
        self.resources.is_some()
    }

    fn context_initialize(&mut self, context: &Context) -> Result<(), Self::Error> {
        self.context_release(context)?;
        let mut handles = HashMap::with_capacity(self.versions.len());
        for (signature, baked) in &self.versions {
            let (program, uniforms, samplers) = Self::compile_program(context, baked)?;
            for (i, name) in baked.samplers.iter().enumerate() {
                if let Some((location, value_type)) = uniforms.get(name) {
                    if matches!(
                        value_type,
                        MaterialValueType::Sampler2d
                            | MaterialValueType::Sampler2dArray
                            | MaterialValueType::Sampler3d
                    ) {
                        unsafe {
                            context.uniform_1_i32(Some(location), i as i32);
                            context.active_texture(TEXTURE0 + i as u32);
                        }
                    }
                }
            }
            handles.insert(
                signature.to_owned(),
                MaterialResourceHandles {
                    program,
                    uniforms,
                    samplers,
                },
            );
        }
        self.resources = Some(MaterialResources(handles));
        Ok(())
    }

    fn context_release(&mut self, context: &Context) -> Result<(), Self::Error> {
        if let Some(resources) = std::mem::take(&mut self.resources) {
            unsafe {
                for handles in resources.0.values() {
                    context.delete_program(handles.program);
                    for handle in handles.samplers.values() {
                        context.delete_sampler(*handle);
                    }
                }
            }
        }
        Ok(())
    }
}

impl Material {
    pub fn new_graph(graph: MaterialGraph) -> Self {
        let default_values = graph
            .default_uniform_values()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        Self {
            content: MaterialContent::Graph(graph),
            versions: Default::default(),
            default_values,
            draw_options: Default::default(),
            resources: None,
        }
    }

    pub fn new_baked(versions: HashMap<MaterialSignature, BakedMaterialShaders>) -> Self {
        Self {
            content: MaterialContent::Baked,
            versions,
            default_values: Default::default(),
            draw_options: Default::default(),
            resources: None,
        }
    }

    pub fn detailed_info(&self) -> MaterialDetailedInfo {
        MaterialDetailedInfo {
            versions: self.versions.clone(),
            default_values: self.default_values.clone(),
            draw_options: self.draw_options.clone(),
        }
    }

    pub fn graph(&self) -> Option<&MaterialGraph> {
        match &self.content {
            MaterialContent::Graph(graph) => Some(graph),
            _ => None,
        }
    }

    pub fn versions(&self) -> impl Iterator<Item = &MaterialSignature> {
        self.versions.keys()
    }

    pub fn resources(
        &self,
        signature: &MaterialSignature,
        _: &RenderStageResources<'_>,
    ) -> Option<&MaterialResourceHandles> {
        match &self.resources {
            Some(resources) => resources.0.get(signature),
            None => None,
        }
    }

    pub(crate) fn activate(
        &self,
        signature: &MaterialSignature,
        context: &Context,
        render_stage_resources: &RenderStageResources<'_>,
        render_stats: &mut RenderStats,
    ) -> Result<(), MaterialError> {
        let resources = match &self.resources {
            Some(resources) => resources,
            None => return Err(MaterialError::NoResources),
        };
        let handles = match resources.0.get(signature) {
            Some(handles) => handles,
            None => {
                return Err(MaterialError::NoShaderVersion(Box::new(
                    signature.to_owned(),
                )))
            }
        };
        self.draw_options.apply(context, render_stats);
        unsafe {
            context.use_program(Some(handles.program));
            render_stats.material_changes += 1;
        }
        for (name, value) in &self.default_values {
            self.submit_uniform(
                signature,
                name,
                value,
                context,
                render_stage_resources,
                render_stats,
            )?;
        }
        Ok(())
    }

    pub fn has_uniform(
        &self,
        signature: &MaterialSignature,
        name: &str,
        value_type: Option<&MaterialValueType>,
    ) -> Result<bool, MaterialError> {
        let resources = match &self.resources {
            Some(resources) => resources,
            None => return Err(MaterialError::NoResources),
        };
        let handles = match resources.0.get(signature) {
            Some(handles) => handles,
            None => {
                return Err(MaterialError::NoShaderVersion(Box::new(
                    signature.to_owned(),
                )))
            }
        };
        if let Some((_, a)) = handles.uniforms.get(name) {
            Ok(value_type.map(|b| a == b).unwrap_or(true))
        } else {
            Ok(false)
        }
    }

    pub(crate) fn submit_uniform(
        &self,
        signature: &MaterialSignature,
        name: &str,
        value: &MaterialValue,
        context: &Context,
        render_stage_resources: &RenderStageResources<'_>,
        render_stats: &mut RenderStats,
    ) -> Result<(), MaterialError> {
        let resources = match &self.resources {
            Some(resources) => resources,
            None => {
                return Err(MaterialError::NoResources);
            }
        };
        let handles = match resources.0.get(signature) {
            Some(handles) => handles,
            None => {
                return Err(MaterialError::NoShaderVersion(Box::new(
                    signature.to_owned(),
                )));
            }
        };
        let (handle, value_type) = match handles.uniforms.get(name) {
            Some(result) => result,
            None => {
                return Ok(());
            }
        };
        if &value.value_type() != value_type {
            return Err(MaterialError::InvalidUniformTypeToSubmit(
                value.value_type(),
            ));
        }
        match value {
            MaterialValue::Scalar(value) => {
                unsafe {
                    context.uniform_1_f32(Some(handle), *value);
                    render_stats.uniform_changes += 1;
                }
                Ok(())
            }
            MaterialValue::Vec2F(value) => {
                unsafe {
                    context.uniform_2_f32_slice(Some(handle), value.as_slice());
                    render_stats.uniform_changes += 1;
                }
                Ok(())
            }
            MaterialValue::Vec3F(value) => {
                unsafe {
                    context.uniform_3_f32_slice(Some(handle), value.as_slice());
                    render_stats.uniform_changes += 1;
                }
                Ok(())
            }
            MaterialValue::Vec4F(value) => {
                unsafe {
                    context.uniform_4_f32_slice(Some(handle), value.as_slice());
                    render_stats.uniform_changes += 1;
                }
                Ok(())
            }
            MaterialValue::Mat2F(value) => {
                unsafe {
                    context.uniform_matrix_2_f32_slice(Some(handle), false, value.as_col_slice());
                    render_stats.uniform_changes += 1;
                }
                Ok(())
            }
            MaterialValue::Mat3F(value) => {
                unsafe {
                    context.uniform_matrix_3_f32_slice(Some(handle), false, value.as_col_slice());
                    render_stats.uniform_changes += 1;
                }
                Ok(())
            }
            MaterialValue::Mat4F(value) => {
                unsafe {
                    context.uniform_matrix_4_f32_slice(Some(handle), false, value.as_col_slice());
                    render_stats.uniform_changes += 1;
                }
                Ok(())
            }
            MaterialValue::Integer(value) => {
                unsafe {
                    context.uniform_1_i32(Some(handle), *value);
                    render_stats.uniform_changes += 1;
                }
                Ok(())
            }
            MaterialValue::Vec2I(value) => {
                unsafe {
                    context.uniform_2_i32_slice(Some(handle), value.as_slice());
                    render_stats.uniform_changes += 1;
                }
                Ok(())
            }
            MaterialValue::Vec3I(value) => {
                unsafe {
                    context.uniform_3_i32_slice(Some(handle), value.as_slice());
                    render_stats.uniform_changes += 1;
                }
                Ok(())
            }
            MaterialValue::Vec4I(value) => {
                unsafe {
                    context.uniform_4_i32_slice(Some(handle), value.as_slice());
                    render_stats.uniform_changes += 1;
                }
                Ok(())
            }
            MaterialValue::Sampler2d {
                reference,
                filtering,
            }
            | MaterialValue::Sampler2dArray {
                reference,
                filtering,
            }
            | MaterialValue::Sampler3d {
                reference,
                filtering,
            } => {
                if let Some(texture) = render_stage_resources.image_handle_by_ref(reference) {
                    if let Some((index, sampler)) = handles
                        .samplers
                        .iter()
                        .enumerate()
                        .find_map(|(i, (n, h))| if n == name { Some((i, h)) } else { None })
                    {
                        unsafe {
                            let (min, mag) = filtering.as_gl();
                            let gl_mode = match value {
                                MaterialValue::Sampler2d { .. } => TEXTURE_2D,
                                MaterialValue::Sampler2dArray { .. } => TEXTURE_2D_ARRAY,
                                MaterialValue::Sampler3d { .. } => TEXTURE_3D,
                                _ => unreachable!(),
                            };
                            context.sampler_parameter_i32(*sampler, TEXTURE_MIN_FILTER, min as _);
                            context.sampler_parameter_i32(*sampler, TEXTURE_MAG_FILTER, mag as _);
                            context.active_texture(TEXTURE0 + index as u32);
                            context.bind_texture(gl_mode, Some(texture));
                            context.bind_sampler(index as u32, Some(*sampler));
                            context.uniform_1_i32(Some(handle), index as i32);
                            render_stats.sampler_changes += 1;
                        }
                    }
                }
                Ok(())
            }
            _ => Err(MaterialError::InvalidUniformTypeToSubmit(
                value.value_type(),
            )),
        }
    }

    fn compile_program(
        context: &Context,
        baked: &BakedMaterialShaders,
    ) -> Result<CompilationResultValue, MaterialError> {
        unsafe {
            let vertex_shader_handle = match context.create_shader(VERTEX_SHADER) {
                Ok(handle) => handle,
                Err(error) => return Err(MaterialError::Internal(error)),
            };
            let fragment_shader_handle = match context.create_shader(FRAGMENT_SHADER) {
                Ok(handle) => handle,
                Err(error) => {
                    context.delete_shader(vertex_shader_handle);
                    return Err(MaterialError::Internal(error));
                }
            };
            let program_handle = match context.create_program() {
                Ok(handle) => handle,
                Err(error) => {
                    context.delete_shader(vertex_shader_handle);
                    context.delete_shader(fragment_shader_handle);
                    return Err(MaterialError::Internal(error));
                }
            };

            context.shader_source(vertex_shader_handle, &baked.vertex);
            context.shader_source(fragment_shader_handle, &baked.fragment);
            context.compile_shader(vertex_shader_handle);
            context.compile_shader(fragment_shader_handle);
            let vertex_errors = if context.get_shader_compile_status(vertex_shader_handle) {
                None
            } else {
                Some(context.get_shader_info_log(vertex_shader_handle))
            };
            let fragment_errors = if context.get_shader_compile_status(fragment_shader_handle) {
                None
            } else {
                Some(context.get_shader_info_log(fragment_shader_handle))
            };
            context.attach_shader(program_handle, vertex_shader_handle);
            context.attach_shader(program_handle, fragment_shader_handle);
            context.link_program(program_handle);
            let link_errors = if context.get_program_link_status(program_handle) {
                None
            } else {
                Some(context.get_program_info_log(program_handle))
            };
            context.delete_shader(vertex_shader_handle);
            context.delete_shader(fragment_shader_handle);
            if vertex_errors.is_some() || fragment_errors.is_some() || link_errors.is_some() {
                context.delete_program(program_handle);
                return Err(MaterialError::ShaderCompilation {
                    vertex_errors,
                    fragment_errors,
                    vertex_shader: baked.vertex.to_owned(),
                    fragment_shader: baked.fragment.to_owned(),
                    link: link_errors,
                });
            }
            let mut uniforms = HashMap::with_capacity(baked.uniforms.len());
            for (name, value_type) in &baked.uniforms {
                if let Some(location) = context.get_uniform_location(program_handle, name) {
                    uniforms.insert(name.to_owned(), (location, value_type.to_owned()));
                }
            }
            let mut samplers = HashMap::with_capacity(baked.samplers.len());
            for name in &baked.samplers {
                if let Ok(handle) = context.create_sampler() {
                    context.sampler_parameter_i32(handle, TEXTURE_WRAP_S, CLAMP_TO_EDGE as _);
                    context.sampler_parameter_i32(handle, TEXTURE_WRAP_T, CLAMP_TO_EDGE as _);
                    context.sampler_parameter_i32(handle, TEXTURE_WRAP_R, CLAMP_TO_EDGE as _);
                    samplers.insert(name.to_owned(), handle);
                }
            }
            Ok((program_handle, uniforms, samplers))
        }
    }

    pub(crate) fn add_version(
        &mut self,
        context: &Context,
        signature: MaterialSignature,
        baked: BakedMaterialShaders,
    ) -> Result<(), MaterialError> {
        if matches!(self.content, MaterialContent::Baked) {
            return Ok(());
        }
        self.remove_version(context, &signature)?;
        self.versions.insert(signature.to_owned(), baked.to_owned());
        let resources = match &mut self.resources {
            Some(resources) => resources,
            None => return Err(MaterialError::NoResources),
        };
        let (program, uniforms, samplers) = Self::compile_program(context, &baked)?;
        resources.0.insert(
            signature,
            MaterialResourceHandles {
                program,
                uniforms,
                samplers,
            },
        );
        Ok(())
    }

    pub(crate) fn remove_version(
        &mut self,
        context: &Context,
        signature: &MaterialSignature,
    ) -> Result<(), MaterialError> {
        if matches!(self.content, MaterialContent::Baked) {
            return Ok(());
        }
        self.versions.remove(signature);
        let resources = match &mut self.resources {
            Some(resources) => resources,
            None => return Err(MaterialError::NoResources),
        };
        if let Some(handles) = resources.0.remove(signature) {
            unsafe { context.delete_program(handles.program) };
        }
        Ok(())
    }

    pub(crate) fn query_is_high_precision_supported_in_fragment_shader(context: &Context) -> bool {
        unsafe {
            let fragment_shader_handle = match context.create_shader(FRAGMENT_SHADER) {
                Ok(shader) => shader,
                Err(_) => return false,
            };
            context.shader_source(
                fragment_shader_handle,
                "#version 300 es\nprecision highp float;\nvoid main() {}\n",
            );
            context.compile_shader(fragment_shader_handle);
            let status = context.get_shader_compile_status(fragment_shader_handle);
            context.delete_shader(fragment_shader_handle);
            status
        }
    }
}

#[macro_export]
macro_rules! material_value_type {
    (bool) => {
        $crate::material::common::MaterialValueType::Bool
    };
    (bvec2) => {
        $crate::material::common::MaterialValueType::Vec2B
    };
    (bvec3) => {
        $crate::material::common::MaterialValueType::Vec3B
    };
    (bvec4) => {
        $crate::material::common::MaterialValueType::Vec4B
    };
    (bmat2) => {
        $crate::material::common::MaterialValueType::Mat2B
    };
    (bmat3) => {
        $crate::material::common::MaterialValueType::Mat3B
    };
    (bmat4) => {
        $crate::material::common::MaterialValueType::Mat4B
    };
    (float) => {
        $crate::material::common::MaterialValueType::Scalar
    };
    (vec2) => {
        $crate::material::common::MaterialValueType::Vec2F
    };
    (vec3) => {
        $crate::material::common::MaterialValueType::Vec3F
    };
    (vec4) => {
        $crate::material::common::MaterialValueType::Vec4F
    };
    (mat2) => {
        $crate::material::common::MaterialValueType::Mat2F
    };
    (mat3) => {
        $crate::material::common::MaterialValueType::Mat3F
    };
    (mat4) => {
        $crate::material::common::MaterialValueType::Mat4F
    };
    (int) => {
        $crate::material::common::MaterialValueType::Integer
    };
    (ivec2) => {
        $crate::material::common::MaterialValueType::Vec2I
    };
    (ivec3) => {
        $crate::material::common::MaterialValueType::Vec3I
    };
    (ivec4) => {
        $crate::material::common::MaterialValueType::Vec4I
    };
    (imat2) => {
        $crate::material::common::MaterialValueType::Mat2I
    };
    (imat3) => {
        $crate::material::common::MaterialValueType::Mat3I
    };
    (imat4) => {
        $crate::material::common::MaterialValueType::Mat4I
    };
    (sampler2D) => {
        $crate::material::common::MaterialValueType::Sampler2d
    };
    (sampler2DArray) => {
        $crate::material::common::MaterialValueType::Sampler2dArray
    };
    (sampler3D) => {
        $crate::material::common::MaterialValueType::Sampler3d
    };
    ([ $ty:ident ; $count:literal ]) => {
        $crate::material::common::MaterialValueType::Array(
            Box::new($crate::material_value_type!($ty)),
            Some($count as usize),
        )
    };
    ([ $ty:ident ]) => {
        $crate::material::common::MaterialValueType::Array(
            Box::new($crate::material_value_type!($ty)),
            None,
        )
    };
    ({ $ty:expr }) => {{
        $ty
    }};
}

#[macro_export]
macro_rules! material_function {
    (
        fn $name:ident ( $( $arg_name:ident : $arg_type:tt ),* ) -> $ret_type:tt
        { $content:expr }
    ) => {
        {
            $crate::material::graph::function::MaterialFunction {
                name: stringify!($name).to_owned(),
                inputs: vec![ $( $crate::material::graph::function::MaterialFunctionInput {
                    name: stringify!($arg_name).to_owned(),
                    value_type: $crate::material_value_type!($arg_type),
                } ),* ],
                output: $crate::material_value_type!($ret_type),
                content: $content,
            }
        }
    };
}

#[macro_export]
macro_rules! material_functions {
    (
        $( { fn $name:ident ( $( $arg_name:ident : $arg_type:tt ),* ) -> $ret_type:tt } )+
        { $content:expr }
    ) => {
        vec![
            $(
                $crate::material_function! {
                    fn $name ( $( $arg_name : $arg_type ),* ) -> $ret_type { $content }
                }
            ),+
        ]
    };
}

#[macro_export]
macro_rules! graph_material_function {
    (
        fn $name:ident ( $( $arg_name:ident : $arg_type:tt ),* ) -> $ret_type:tt
        { $( $statement:tt )* }
    ) => {
        $crate::material_function! {
            fn $name ( $( $arg_name : $arg_type ),* ) -> $ret_type
            {
                {
                    #[allow(unused_mut)]
                    let mut ___graph = $crate::material::graph::MaterialGraph::default();
                    $(
                        #[allow(non_snake_case)]
                        let $arg_name = ___graph.add_node($crate::material_graph_input! {
                            builtin $arg_name : $arg_type
                        });
                    )*
                    #[allow(unused_variables)]
                    let result = ___graph.add_node($crate::material_graph_output! {
                        builtin result : $ret_type
                    });
                    $(
                        $crate::material_graph!( @statement $statement, ___graph, result );
                    )*
                    $crate::material::graph::function::MaterialFunctionContent::Graph(___graph)
                }
            }
        }
    };
}

#[macro_export]
macro_rules! code_material_function {
    (
        fn $name:ident ( $( $arg_name:ident : $arg_type:tt ),* ) -> $ret_type:tt
        { $code:literal }
    ) => {
        $crate::material_function! {
            fn $name ( $( $arg_name : $arg_type ),* ) -> $ret_type
            { $crate::material::graph::function::MaterialFunctionContent::Code($code.to_string()) }
        }
    };
}

#[macro_export]
macro_rules! code_material_functions {
    (
        $( { fn $name:ident ( $( $arg_name:ident : $arg_type:tt ),* ) -> $ret_type:tt } )+
        { $code:literal }
    ) => {
        $crate::material_functions! {
            $( { fn $name ( $( $arg_name : $arg_type ),* ) -> $ret_type } )+
            { $crate::material::graph::function::MaterialFunctionContent::Code($code.to_string()) }
        }
    };
}

#[macro_export]
macro_rules! builtin_material_function {
    (
        fn $name:ident ( $( $arg_name:ident : $arg_type:tt ),* ) -> $ret_type:tt
        { $alias:literal }
    ) => {
        $crate::material_function! {
            fn $name ( $( $arg_name : $arg_type ),* ) -> $ret_type
            {
                $crate::material::graph::function::MaterialFunctionContent::BuiltIn(
                    $alias.to_string()
                )
            }
        }
    };
}

#[macro_export]
macro_rules! builtin_material_functions {
    (
        $( { fn $name:ident ( $( $arg_name:ident : $arg_type:tt ),* ) -> $ret_type:tt } )+
        { $alias:literal }
    ) => {
        $crate::material_functions! {
            $( { fn $name ( $( $arg_name : $arg_type ),* ) -> $ret_type } )+
            {
                $crate::material::graph::function::MaterialFunctionContent::BuiltIn(
                    $alias.to_string()
                )
            }
        }
    };
}

#[macro_export]
macro_rules! material_graph_input {
    (vertex) => {
        $crate::material::common::MaterialShaderType::Vertex
    };
    (fragment) => {
        $crate::material::common::MaterialShaderType::Fragment
    };
    (builtin) => {
        ($crate::material::common::MaterialDataType::BuiltIn, false)
    };
    (in) => {
        ($crate::material::common::MaterialDataType::Attribute, false)
    };
    (inout) => {
        ($crate::material::common::MaterialDataType::Attribute, true)
    };
    (uniform) => {
        ($crate::material::common::MaterialDataType::Uniform, false)
    };
    (defaultp) => {
        $crate::material::common::MaterialDataPrecision::Default
    };
    (lowp) => {
        $crate::material::common::MaterialDataPrecision::Low
    };
    (mediump) => {
        $crate::material::common::MaterialDataPrecision::Medium
    };
    (highp) => {
        $crate::material::common::MaterialDataPrecision::High
    };
    (
        $( [ $shader_type:ident ] )?
        $data_type:ident $name:ident : $value_type:tt $( @ $data_precision:ident )? $( = $default_value:expr )?
    ) => {
        {
            let (data_type, undirected) = $crate::material_graph_input!($data_type);
            #[allow(unused_assignments, unused_mut)]
            let mut data_precision = $crate::material::common::MaterialDataPrecision::Default;
            $(
                data_precision = $crate::material_graph_input!($data_precision);
            )?
            #[allow(unused_assignments, unused_mut)]
            let mut shader_type = $crate::material::common::MaterialShaderType::Undefined;
            $(
                shader_type = $crate::material_graph_input!($shader_type);
            )?
            #[allow(unused_assignments, unused_mut)]
            let mut default_value = None;
            $(
                default_value = Some($default_value.into());
            )?
            $crate::material::graph::node::MaterialGraphInput {
                name: stringify!($name).to_owned(),
                undirected,
                data_precision,
                data_type,
                value_type: $crate::material_value_type!($value_type),
                shader_type,
                default_value,
            }.into()
        }
    };
}

#[macro_export]
macro_rules! material_graph_output {
    (vertex) => {
        $crate::material::common::MaterialShaderType::Vertex
    };
    (fragment) => {
        $crate::material::common::MaterialShaderType::Fragment
    };
    (builtin) => {
        ($crate::material::common::MaterialDataType::BuiltIn, false)
    };
    (out) => {
        ($crate::material::common::MaterialDataType::BufferOutput, false)
    };
    (inout) => {
        ($crate::material::common::MaterialDataType::BufferOutput, true)
    };
    ( $( [ $shader_type:ident ] )? $data_type:ident $name:ident : $value_type:tt ) => {
        {
            let (data_type, undirected) = $crate::material_graph_output!($data_type);
            #[allow(unused_assignments, unused_mut)]
            let mut shader_type = $crate::material::common::MaterialShaderType::Undefined;
            $(
                shader_type = $crate::material_graph_output!($shader_type);
            )?
            $crate::material::graph::node::MaterialGraphOutput::new(
                stringify!($name).to_owned(),
                undirected,
                data_type,
                $crate::material_value_type!($value_type),
                shader_type,
            ).into()
        }
    };
}

#[macro_export]
macro_rules! material_graph {
    (
        inputs {
            $(
                $( [ $in_shader_type:ident ] )?
                $in_data_type:ident $in_name:ident $( as $in_as_name:ident )? : $in_value_type:tt
                $( @ $in_data_precision:ident )?
                $( = $in_default_value:expr )?;
            )*
        }
        outputs {
            $(
                $( [ $out_shader_type:ident ] )?
                $out_data_type:ident $out_name:ident $( as $out_as_name:ident )? : $out_value_type:tt;
            )*
        }
        $( $statement:tt )*
    ) => {
        {
            #[allow(unused_mut)]
            let mut ___graph = $crate::material::graph::MaterialGraph::default();
            $(
                #[deny(clippy::shadow_reuse)]
                #[warn(clippy::shadow_unrelated)]
                #[allow(non_snake_case)]
                let $crate::material_graph!(@optional_name $in_name $( as $in_as_name )?) = {
                    ___graph.add_node($crate::material_graph_input! {
                        $( [ $in_shader_type ] )?
                        $in_data_type $in_name : $in_value_type
                        $( @ $in_data_precision )?
                        $( = $in_default_value )?
                    })
                };
            )*
            $(
                #[deny(clippy::shadow_reuse)]
                #[warn(clippy::shadow_unrelated)]
                #[allow(non_snake_case)]
                let $crate::material_graph!(@optional_name $out_name $( as $out_as_name )?) = {
                    ___graph.add_node($crate::material_graph_output! {
                        $( [ $out_shader_type ] )? $out_data_type $out_name : $out_value_type
                    })
                };
            )*
            $(
                $crate::material_graph!( @statement $statement, ___graph, result );
            )*
            ___graph
        }
    };
    ( @optional_name $in_name:ident as $in_as_name:ident ) => { $in_as_name };
    ( @optional_name $in_name:ident ) => { $in_name };
    ( @statement [ return $expression:tt ], $graph:expr, $result:ident ) => {
        {
            let ___source = $crate::material_graph!(@expression $expression, $graph);
            let _ = $graph.connect(___source, $result, None);
        }
    };
    ( @statement [ $node:ident := $expression:tt ], $graph:expr, $result:ident ) => {
        #[allow(clippy::shadow_reuse)]
        #[allow(clippy::shadow_unrelated)]
        #[allow(non_snake_case)]
        let $node = $crate::material_graph!(@expression $expression, $graph);
    };
    ( @statement [ $node:ident = $expression:tt ], $graph:expr, $result:ident ) => {
        #[deny(clippy::shadow_reuse)]
        #[warn(clippy::shadow_unrelated)]
        #[allow(non_snake_case)]
        let $node = $crate::material_graph!(@expression $expression, $graph);
    };
    ( @statement [ $expression:tt -> $target:ident ], $graph:expr, $result:ident ) => {
        {
            let ___source = $crate::material_graph!(@expression $expression, $graph);
            let _ = $graph.connect(___source, $target, None);
        }
    };
    ( @expression { $value:expr }, $graph:expr ) => {
        {
            let ___temp = $crate::material::common::MaterialValue::from($value).into();
            $graph.add_node(___temp)
        }
    };
    ( @expression [ $node:tt => $name:ident ], $graph:expr ) => {
        {
            let ___temp = $crate::material::graph::node::MaterialGraphTransfer::new_connected(
                stringify!($name).to_owned(),
                $crate::material_graph!(@expression $node, $graph)
            ).into();
            $graph.add_node(___temp)
        }
    };
    ( @expression ( $name:tt $( , $param_name:ident : $param_value:tt )* ), $graph:expr ) => {
        {
            let mut ___connections = std::collections::HashMap::<
                String,
                $crate::material::graph::node::MaterialGraphNodeId,
            >::new();
            $(
                ___connections.insert(
                    stringify!($param_name).to_owned(),
                    $crate::material_graph!(@expression $param_value, $graph),
                );
            )*
            let ___temp = $crate::material::graph::node::MaterialGraphOperation::new_connected(
                $crate::material_graph!(@name $name),
                ___connections,
            ).into();
            $graph.add_node(___temp)
        }
    };
    (@expression $node:ident, $graph:expr) => {
        $node
    };
    (@name $name:ident) => {
        stringify!($name).to_owned()
    };
    (@name {$name:expr}) => {
        {$name.to_string()}
    };
}
