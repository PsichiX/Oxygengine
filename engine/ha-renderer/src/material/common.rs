use crate::{
    image::{ImageFiltering, ImageReference, ImageResourceMapping},
    math::vek::*,
    mesh::VertexLayout,
    render_target::RenderTarget,
    resources::material_library::MaterialLibrary,
};
use core::utils::{StrSequence, StringSequence};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    io::Write,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialMeshSignature(Vec<(String, usize)>);

impl MaterialMeshSignature {
    pub fn new(vertex_layout: &VertexLayout) -> Self {
        Self(
            vertex_layout
                .vertex_attribs()
                .map(|(_, id, chunk)| (id.to_owned(), chunk.location()))
                .collect(),
        )
    }

    /// # Safety
    /// Constructing signature from raw data might cause invalid signature.
    /// Consider using safe constructors.
    pub unsafe fn from_raw(data: Vec<(String, usize)>) -> Self {
        Self(data)
    }

    pub fn sources(&self) -> impl Iterator<Item = (&str, usize)> {
        self.0.iter().map(|(id, loc)| (id.as_str(), *loc))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialRenderTargetSignature(Vec<String>);

impl MaterialRenderTargetSignature {
    pub fn new(render_target: &RenderTarget) -> Self {
        Self(
            render_target
                .fragment_buffers()
                .map(|id| id.to_owned())
                .collect(),
        )
    }

    /// # Safety
    /// Constructing signature from raw data might cause invalid signature.
    /// Consider using safe constructors.
    pub unsafe fn from_raw(data: Vec<String>) -> Self {
        Self(data)
    }

    pub fn targets(&self) -> impl Iterator<Item = (&str, usize)> {
        self.0
            .iter()
            .enumerate()
            .map(|(index, id)| (id.as_str(), index))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialSignature {
    mesh: MaterialMeshSignature,
    render_target: MaterialRenderTargetSignature,
    domain: Option<String>,
    middlewares: StringSequence,
}

impl MaterialSignature {
    pub fn new(
        mesh: MaterialMeshSignature,
        render_target: MaterialRenderTargetSignature,
        domain: Option<String>,
        middlewares: StringSequence,
    ) -> Self {
        Self {
            mesh,
            render_target,
            domain,
            middlewares,
        }
    }

    pub fn from_objects(
        vertex_layout: &VertexLayout,
        render_target: &RenderTarget,
        domain: Option<String>,
        middlewares: StringSequence,
    ) -> Self {
        let mesh = MaterialMeshSignature::new(vertex_layout);
        let render_target = MaterialRenderTargetSignature::new(render_target);
        Self::new(mesh, render_target, domain, middlewares)
    }

    /// # Safety
    /// Constructing signature from raw data might cause invalid signature.
    /// Consider using safe constructors.
    pub unsafe fn from_raw(
        vertex_layout: Vec<(String, usize)>,
        render_target: Vec<String>,
        domain: Option<String>,
        middlewares: StringSequence,
    ) -> Self {
        let mesh = MaterialMeshSignature::from_raw(vertex_layout);
        let render_target = MaterialRenderTargetSignature::from_raw(render_target);
        Self::new(mesh, render_target, domain, middlewares)
    }

    pub fn hashed(&self) -> MaterialHashedSignature {
        let mut hasher = DefaultHasher::new();
        self.mesh.hash(&mut hasher);
        self.render_target.hash(&mut hasher);
        MaterialHashedSignature(hasher.finish())
    }

    pub fn sources(&self) -> impl Iterator<Item = (&str, usize)> {
        self.mesh.sources()
    }

    pub fn targets(&self) -> impl Iterator<Item = (&str, usize)> {
        self.render_target.targets()
    }

    pub fn domain(&self) -> Option<&str> {
        self.domain.as_deref()
    }

    pub fn middlewares(&self) -> StrSequence {
        self.middlewares.as_slice()
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MaterialHashedSignature(u64);

pub trait MaterialCompile<S, R, T>
where
    S: Write,
{
    fn compile_to(&self, output: &mut S, state: T) -> std::io::Result<()>;

    fn compile(&self, state: T) -> std::io::Result<R>
    where
        S: Default + Into<std::io::Result<R>>,
    {
        let mut buffer = S::default();
        self.compile_to(&mut buffer, state)?;
        buffer.into()
    }
}

pub enum MaterialCompilationState<'a> {
    FunctionDeclaration,
    FunctionDefinition {
        library: &'a MaterialLibrary,
    },
    FunctionBody {
        library: &'a MaterialLibrary,
    },
    Main {
        shader_type: MaterialShaderType,
        signature: &'a MaterialSignature,
        library: &'a MaterialLibrary,
        fragment_high_precision_support: bool,
    },
    GraphBody {
        shader_type: MaterialShaderType,
        library: &'a MaterialLibrary,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BakedMaterialShaders {
    pub vertex: String,
    pub fragment: String,
    pub uniforms: HashMap<String, MaterialValueType>,
    pub samplers: Vec<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaterialDataPrecision {
    Default,
    Low,
    Medium,
    High,
}

impl Default for MaterialDataPrecision {
    fn default() -> Self {
        Self::Default
    }
}

impl ToString for MaterialDataPrecision {
    fn to_string(&self) -> String {
        match self {
            Self::Default => "".to_owned(),
            Self::Low => "lowp".to_owned(),
            Self::Medium => "mediump".to_owned(),
            Self::High => "highp".to_owned(),
        }
    }
}

impl MaterialDataPrecision {
    pub fn ensure(self, fragment_high_precision_support: bool) -> Self {
        if self == Self::High && !fragment_high_precision_support {
            Self::Medium
        } else {
            self
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialDataType {
    BuiltIn,
    Constant,
    Attribute,
    Uniform,
    BufferOutput,
}

impl Default for MaterialDataType {
    fn default() -> Self {
        Self::Constant
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialValueCategory {
    Bool,
    Float,
    Integer,
    Sampler,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialValueType {
    Bool,
    Vec2B,
    Vec3B,
    Vec4B,
    Mat2B,
    Mat3B,
    Mat4B,
    Scalar,
    Vec2F,
    Vec3F,
    Vec4F,
    Mat2F,
    Mat3F,
    Mat4F,
    Integer,
    Vec2I,
    Vec3I,
    Vec4I,
    Mat2I,
    Mat3I,
    Mat4I,
    Sampler2d,
    Sampler2dArray,
    Sampler3d,
    Array(Box<MaterialValueType>, Option<usize>),
}

impl Default for MaterialValueType {
    fn default() -> Self {
        Self::Scalar
    }
}

impl MaterialValueType {
    pub fn category(&self) -> MaterialValueCategory {
        match self {
            Self::Bool
            | Self::Vec2B
            | Self::Vec3B
            | Self::Vec4B
            | Self::Mat2B
            | Self::Mat3B
            | Self::Mat4B => MaterialValueCategory::Bool,
            Self::Scalar
            | Self::Vec2F
            | Self::Vec3F
            | Self::Vec4F
            | Self::Mat2F
            | Self::Mat3F
            | Self::Mat4F => MaterialValueCategory::Float,
            Self::Integer
            | Self::Vec2I
            | Self::Vec3I
            | Self::Vec4I
            | Self::Mat2I
            | Self::Mat3I
            | Self::Mat4I => MaterialValueCategory::Integer,
            Self::Sampler2d | Self::Sampler2dArray | Self::Sampler3d => {
                MaterialValueCategory::Sampler
            }
            Self::Array(value_type, _) => value_type.category(),
        }
    }
}

impl ToString for MaterialValueType {
    fn to_string(&self) -> String {
        match self {
            Self::Bool => "bool".to_owned(),
            Self::Vec2B => "bvec2".to_owned(),
            Self::Vec3B => "bvec3".to_owned(),
            Self::Vec4B => "bvec4".to_owned(),
            Self::Mat2B => "bmat2".to_owned(),
            Self::Mat3B => "bmat3".to_owned(),
            Self::Mat4B => "bmat4".to_owned(),
            Self::Scalar => "float".to_owned(),
            Self::Vec2F => "vec2".to_owned(),
            Self::Vec3F => "vec3".to_owned(),
            Self::Vec4F => "vec4".to_owned(),
            Self::Mat2F => "mat2".to_owned(),
            Self::Mat3F => "mat3".to_owned(),
            Self::Mat4F => "mat4".to_owned(),
            Self::Integer => "int".to_owned(),
            Self::Vec2I => "ivec2".to_owned(),
            Self::Vec3I => "ivec3".to_owned(),
            Self::Vec4I => "ivec4".to_owned(),
            Self::Mat2I => "imat2".to_owned(),
            Self::Mat3I => "imat3".to_owned(),
            Self::Mat4I => "imat4".to_owned(),
            Self::Sampler2d => "sampler2D".to_owned(),
            Self::Sampler2dArray => "sampler2DArray".to_owned(),
            Self::Sampler3d => "sampler3D".to_owned(),
            Self::Array(t, c) => match c {
                Some(c) => format!("{}[{}]", t.to_string(), c),
                None => format!("{}[]", t.to_string()),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaterialValue {
    Bool(bool),
    Vec2B(Vec2<bool>),
    Vec3B(Vec3<bool>),
    Vec4B(Vec4<bool>),
    Mat2B(Mat2<bool>),
    Mat3B(Mat3<bool>),
    Mat4B(Mat4<bool>),
    Scalar(f32),
    Vec2F(Vec2<f32>),
    Vec3F(Vec3<f32>),
    Vec4F(Vec4<f32>),
    Mat2F(Mat2<f32>),
    Mat3F(Mat3<f32>),
    Mat4F(Mat4<f32>),
    Integer(i32),
    Vec2I(Vec2<i32>),
    Vec3I(Vec3<i32>),
    Vec4I(Vec4<i32>),
    Mat2I(Mat2<i32>),
    Mat3I(Mat3<i32>),
    Mat4I(Mat4<i32>),
    Sampler2d {
        reference: ImageReference,
        #[serde(default)]
        filtering: ImageFiltering,
    },
    Sampler2dArray {
        reference: ImageReference,
        #[serde(default)]
        filtering: ImageFiltering,
    },
    Sampler3d {
        reference: ImageReference,
        #[serde(default)]
        filtering: ImageFiltering,
    },
    Array(Vec<MaterialValue>),
}

impl MaterialValue {
    pub fn sampler_2d(reference: ImageReference) -> Self {
        Self::Sampler2d {
            reference,
            filtering: Default::default(),
        }
    }

    pub fn sampler_2d_filter(reference: ImageReference, filtering: ImageFiltering) -> Self {
        Self::Sampler2d {
            reference,
            filtering,
        }
    }

    pub fn sampler_2d_array(reference: ImageReference) -> Self {
        Self::Sampler2dArray {
            reference,
            filtering: Default::default(),
        }
    }

    pub fn sampler_2d_array_filter(reference: ImageReference, filtering: ImageFiltering) -> Self {
        Self::Sampler2dArray {
            reference,
            filtering,
        }
    }

    pub fn sampler_3d(reference: ImageReference) -> Self {
        Self::Sampler3d {
            reference,
            filtering: Default::default(),
        }
    }

    pub fn sampler_3d_filter(reference: ImageReference, filtering: ImageFiltering) -> Self {
        Self::Sampler3d {
            reference,
            filtering,
        }
    }

    pub fn value_type(&self) -> MaterialValueType {
        match self {
            Self::Bool(_) => MaterialValueType::Bool,
            Self::Vec2B(_) => MaterialValueType::Vec2B,
            Self::Vec3B(_) => MaterialValueType::Vec3B,
            Self::Vec4B(_) => MaterialValueType::Vec4B,
            Self::Mat2B(_) => MaterialValueType::Mat2B,
            Self::Mat3B(_) => MaterialValueType::Mat3B,
            Self::Mat4B(_) => MaterialValueType::Mat4B,
            Self::Scalar(_) => MaterialValueType::Scalar,
            Self::Vec2F(_) => MaterialValueType::Vec2F,
            Self::Vec3F(_) => MaterialValueType::Vec3F,
            Self::Vec4F(_) => MaterialValueType::Vec4F,
            Self::Mat2F(_) => MaterialValueType::Mat2F,
            Self::Mat3F(_) => MaterialValueType::Mat3F,
            Self::Mat4F(_) => MaterialValueType::Mat4F,
            Self::Integer(_) => MaterialValueType::Integer,
            Self::Vec2I(_) => MaterialValueType::Vec2I,
            Self::Vec3I(_) => MaterialValueType::Vec3I,
            Self::Vec4I(_) => MaterialValueType::Vec4I,
            Self::Mat2I(_) => MaterialValueType::Mat2I,
            Self::Mat3I(_) => MaterialValueType::Mat3I,
            Self::Mat4I(_) => MaterialValueType::Mat4I,
            Self::Sampler2d { .. } => MaterialValueType::Sampler2d,
            Self::Sampler2dArray { .. } => MaterialValueType::Sampler2dArray,
            Self::Sampler3d { .. } => MaterialValueType::Sampler3d,
            Self::Array(data) => MaterialValueType::Array(
                Box::new(if let Some(item) = data.first() {
                    item.value_type()
                } else {
                    Default::default()
                }),
                Some(data.len()),
            ),
        }
    }

    pub fn update_references(&mut self, image_mapping: &ImageResourceMapping) {
        match self {
            Self::Sampler2d { reference, .. }
            | Self::Sampler2dArray { reference, .. }
            | Self::Sampler3d { reference, .. } => match reference {
                ImageReference::Asset(path) => {
                    if let Some(id) = image_mapping.resource_by_name(path) {
                        *reference = ImageReference::Id(id);
                    }
                }
                ImageReference::VirtualAsset(path) => {
                    if let Some((owner, id)) = image_mapping.virtual_resource_by_name(path) {
                        *reference = ImageReference::VirtualId { owner, id };
                    }
                }
                _ => {}
            },
            Self::Array(data) => {
                for value in data {
                    value.update_references(image_mapping);
                }
            }
            _ => {}
        }
    }
}

impl ToString for MaterialValue {
    #[allow(clippy::many_single_char_names)]
    fn to_string(&self) -> String {
        match self {
            Self::Bool(v) => v.to_string(),
            Self::Vec2B(v) => format!("bvec2({}, {})", v.x, v.y),
            Self::Vec3B(v) => format!("bvec3({}, {}, {})", v.x, v.y, v.z),
            Self::Vec4B(v) => format!("bvec4({}, {}, {}, {})", v.x, v.y, v.z, v.w),
            Self::Mat2B(v) => {
                let [a, b, c, d] = v.into_col_array();
                format!("bmat2({}, {}, {}, {})", a, b, c, d)
            }
            Self::Mat3B(v) => {
                let [a, b, c, d, e, f, g, h, i] = v.into_col_array();
                format!(
                    "bmat3({}, {}, {}, {}, {}, {}, {}, {}, {})",
                    a, b, c, d, e, f, g, h, i
                )
            }
            Self::Mat4B(v) => {
                let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = v.into_col_array();
                format!(
                    "bmat4({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                    a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p
                )
            }
            Self::Scalar(v) => format!("{:.6}", v),
            Self::Vec2F(v) => format!("vec2({:.6}, {:.6})", v.x, v.y),
            Self::Vec3F(v) => format!("vec3({:.6}, {:.6}, {:.6})", v.x, v.y, v.z),
            Self::Vec4F(v) => format!("vec4({:.6}, {:.6}, {:.6}, {:.6})", v.x, v.y, v.z, v.w),
            Self::Mat2F(v) => {
                let [a, b, c, d] = v.into_col_array();
                format!("mat2({:.6}, {:.6}, {:.6}, {:.6})", a, b, c, d)
            }
            Self::Mat3F(v) => {
                let [a, b, c, d, e, f, g, h, i] = v.into_col_array();
                format!(
                    "mat3({:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6})",
                    a, b, c, d, e, f, g, h, i
                )
            }
            Self::Mat4F(v) => {
                let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = v.into_col_array();
                format!(
                    "mat4({:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6})",
                    a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p
                )
            }
            Self::Integer(v) => format!("{}", v),
            Self::Vec2I(v) => format!("ivec2({}, {})", v.x, v.y),
            Self::Vec3I(v) => format!("ivec3({}, {}, {})", v.x, v.y, v.z),
            Self::Vec4I(v) => format!("ivec4({}, {}, {}, {})", v.x, v.y, v.z, v.w),
            Self::Mat2I(v) => {
                let [a, b, c, d] = v.into_col_array();
                format!("imat2({}, {}, {}, {})", a, b, c, d)
            }
            Self::Mat3I(v) => {
                let [a, b, c, d, e, f, g, h, i] = v.into_col_array();
                format!(
                    "imat3({}, {}, {}, {}, {}, {}, {}, {}, {})",
                    a, b, c, d, e, f, g, h, i
                )
            }
            Self::Mat4I(v) => {
                let [a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p] = v.into_col_array();
                format!(
                    "imat4({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                    a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p
                )
            }
            Self::Sampler2d { reference, .. }
            | Self::Sampler2dArray { reference, .. }
            | Self::Sampler3d { reference, .. } => reference.to_string(),
            Self::Array(v) => {
                let t = if let Some(item) = v.first() {
                    item.value_type()
                } else {
                    Default::default()
                };
                format!(
                    "{}[]({})",
                    t.to_string(),
                    v.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

macro_rules! impl_value_from {
    ( $( $name:ident ( $type:ty ) , )+ ) => {
        $(
            impl From<$type> for MaterialValue {
                fn from(v: $type) -> Self {
                    Self::$name(v.into())
                }
            }
        )+
    };
}

impl_value_from! {
    Bool(bool),
    Vec2B(Vec2<bool>),
    Vec3B(Vec3<bool>),
    Vec4B(Vec4<bool>),
    Mat2B(Mat2<bool>),
    Mat3B(Mat3<bool>),
    Mat4B(Mat4<bool>),
    Scalar(f32),
    Vec2F(Vec2<f32>),
    Vec3F(Vec3<f32>),
    Vec4F(Vec4<f32>),
    Mat2F(Mat2<f32>),
    Mat3F(Mat3<f32>),
    Mat4F(Mat4<f32>),
    Integer(i32),
    Vec2I(Vec2<i32>),
    Vec3I(Vec3<i32>),
    Vec4I(Vec4<i32>),
    Mat2I(Mat2<i32>),
    Mat3I(Mat3<i32>),
    Mat4I(Mat4<i32>),
    Array(Vec<MaterialValue>),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MaterialShaderType {
    Undefined,
    Vertex,
    Fragment,
}

impl Default for MaterialShaderType {
    fn default() -> Self {
        Self::Undefined
    }
}
