use crate::mesh::{
    vertex_factory::StaticVertexFactory, MeshDrawMode, MeshError, VertexType, VertexValueType,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vek::*;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum GeometryValueType {
    Bool,
    String,
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
}

impl GeometryValueType {
    pub fn as_vertex_value(&self) -> Option<VertexValueType> {
        match self {
            Self::Bool | Self::String => None,
            Self::Scalar => Some(VertexValueType::Scalar),
            Self::Vec2F => Some(VertexValueType::Vec2F),
            Self::Vec3F => Some(VertexValueType::Vec3F),
            Self::Vec4F => Some(VertexValueType::Vec4F),
            Self::Mat2F => Some(VertexValueType::Mat2F),
            Self::Mat3F => Some(VertexValueType::Mat3F),
            Self::Mat4F => Some(VertexValueType::Mat4F),
            Self::Integer => Some(VertexValueType::Integer),
            Self::Vec2I => Some(VertexValueType::Vec2I),
            Self::Vec3I => Some(VertexValueType::Vec3I),
            Self::Vec4I => Some(VertexValueType::Vec4I),
            Self::Mat2I => Some(VertexValueType::Mat2I),
            Self::Mat3I => Some(VertexValueType::Mat3I),
            Self::Mat4I => Some(VertexValueType::Mat4I),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GeometryValue {
    Bool(bool),
    String(String),
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
}

impl GeometryValue {
    pub fn as_value_type(&self) -> GeometryValueType {
        match self {
            Self::Bool(_) => GeometryValueType::Bool,
            Self::String(_) => GeometryValueType::String,
            Self::Scalar(_) => GeometryValueType::Scalar,
            Self::Vec2F(_) => GeometryValueType::Vec2F,
            Self::Vec3F(_) => GeometryValueType::Vec3F,
            Self::Vec4F(_) => GeometryValueType::Vec4F,
            Self::Mat2F(_) => GeometryValueType::Mat2F,
            Self::Mat3F(_) => GeometryValueType::Mat3F,
            Self::Mat4F(_) => GeometryValueType::Mat4F,
            Self::Integer(_) => GeometryValueType::Integer,
            Self::Vec2I(_) => GeometryValueType::Vec2I,
            Self::Vec3I(_) => GeometryValueType::Vec3I,
            Self::Vec4I(_) => GeometryValueType::Vec4I,
            Self::Mat2I(_) => GeometryValueType::Mat2I,
            Self::Mat3I(_) => GeometryValueType::Mat3I,
            Self::Mat4I(_) => GeometryValueType::Mat4I,
        }
    }
}

impl ToString for GeometryValue {
    fn to_string(&self) -> String {
        match self {
            Self::Bool(value) => value.to_string(),
            Self::String(value) => value.to_string(),
            Self::Scalar(value) => value.to_string(),
            Self::Vec2F(value) => value.to_string(),
            Self::Vec3F(value) => value.to_string(),
            Self::Vec4F(value) => value.to_string(),
            Self::Mat2F(value) => value.to_string(),
            Self::Mat3F(value) => value.to_string(),
            Self::Mat4F(value) => value.to_string(),
            Self::Integer(value) => value.to_string(),
            Self::Vec2I(value) => value.to_string(),
            Self::Vec3I(value) => value.to_string(),
            Self::Vec4I(value) => value.to_string(),
            Self::Mat2I(value) => value.to_string(),
            Self::Mat3I(value) => value.to_string(),
            Self::Mat4I(value) => value.to_string(),
        }
    }
}

macro_rules! impl_value_from {
    (@copy $type:ty => $variant:ident) => {
        impl From<$type> for GeometryValue {
            fn from(value: $type) -> Self {
                Self::$variant(value)
            }
        }

        impl From<&$type> for GeometryValue {
            fn from(value: &$type) -> Self {
                Self::$variant(*value)
            }
        }
    };
    (@ownedref $type:ty => $variant:ident) => {
        impl From<&$type> for GeometryValue {
            fn from(value: &$type) -> Self {
                Self::$variant(value.to_owned())
            }
        }
    };
    (@owned $type:ty => $variant:ident) => {
        impl From<$type> for GeometryValue {
            fn from(value: $type) -> Self {
                Self::$variant(value)
            }
        }
    };
}

macro_rules! impl_value_into {
    (@copy $type:ty => $($variant:ident),+) => {
        impl TryInto<$type> for &GeometryValue {
            type Error = MeshError;

            fn try_into(self) -> Result<$type, Self::Error> {
                match self {
                    $(
                        GeometryValue::$variant(result) => Ok(<$type>::from(*result)),
                    )+
                    _ => Err(MeshError::UnsupportedGeometryValueConversionType(std::any::type_name::<$type>().to_owned())),
                }
            }
        }

        impl TryInto<$type> for GeometryValue {
            type Error = MeshError;

            fn try_into(self) -> Result<$type, Self::Error> {
                match self {
                    $(
                        GeometryValue::$variant(result) => Ok(<$type>::from(result)),
                    )+
                    _ => Err(MeshError::UnsupportedGeometryValueConversionType(std::any::type_name::<$type>().to_owned())),
                }
            }
        }
    };
    (@owned $type:ty => $variant:ident) => {
        impl TryInto<$type> for &GeometryValue {
            type Error = MeshError;

            fn try_into(self) -> Result<$type, Self::Error> {
                match self {
                    GeometryValue::$variant(result) => Ok(result.to_owned()),
                    _ => Err(MeshError::UnsupportedGeometryValueConversionType(std::any::type_name::<$type>().to_owned())),
                }
            }
        }
        impl TryInto<$type> for GeometryValue {
            type Error = MeshError;

            fn try_into(self) -> Result<$type, Self::Error> {
                match self {
                    GeometryValue::$variant(result) => Ok(result),
                    _ => Err(MeshError::UnsupportedGeometryValueConversionType(std::any::type_name::<$type>().to_owned())),
                }
            }
        }
    };
}

impl_value_from!(@copy bool => Bool);
impl_value_from!(@ownedref str => String);
impl_value_from!(@ownedref String => String);
impl_value_from!(@owned String => String);
impl_value_from!(@copy f32 => Scalar);
impl_value_from!(@copy Vec2<f32> => Vec2F);
impl_value_from!(@copy Vec3<f32> => Vec3F);
impl_value_from!(@copy Vec4<f32> => Vec4F);
impl_value_from!(@copy Mat2<f32> => Mat2F);
impl_value_from!(@copy Mat3<f32> => Mat3F);
impl_value_from!(@copy Mat4<f32> => Mat4F);
impl_value_from!(@copy i32 => Integer);
impl_value_from!(@copy Vec2<i32> => Vec2I);
impl_value_from!(@copy Vec3<i32> => Vec3I);
impl_value_from!(@copy Vec4<i32> => Vec4I);
impl_value_from!(@copy Mat2<i32> => Mat2I);
impl_value_from!(@copy Mat3<i32> => Mat3I);
impl_value_from!(@copy Mat4<i32> => Mat4I);

impl_value_into!(@copy bool => Bool);
impl_value_into!(@owned String => String);
impl_value_into!(@copy f32 => Scalar);
impl_value_into!(@copy Vec2<f32> => Vec2F, Vec3F, Vec4F);
impl_value_into!(@copy Vec3<f32> => Vec2F, Vec3F, Vec4F);
impl_value_into!(@copy Vec4<f32> => Vec2F, Vec3F, Vec4F);
impl_value_into!(@copy Mat2<f32> => Mat2F, Mat3F, Mat4F);
impl_value_into!(@copy Mat3<f32> => Mat2F, Mat3F, Mat4F);
impl_value_into!(@copy Mat4<f32> => Mat2F, Mat3F, Mat4F);
impl_value_into!(@copy i32 => Integer);
impl_value_into!(@copy Vec2<i32> => Vec2I, Vec3I, Vec4I);
impl_value_into!(@copy Vec3<i32> => Vec2I, Vec3I, Vec4I);
impl_value_into!(@copy Vec4<i32> => Vec2I, Vec3I, Vec4I);
impl_value_into!(@copy Mat2<i32> => Mat2I, Mat3I, Mat4I);
impl_value_into!(@copy Mat3<i32> => Mat2I, Mat3I, Mat4I);
impl_value_into!(@copy Mat4<i32> => Mat2I, Mat3I, Mat4I);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GeometryValues {
    Bool(Vec<bool>),
    String(Vec<String>),
    Scalar(Vec<f32>),
    Vec2F(Vec<Vec2<f32>>),
    Vec3F(Vec<Vec3<f32>>),
    Vec4F(Vec<Vec4<f32>>),
    Mat2F(Vec<Mat2<f32>>),
    Mat3F(Vec<Mat3<f32>>),
    Mat4F(Vec<Mat4<f32>>),
    Integer(Vec<i32>),
    Vec2I(Vec<Vec2<i32>>),
    Vec3I(Vec<Vec3<i32>>),
    Vec4I(Vec<Vec4<i32>>),
    Mat2I(Vec<Mat2<i32>>),
    Mat3I(Vec<Mat3<i32>>),
    Mat4I(Vec<Mat4<i32>>),
}

impl GeometryValues {
    pub fn as_value_type(&self) -> GeometryValueType {
        match self {
            Self::Bool(_) => GeometryValueType::Bool,
            Self::String(_) => GeometryValueType::String,
            Self::Scalar(_) => GeometryValueType::Scalar,
            Self::Vec2F(_) => GeometryValueType::Vec2F,
            Self::Vec3F(_) => GeometryValueType::Vec3F,
            Self::Vec4F(_) => GeometryValueType::Vec4F,
            Self::Mat2F(_) => GeometryValueType::Mat2F,
            Self::Mat3F(_) => GeometryValueType::Mat3F,
            Self::Mat4F(_) => GeometryValueType::Mat4F,
            Self::Integer(_) => GeometryValueType::Integer,
            Self::Vec2I(_) => GeometryValueType::Vec2I,
            Self::Vec3I(_) => GeometryValueType::Vec3I,
            Self::Vec4I(_) => GeometryValueType::Vec4I,
            Self::Mat2I(_) => GeometryValueType::Mat2I,
            Self::Mat3I(_) => GeometryValueType::Mat3I,
            Self::Mat4I(_) => GeometryValueType::Mat4I,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Bool(items) => items.len(),
            Self::String(items) => items.len(),
            Self::Scalar(items) => items.len(),
            Self::Vec2F(items) => items.len(),
            Self::Vec3F(items) => items.len(),
            Self::Vec4F(items) => items.len(),
            Self::Mat2F(items) => items.len(),
            Self::Mat3F(items) => items.len(),
            Self::Mat4F(items) => items.len(),
            Self::Integer(items) => items.len(),
            Self::Vec2I(items) => items.len(),
            Self::Vec3I(items) => items.len(),
            Self::Vec4I(items) => items.len(),
            Self::Mat2I(items) => items.len(),
            Self::Mat3I(items) => items.len(),
            Self::Mat4I(items) => items.len(),
        }
    }

    pub fn get(&self, index: usize) -> Option<GeometryValue> {
        match self {
            Self::Bool(items) => items.get(index).map(|value| value.into()),
            Self::String(items) => items.get(index).map(|value| value.into()),
            Self::Scalar(items) => items.get(index).map(|value| value.into()),
            Self::Vec2F(items) => items.get(index).map(|value| value.into()),
            Self::Vec3F(items) => items.get(index).map(|value| value.into()),
            Self::Vec4F(items) => items.get(index).map(|value| value.into()),
            Self::Mat2F(items) => items.get(index).map(|value| value.into()),
            Self::Mat3F(items) => items.get(index).map(|value| value.into()),
            Self::Mat4F(items) => items.get(index).map(|value| value.into()),
            Self::Integer(items) => items.get(index).map(|value| value.into()),
            Self::Vec2I(items) => items.get(index).map(|value| value.into()),
            Self::Vec3I(items) => items.get(index).map(|value| value.into()),
            Self::Vec4I(items) => items.get(index).map(|value| value.into()),
            Self::Mat2I(items) => items.get(index).map(|value| value.into()),
            Self::Mat3I(items) => items.get(index).map(|value| value.into()),
            Self::Mat4I(items) => items.get(index).map(|value| value.into()),
        }
    }

    pub fn iter(&self) -> GeometryValuesIter {
        match self {
            Self::Bool(items) => GeometryValuesIter::Bool(items.iter()),
            Self::String(items) => GeometryValuesIter::String(items.iter()),
            Self::Scalar(items) => GeometryValuesIter::Scalar(items.iter()),
            Self::Vec2F(items) => GeometryValuesIter::Vec2F(items.iter()),
            Self::Vec3F(items) => GeometryValuesIter::Vec3F(items.iter()),
            Self::Vec4F(items) => GeometryValuesIter::Vec4F(items.iter()),
            Self::Mat2F(items) => GeometryValuesIter::Mat2F(items.iter()),
            Self::Mat3F(items) => GeometryValuesIter::Mat3F(items.iter()),
            Self::Mat4F(items) => GeometryValuesIter::Mat4F(items.iter()),
            Self::Integer(items) => GeometryValuesIter::Integer(items.iter()),
            Self::Vec2I(items) => GeometryValuesIter::Vec2I(items.iter()),
            Self::Vec3I(items) => GeometryValuesIter::Vec3I(items.iter()),
            Self::Vec4I(items) => GeometryValuesIter::Vec4I(items.iter()),
            Self::Mat2I(items) => GeometryValuesIter::Mat2I(items.iter()),
            Self::Mat3I(items) => GeometryValuesIter::Mat3I(items.iter()),
            Self::Mat4I(items) => GeometryValuesIter::Mat4I(items.iter()),
        }
    }
}

pub enum GeometryValuesIter<'a> {
    Bool(std::slice::Iter<'a, bool>),
    String(std::slice::Iter<'a, String>),
    Scalar(std::slice::Iter<'a, f32>),
    Vec2F(std::slice::Iter<'a, Vec2<f32>>),
    Vec3F(std::slice::Iter<'a, Vec3<f32>>),
    Vec4F(std::slice::Iter<'a, Vec4<f32>>),
    Mat2F(std::slice::Iter<'a, Mat2<f32>>),
    Mat3F(std::slice::Iter<'a, Mat3<f32>>),
    Mat4F(std::slice::Iter<'a, Mat4<f32>>),
    Integer(std::slice::Iter<'a, i32>),
    Vec2I(std::slice::Iter<'a, Vec2<i32>>),
    Vec3I(std::slice::Iter<'a, Vec3<i32>>),
    Vec4I(std::slice::Iter<'a, Vec4<i32>>),
    Mat2I(std::slice::Iter<'a, Mat2<i32>>),
    Mat3I(std::slice::Iter<'a, Mat3<i32>>),
    Mat4I(std::slice::Iter<'a, Mat4<i32>>),
}

impl<'a> Iterator for GeometryValuesIter<'a> {
    type Item = GeometryValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Bool(iter) => iter.next().map(|value| value.into()),
            Self::String(iter) => iter.next().map(|value| value.into()),
            Self::Scalar(iter) => iter.next().map(|value| value.into()),
            Self::Vec2F(iter) => iter.next().map(|value| value.into()),
            Self::Vec3F(iter) => iter.next().map(|value| value.into()),
            Self::Vec4F(iter) => iter.next().map(|value| value.into()),
            Self::Mat2F(iter) => iter.next().map(|value| value.into()),
            Self::Mat3F(iter) => iter.next().map(|value| value.into()),
            Self::Mat4F(iter) => iter.next().map(|value| value.into()),
            Self::Integer(iter) => iter.next().map(|value| value.into()),
            Self::Vec2I(iter) => iter.next().map(|value| value.into()),
            Self::Vec3I(iter) => iter.next().map(|value| value.into()),
            Self::Vec4I(iter) => iter.next().map(|value| value.into()),
            Self::Mat2I(iter) => iter.next().map(|value| value.into()),
            Self::Mat3I(iter) => iter.next().map(|value| value.into()),
            Self::Mat4I(iter) => iter.next().map(|value| value.into()),
        }
    }
}

macro_rules! impl_values_from {
    ($type:ty => $variant:ident) => {
        impl FromIterator<$type> for GeometryValues {
            fn from_iter<I: IntoIterator<Item = $type>>(iter: I) -> Self {
                Self::$variant(iter.into_iter().collect())
            }
        }
    };
}

impl_values_from!(bool => Bool);
impl_values_from!(String => String);
impl_values_from!(f32 => Scalar);
impl_values_from!(Vec2<f32> => Vec2F);
impl_values_from!(Vec3<f32> => Vec3F);
impl_values_from!(Vec4<f32> => Vec4F);
impl_values_from!(Mat2<f32> => Mat2F);
impl_values_from!(Mat3<f32> => Mat3F);
impl_values_from!(Mat4<f32> => Mat4F);
impl_values_from!(i32 => Integer);
impl_values_from!(Vec2<i32> => Vec2I);
impl_values_from!(Vec3<i32> => Vec3I);
impl_values_from!(Vec4<i32> => Vec4I);
impl_values_from!(Mat2<i32> => Mat2I);
impl_values_from!(Mat3<i32> => Mat3I);
impl_values_from!(Mat4<i32> => Mat4I);

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryAttributes {
    items: BTreeMap<String, GeometryValue>,
}

impl GeometryAttributes {
    pub fn with(mut self, name: impl ToString, value: impl Into<GeometryValue>) -> Self {
        self.set(name, value);
        self
    }

    pub fn get(&self, name: &str) -> Option<GeometryValue> {
        self.items.get(name).cloned()
    }

    pub fn set(
        &mut self,
        name: impl ToString,
        value: impl Into<GeometryValue>,
    ) -> Option<GeometryValue> {
        self.items.insert(name.to_string(), value.into())
    }

    pub fn delete(&mut self, name: &str) -> Option<GeometryValue> {
        self.items.remove(name)
    }

    pub fn has(&self, name: &str) -> bool {
        self.items.contains_key(name)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &GeometryValue)> {
        self.items.iter().map(|(key, value)| (key.as_str(), value))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&str, &mut GeometryValue)> {
        self.items
            .iter_mut()
            .map(|(key, value)| (key.as_str(), value))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryTriangle {
    pub indices: [usize; 3],
    pub attributes: GeometryAttributes,
}

impl GeometryTriangle {
    pub fn new(indices: [usize; 3]) -> Self {
        Self {
            indices,
            attributes: Default::default(),
        }
    }

    pub fn with_attribute(mut self, name: impl ToString, value: GeometryValue) -> Self {
        self.attributes.set(name, value);
        self
    }
}

impl From<[usize; 3]> for GeometryTriangle {
    fn from(indices: [usize; 3]) -> Self {
        Self::new(indices)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryTriangles {
    pub items: Vec<GeometryTriangle>,
    pub attributes: GeometryAttributes,
}

impl GeometryTriangles {
    pub fn with(mut self, triangle: impl Into<GeometryTriangle>) -> Self {
        self.items.push(triangle.into());
        self
    }

    pub fn with_attribute(mut self, name: impl ToString, value: GeometryValue) -> Self {
        self.attributes.set(name, value);
        self
    }
}

impl From<()> for GeometryTriangles {
    fn from(_: ()) -> Self {
        Self::default()
    }
}

impl From<&[GeometryTriangle]> for GeometryTriangles {
    fn from(items: &[GeometryTriangle]) -> Self {
        items.into_iter().cloned().collect()
    }
}

impl From<Vec<GeometryTriangle>> for GeometryTriangles {
    fn from(items: Vec<GeometryTriangle>) -> Self {
        Self {
            items,
            attributes: Default::default(),
        }
    }
}

impl FromIterator<GeometryTriangle> for GeometryTriangles {
    fn from_iter<I: IntoIterator<Item = GeometryTriangle>>(iter: I) -> Self {
        Self {
            items: iter.into_iter().collect(),
            attributes: Default::default(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryLine {
    pub indices: [usize; 2],
    pub attributes: GeometryAttributes,
}

impl GeometryLine {
    pub fn new(indices: [usize; 2]) -> Self {
        Self {
            indices,
            attributes: Default::default(),
        }
    }

    pub fn with_attribute(mut self, name: impl ToString, value: GeometryValue) -> Self {
        self.attributes.set(name, value);
        self
    }
}

impl From<[usize; 2]> for GeometryLine {
    fn from(indices: [usize; 2]) -> Self {
        Self::new(indices)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryLines {
    pub items: Vec<GeometryLine>,
    pub attributes: GeometryAttributes,
}

impl GeometryLines {
    pub fn with(mut self, line: impl Into<GeometryLine>) -> Self {
        self.items.push(line.into());
        self
    }

    pub fn with_attribute(mut self, name: impl ToString, value: GeometryValue) -> Self {
        self.attributes.set(name, value);
        self
    }
}

impl From<()> for GeometryLines {
    fn from(_: ()) -> Self {
        Self::default()
    }
}

impl From<&[GeometryLine]> for GeometryLines {
    fn from(items: &[GeometryLine]) -> Self {
        items.into_iter().cloned().collect()
    }
}

impl From<Vec<GeometryLine>> for GeometryLines {
    fn from(items: Vec<GeometryLine>) -> Self {
        Self {
            items,
            attributes: Default::default(),
        }
    }
}

impl FromIterator<GeometryLine> for GeometryLines {
    fn from_iter<I: IntoIterator<Item = GeometryLine>>(iter: I) -> Self {
        Self {
            items: iter.into_iter().collect(),
            attributes: Default::default(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryPoint {
    pub index: usize,
    pub attributes: GeometryAttributes,
}

impl GeometryPoint {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            attributes: Default::default(),
        }
    }

    pub fn with_attribute(mut self, name: impl ToString, value: GeometryValue) -> Self {
        self.attributes.set(name, value);
        self
    }
}

impl From<usize> for GeometryPoint {
    fn from(index: usize) -> Self {
        Self::new(index)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryPoints {
    pub items: Vec<GeometryPoint>,
    pub attributes: GeometryAttributes,
}

impl GeometryPoints {
    pub fn with(mut self, point: impl Into<GeometryPoint>) -> Self {
        self.items.push(point.into());
        self
    }

    pub fn with_attribute(mut self, name: impl ToString, value: GeometryValue) -> Self {
        self.attributes.set(name, value);
        self
    }
}

impl From<()> for GeometryPoints {
    fn from(_: ()) -> Self {
        Self::default()
    }
}

impl From<&[GeometryPoint]> for GeometryPoints {
    fn from(items: &[GeometryPoint]) -> Self {
        items.into_iter().cloned().collect()
    }
}

impl From<Vec<GeometryPoint>> for GeometryPoints {
    fn from(items: Vec<GeometryPoint>) -> Self {
        Self {
            items,
            attributes: Default::default(),
        }
    }
}

impl FromIterator<GeometryPoint> for GeometryPoints {
    fn from_iter<I: IntoIterator<Item = GeometryPoint>>(iter: I) -> Self {
        Self {
            items: iter.into_iter().collect(),
            attributes: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GeometryPrimitives {
    Triangles(GeometryTriangles),
    Lines(GeometryLines),
    Points(GeometryPoints),
}

impl GeometryPrimitives {
    pub fn triangles(triangles: impl Into<GeometryTriangles>) -> Self {
        Self::Triangles(triangles.into())
    }

    pub fn lines(lines: impl Into<GeometryLines>) -> Self {
        Self::Lines(lines.into())
    }

    pub fn points(points: impl Into<GeometryPoints>) -> Self {
        Self::Points(points.into())
    }

    pub fn as_triangles(&self) -> Result<&GeometryTriangles, MeshError> {
        match self {
            Self::Triangles(result) => Ok(result),
            _ => Err(MeshError::GeometryIsNotTriangles),
        }
    }

    pub fn as_lines(&self) -> Result<&GeometryLines, MeshError> {
        match self {
            Self::Lines(result) => Ok(result),
            _ => Err(MeshError::GeometryIsNotTriangles),
        }
    }

    pub fn as_points(&self) -> Result<&GeometryPoints, MeshError> {
        match self {
            Self::Points(result) => Ok(result),
            _ => Err(MeshError::GeometryIsNotTriangles),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryVerticesColumn {
    pub name: String,
    pub values: GeometryValues,
}

impl GeometryVerticesColumn {
    pub fn new(name: impl ToString, values: GeometryValues) -> Self {
        Self {
            name: name.to_string(),
            values,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryVertices {
    columns: Vec<GeometryVerticesColumn>,
    count: usize,
    pub attributes: GeometryAttributes,
}

impl GeometryVertices {
    pub fn new(columns: &[(&str, GeometryValueType)]) -> Result<Self, MeshError> {
        let mut result = Self::default();
        for (name, value_type) in columns {
            result.ensure_column(name, *value_type)?;
        }
        Ok(result)
    }

    pub fn with_attribute(mut self, name: impl ToString, value: GeometryValue) -> Self {
        self.attributes.set(name, value);
        self
    }

    pub fn with(mut self, attributes: GeometryAttributes) -> Result<Self, MeshError> {
        self.push(attributes)?;
        Ok(self)
    }

    pub fn with_columns(
        mut self,
        columns: impl IntoIterator<Item = GeometryVerticesColumn>,
    ) -> Result<Self, MeshError> {
        self.push_columns(columns)?;
        Ok(self)
    }

    pub fn ensure_column(
        &mut self,
        name: &str,
        value_type: GeometryValueType,
    ) -> Result<(), MeshError> {
        if let Some(column) = self.columns.iter().find(|column| column.name == name) {
            if column.values.as_value_type() != value_type {
                return Err(MeshError::GeometryValueTypeMismatch(
                    value_type,
                    column.values.as_value_type(),
                ));
            }
        } else {
            self.columns.push(GeometryVerticesColumn {
                name: name.to_owned(),
                values: match value_type {
                    GeometryValueType::Bool => {
                        GeometryValues::Bool(vec![Default::default(); self.count])
                    }
                    GeometryValueType::String => {
                        GeometryValues::String(vec![Default::default(); self.count])
                    }
                    GeometryValueType::Scalar => {
                        GeometryValues::Scalar(vec![Default::default(); self.count])
                    }
                    GeometryValueType::Vec2F => {
                        GeometryValues::Vec2F(vec![Default::default(); self.count])
                    }
                    GeometryValueType::Vec3F => {
                        GeometryValues::Vec3F(vec![Default::default(); self.count])
                    }
                    GeometryValueType::Vec4F => {
                        GeometryValues::Vec4F(vec![Default::default(); self.count])
                    }
                    GeometryValueType::Mat2F => {
                        GeometryValues::Mat2F(vec![Default::default(); self.count])
                    }
                    GeometryValueType::Mat3F => {
                        GeometryValues::Mat3F(vec![Default::default(); self.count])
                    }
                    GeometryValueType::Mat4F => {
                        GeometryValues::Mat4F(vec![Default::default(); self.count])
                    }
                    GeometryValueType::Integer => {
                        GeometryValues::Integer(vec![Default::default(); self.count])
                    }
                    GeometryValueType::Vec2I => {
                        GeometryValues::Vec2I(vec![Default::default(); self.count])
                    }
                    GeometryValueType::Vec3I => {
                        GeometryValues::Vec3I(vec![Default::default(); self.count])
                    }
                    GeometryValueType::Vec4I => {
                        GeometryValues::Vec4I(vec![Default::default(); self.count])
                    }
                    GeometryValueType::Mat2I => {
                        GeometryValues::Mat2I(vec![Default::default(); self.count])
                    }
                    GeometryValueType::Mat3I => {
                        GeometryValues::Mat3I(vec![Default::default(); self.count])
                    }
                    GeometryValueType::Mat4I => {
                        GeometryValues::Mat4I(vec![Default::default(); self.count])
                    }
                },
            });
        }
        Ok(())
    }

    pub fn ensure_columns(&mut self, attributes: &GeometryAttributes) -> Result<(), MeshError> {
        for (key, value) in attributes.iter() {
            self.ensure_column(key, value.as_value_type())?;
        }
        Ok(())
    }

    pub fn reserve(&mut self, additional: usize) {
        for column in &mut self.columns {
            match &mut column.values {
                GeometryValues::Bool(items) => items.reserve(additional),
                GeometryValues::String(items) => items.reserve(additional),
                GeometryValues::Scalar(items) => items.reserve(additional),
                GeometryValues::Vec2F(items) => items.reserve(additional),
                GeometryValues::Vec3F(items) => items.reserve(additional),
                GeometryValues::Vec4F(items) => items.reserve(additional),
                GeometryValues::Mat2F(items) => items.reserve(additional),
                GeometryValues::Mat3F(items) => items.reserve(additional),
                GeometryValues::Mat4F(items) => items.reserve(additional),
                GeometryValues::Integer(items) => items.reserve(additional),
                GeometryValues::Vec2I(items) => items.reserve(additional),
                GeometryValues::Vec3I(items) => items.reserve(additional),
                GeometryValues::Vec4I(items) => items.reserve(additional),
                GeometryValues::Mat2I(items) => items.reserve(additional),
                GeometryValues::Mat3I(items) => items.reserve(additional),
                GeometryValues::Mat4I(items) => items.reserve(additional),
            }
        }
    }

    pub fn ensure_len(&mut self, count: usize) {
        for column in &mut self.columns {
            match &mut column.values {
                GeometryValues::Bool(items) => items.extend(
                    std::iter::repeat(bool::default()).take(count.saturating_sub(items.len())),
                ),
                GeometryValues::String(items) => items.extend(
                    std::iter::repeat(String::default()).take(count.saturating_sub(items.len())),
                ),
                GeometryValues::Scalar(items) => items.extend(
                    std::iter::repeat(f32::default()).take(count.saturating_sub(items.len())),
                ),
                GeometryValues::Vec2F(items) => items.extend(
                    std::iter::repeat(Vec2::<f32>::default())
                        .take(count.saturating_sub(items.len())),
                ),
                GeometryValues::Vec3F(items) => items.extend(
                    std::iter::repeat(Vec3::<f32>::default())
                        .take(count.saturating_sub(items.len())),
                ),
                GeometryValues::Vec4F(items) => items.extend(
                    std::iter::repeat(Vec4::<f32>::default())
                        .take(count.saturating_sub(items.len())),
                ),
                GeometryValues::Mat2F(items) => items.extend(
                    std::iter::repeat(Mat2::<f32>::default())
                        .take(count.saturating_sub(items.len())),
                ),
                GeometryValues::Mat3F(items) => items.extend(
                    std::iter::repeat(Mat3::<f32>::default())
                        .take(count.saturating_sub(items.len())),
                ),
                GeometryValues::Mat4F(items) => items.extend(
                    std::iter::repeat(Mat4::<f32>::default())
                        .take(count.saturating_sub(items.len())),
                ),
                GeometryValues::Integer(items) => items.extend(
                    std::iter::repeat(i32::default()).take(count.saturating_sub(items.len())),
                ),
                GeometryValues::Vec2I(items) => items.extend(
                    std::iter::repeat(Vec2::<i32>::default())
                        .take(count.saturating_sub(items.len())),
                ),
                GeometryValues::Vec3I(items) => items.extend(
                    std::iter::repeat(Vec3::<i32>::default())
                        .take(count.saturating_sub(items.len())),
                ),
                GeometryValues::Vec4I(items) => items.extend(
                    std::iter::repeat(Vec4::<i32>::default())
                        .take(count.saturating_sub(items.len())),
                ),
                GeometryValues::Mat2I(items) => items.extend(
                    std::iter::repeat(Mat2::<i32>::default())
                        .take(count.saturating_sub(items.len())),
                ),
                GeometryValues::Mat3I(items) => items.extend(
                    std::iter::repeat(Mat3::<i32>::default())
                        .take(count.saturating_sub(items.len())),
                ),
                GeometryValues::Mat4I(items) => items.extend(
                    std::iter::repeat(Mat4::<i32>::default())
                        .take(count.saturating_sub(items.len())),
                ),
            }
        }
        self.count = count;
    }

    pub fn clear(&mut self) {
        for column in &mut self.columns {
            match &mut column.values {
                GeometryValues::Bool(items) => items.clear(),
                GeometryValues::String(items) => items.clear(),
                GeometryValues::Scalar(items) => items.clear(),
                GeometryValues::Vec2F(items) => items.clear(),
                GeometryValues::Vec3F(items) => items.clear(),
                GeometryValues::Vec4F(items) => items.clear(),
                GeometryValues::Mat2F(items) => items.clear(),
                GeometryValues::Mat3F(items) => items.clear(),
                GeometryValues::Mat4F(items) => items.clear(),
                GeometryValues::Integer(items) => items.clear(),
                GeometryValues::Vec2I(items) => items.clear(),
                GeometryValues::Vec3I(items) => items.clear(),
                GeometryValues::Vec4I(items) => items.clear(),
                GeometryValues::Mat2I(items) => items.clear(),
                GeometryValues::Mat3I(items) => items.clear(),
                GeometryValues::Mat4I(items) => items.clear(),
            }
        }
        self.count = 0;
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn remove_column(&mut self, name: &str) {
        if let Some(index) = self.columns.iter().position(|c| c.name == name) {
            self.columns.remove(index);
        }
    }

    pub fn set_column(&mut self, column: GeometryVerticesColumn) {
        self.remove_column(&column.name);
        self.columns.push(column);
        let count = self
            .columns
            .iter()
            .fold(0, |accum, column| column.values.len().max(accum));
        self.ensure_len(count);
    }

    pub fn push_columns(
        &mut self,
        columns: impl IntoIterator<Item = GeometryVerticesColumn>,
    ) -> Result<(), MeshError> {
        for column in columns {
            self.ensure_column(&column.name, column.values.as_value_type())?;
            let target = self
                .columns
                .iter_mut()
                .find(|c| c.name == column.name)
                .unwrap();
            match (&mut target.values, &column.values) {
                (GeometryValues::Bool(target), GeometryValues::Bool(source)) => {
                    target.extend(source)
                }
                (GeometryValues::String(target), GeometryValues::String(source)) => {
                    target.extend(source.to_owned())
                }
                (GeometryValues::Scalar(target), GeometryValues::Scalar(source)) => {
                    target.extend(source)
                }
                (GeometryValues::Vec2F(target), GeometryValues::Vec2F(source)) => {
                    target.extend(source)
                }
                (GeometryValues::Vec3F(target), GeometryValues::Vec3F(source)) => {
                    target.extend(source)
                }
                (GeometryValues::Vec4F(target), GeometryValues::Vec4F(source)) => {
                    target.extend(source)
                }
                (GeometryValues::Mat2F(target), GeometryValues::Mat2F(source)) => {
                    target.extend(source)
                }
                (GeometryValues::Mat3F(target), GeometryValues::Mat3F(source)) => {
                    target.extend(source)
                }
                (GeometryValues::Mat4F(target), GeometryValues::Mat4F(source)) => {
                    target.extend(source)
                }
                (GeometryValues::Integer(target), GeometryValues::Integer(source)) => {
                    target.extend(source)
                }
                (GeometryValues::Vec2I(target), GeometryValues::Vec2I(source)) => {
                    target.extend(source)
                }
                (GeometryValues::Vec3I(target), GeometryValues::Vec3I(source)) => {
                    target.extend(source)
                }
                (GeometryValues::Vec4I(target), GeometryValues::Vec4I(source)) => {
                    target.extend(source)
                }
                (GeometryValues::Mat2I(target), GeometryValues::Mat2I(source)) => {
                    target.extend(source)
                }
                (GeometryValues::Mat3I(target), GeometryValues::Mat3I(source)) => {
                    target.extend(source)
                }
                (GeometryValues::Mat4I(target), GeometryValues::Mat4I(source)) => {
                    target.extend(source)
                }
                _ => unreachable!(),
            }
        }
        let count = self
            .columns
            .iter()
            .fold(0, |accum, column| column.values.len().max(accum));
        self.ensure_len(count);
        Ok(())
    }

    pub fn push(&mut self, attributes: GeometryAttributes) -> Result<(), MeshError> {
        self.ensure_columns(&attributes)?;
        for column in &mut self.columns {
            match &mut column.values {
                GeometryValues::Bool(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::String(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Scalar(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Vec2F(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Vec3F(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Vec4F(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Mat2F(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Mat3F(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Mat4F(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Integer(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Vec2I(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Vec3I(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Vec4I(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Mat2I(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Mat3I(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Mat4I(items) => items.push(
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
            }
        }
        self.count += 1;
        Ok(())
    }

    pub fn set(&mut self, index: usize, attributes: GeometryAttributes) -> Result<(), MeshError> {
        if index >= self.count {
            return Err(MeshError::OutOfBounds(index, self.count));
        }
        self.ensure_columns(&attributes)?;
        for (key, value) in attributes.iter() {
            if let Some(column) = self.columns.iter_mut().find(|column| column.name == key) {
                match &mut column.values {
                    GeometryValues::Bool(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::String(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::Scalar(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::Vec2F(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::Vec3F(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::Vec4F(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::Mat2F(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::Mat3F(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::Mat4F(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::Integer(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::Vec2I(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::Vec3I(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::Vec4I(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::Mat2I(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::Mat3I(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                    GeometryValues::Mat4I(items) => {
                        items[index] = value.try_into().unwrap_or_default();
                    }
                }
            }
        }
        Ok(())
    }

    pub fn insert(
        &mut self,
        mut index: usize,
        attributes: GeometryAttributes,
    ) -> Result<(), MeshError> {
        index = index.min(self.count);
        self.ensure_columns(&attributes)?;
        for column in &mut self.columns {
            match &mut column.values {
                GeometryValues::Bool(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::String(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Scalar(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Vec2F(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Vec3F(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Vec4F(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Mat2F(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Mat3F(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Mat4F(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Integer(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Vec2I(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Vec3I(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Vec4I(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Mat2I(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Mat3I(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
                GeometryValues::Mat4I(items) => items.insert(
                    index,
                    attributes
                        .get(&column.name)
                        .and_then(|value| value.try_into().ok())
                        .unwrap_or_default(),
                ),
            }
        }
        self.count += 1;
        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<GeometryAttributes> {
        if index >= self.count {
            return None;
        }
        let mut result = GeometryAttributes::default();
        for column in &self.columns {
            match &column.values {
                GeometryValues::Bool(items) => result.set(&column.name, items[index])?,
                GeometryValues::String(items) => result.set(&column.name, items[index].as_str())?,
                GeometryValues::Scalar(items) => result.set(&column.name, items[index])?,
                GeometryValues::Vec2F(items) => result.set(&column.name, items[index])?,
                GeometryValues::Vec3F(items) => result.set(&column.name, items[index])?,
                GeometryValues::Vec4F(items) => result.set(&column.name, items[index])?,
                GeometryValues::Mat2F(items) => result.set(&column.name, items[index])?,
                GeometryValues::Mat3F(items) => result.set(&column.name, items[index])?,
                GeometryValues::Mat4F(items) => result.set(&column.name, items[index])?,
                GeometryValues::Integer(items) => result.set(&column.name, items[index])?,
                GeometryValues::Vec2I(items) => result.set(&column.name, items[index])?,
                GeometryValues::Vec3I(items) => result.set(&column.name, items[index])?,
                GeometryValues::Vec4I(items) => result.set(&column.name, items[index])?,
                GeometryValues::Mat2I(items) => result.set(&column.name, items[index])?,
                GeometryValues::Mat3I(items) => result.set(&column.name, items[index])?,
                GeometryValues::Mat4I(items) => result.set(&column.name, items[index])?,
            };
        }
        Some(result)
    }

    pub fn get_column(&self, name: &str, index: usize) -> Option<GeometryValue> {
        let column = self.columns.iter().find(|column| column.name == name)?;
        Some(match &column.values {
            GeometryValues::Bool(items) => items[index].into(),
            GeometryValues::String(items) => items[index].to_owned().into(),
            GeometryValues::Scalar(items) => items[index].into(),
            GeometryValues::Vec2F(items) => items[index].into(),
            GeometryValues::Vec3F(items) => items[index].into(),
            GeometryValues::Vec4F(items) => items[index].into(),
            GeometryValues::Mat2F(items) => items[index].into(),
            GeometryValues::Mat3F(items) => items[index].into(),
            GeometryValues::Mat4F(items) => items[index].into(),
            GeometryValues::Integer(items) => items[index].into(),
            GeometryValues::Vec2I(items) => items[index].into(),
            GeometryValues::Vec3I(items) => items[index].into(),
            GeometryValues::Vec4I(items) => items[index].into(),
            GeometryValues::Mat2I(items) => items[index].into(),
            GeometryValues::Mat3I(items) => items[index].into(),
            GeometryValues::Mat4I(items) => items[index].into(),
        })
    }

    pub fn remove(&mut self, index: usize) -> Option<GeometryAttributes> {
        let result = self.get(index)?;
        for column in &mut self.columns {
            match &mut column.values {
                GeometryValues::Bool(items) => {
                    items.remove(index);
                }
                GeometryValues::String(items) => {
                    items.remove(index);
                }
                GeometryValues::Scalar(items) => {
                    items.remove(index);
                }
                GeometryValues::Vec2F(items) => {
                    items.remove(index);
                }
                GeometryValues::Vec3F(items) => {
                    items.remove(index);
                }
                GeometryValues::Vec4F(items) => {
                    items.remove(index);
                }
                GeometryValues::Mat2F(items) => {
                    items.remove(index);
                }
                GeometryValues::Mat3F(items) => {
                    items.remove(index);
                }
                GeometryValues::Mat4F(items) => {
                    items.remove(index);
                }
                GeometryValues::Integer(items) => {
                    items.remove(index);
                }
                GeometryValues::Vec2I(items) => {
                    items.remove(index);
                }
                GeometryValues::Vec3I(items) => {
                    items.remove(index);
                }
                GeometryValues::Vec4I(items) => {
                    items.remove(index);
                }
                GeometryValues::Mat2I(items) => {
                    items.remove(index);
                }
                GeometryValues::Mat3I(items) => {
                    items.remove(index);
                }
                GeometryValues::Mat4I(items) => {
                    items.remove(index);
                }
            }
        }
        self.count -= 1;
        Some(result)
    }

    pub fn columns_types(&self) -> impl Iterator<Item = (&str, GeometryValueType)> {
        self.columns
            .iter()
            .map(|column| (column.name.as_str(), column.values.as_value_type()))
    }

    pub fn column_type(&self, name: &str) -> Result<GeometryValueType, MeshError> {
        match self
            .columns_types()
            .find(|(n, _)| n == &name)
            .map(|(_, t)| t)
        {
            Some(result) => Ok(result),
            None => Err(MeshError::GeometryAttributeNotFound(name.to_owned())),
        }
    }

    pub fn columns(&self) -> impl Iterator<Item = (&str, &GeometryValues)> {
        self.columns
            .iter()
            .map(|column| (column.name.as_str(), &column.values))
    }

    pub fn column(&self, name: &str) -> Result<&GeometryValues, MeshError> {
        match self
            .columns
            .iter()
            .find(|column| column.name == name)
            .map(|column| &column.values)
        {
            Some(result) => Ok(result),
            None => Err(MeshError::GeometryAttributeNotFound(name.to_owned())),
        }
    }

    pub fn iter(&self) -> GeometryVerticesIter {
        GeometryVerticesIter {
            inner: self
                .columns
                .iter()
                .map(|column| (column.name.as_str(), column.values.iter()))
                .collect(),
        }
    }
}

pub struct GeometryVerticesIter<'a> {
    inner: Vec<(&'a str, GeometryValuesIter<'a>)>,
}

impl<'a> Iterator for GeometryVerticesIter<'a> {
    type Item = GeometryAttributes;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = GeometryAttributes::default();
        for (name, iter) in &mut self.inner {
            result.set(name, iter.next()?);
        }
        Some(result)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Geometry {
    pub vertices: GeometryVertices,
    pub primitives: GeometryPrimitives,
    pub attributes: GeometryAttributes,
}

impl Geometry {
    pub fn triangles() -> Self {
        Self::new(
            GeometryVertices::default(),
            GeometryPrimitives::triangles(vec![]),
        )
    }

    pub fn lines() -> Self {
        Self::new(
            GeometryVertices::default(),
            GeometryPrimitives::lines(vec![]),
        )
    }

    pub fn points() -> Self {
        Self::new(
            GeometryVertices::default(),
            GeometryPrimitives::points(vec![]),
        )
    }

    pub fn new(vertices: GeometryVertices, primitives: GeometryPrimitives) -> Self {
        Self {
            vertices,
            primitives,
            attributes: Default::default(),
        }
    }

    pub fn with_attribute(mut self, name: impl ToString, value: GeometryValue) -> Self {
        self.attributes.set(name, value);
        self
    }

    pub fn factory<T: VertexType>(&self) -> Result<StaticVertexFactory, MeshError> {
        let vertex_layout = T::vertex_layout()?;
        let mut result = match &self.primitives {
            GeometryPrimitives::Triangles(triangles) => {
                let mut result = StaticVertexFactory::new(
                    vertex_layout.to_owned(),
                    triangles.items.len() * 3,
                    triangles.items.len(),
                    MeshDrawMode::Triangles,
                );
                result.triangles(
                    &triangles
                        .items
                        .iter()
                        .map(|item| {
                            let [a, b, c] = item.indices;
                            (a as u32, b as u32, c as u32)
                        })
                        .collect::<Vec<_>>(),
                    None,
                )?;
                result
            }
            GeometryPrimitives::Lines(lines) => {
                let mut result = StaticVertexFactory::new(
                    vertex_layout.to_owned(),
                    lines.items.len() * 2,
                    lines.items.len(),
                    MeshDrawMode::Lines,
                );
                result.lines(
                    &lines
                        .items
                        .iter()
                        .map(|item| {
                            let [a, b] = item.indices;
                            (a as u32, b as u32)
                        })
                        .collect::<Vec<_>>(),
                    None,
                )?;
                result
            }
            GeometryPrimitives::Points(points) => {
                let mut result = StaticVertexFactory::new(
                    vertex_layout.to_owned(),
                    points.items.len(),
                    points.items.len(),
                    MeshDrawMode::Points,
                );
                result.points(
                    &points
                        .items
                        .iter()
                        .map(|item| item.index as u32)
                        .collect::<Vec<_>>(),
                    None,
                )?;
                result
            }
        };
        for attribute in vertex_layout.attributes() {
            if let Ok(values) = self.vertices.column(&attribute.id) {
                match values {
                    GeometryValues::Bool(_) | GeometryValues::String(_) => {}
                    GeometryValues::Scalar(items) => {
                        result.vertices_scalar(&attribute.id, &items, None)?;
                    }
                    GeometryValues::Vec2F(items) => match attribute.value_type {
                        VertexValueType::Vec2F => {
                            result.vertices_vec2f(&attribute.id, &items, None)?;
                        }
                        VertexValueType::Vec3F => {
                            result.vertices_vec3f(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Vec4F => {
                            result.vertices_vec4f(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        _ => {}
                    },
                    GeometryValues::Vec3F(items) => match attribute.value_type {
                        VertexValueType::Vec2F => {
                            result.vertices_vec2f(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Vec3F => {
                            result.vertices_vec3f(&attribute.id, &items, None)?;
                        }
                        VertexValueType::Vec4F => {
                            result.vertices_vec4f(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        _ => {}
                    },
                    GeometryValues::Vec4F(items) => match attribute.value_type {
                        VertexValueType::Vec2F => {
                            result.vertices_vec2f(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Vec3F => {
                            result.vertices_vec3f(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Vec4F => {
                            result.vertices_vec4f(&attribute.id, &items, None)?;
                        }
                        _ => {}
                    },
                    GeometryValues::Mat2F(items) => match attribute.value_type {
                        VertexValueType::Mat2F => {
                            result.vertices_mat2f(&attribute.id, &items, None)?;
                        }
                        VertexValueType::Mat3F => {
                            result.vertices_mat3f(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Mat4F => {
                            result.vertices_mat4f(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        _ => {}
                    },
                    GeometryValues::Mat3F(items) => match attribute.value_type {
                        VertexValueType::Mat2F => {
                            result.vertices_mat2f(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Mat3F => {
                            result.vertices_mat3f(&attribute.id, &items, None)?;
                        }
                        VertexValueType::Mat4F => {
                            result.vertices_mat4f(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        _ => {}
                    },
                    GeometryValues::Mat4F(items) => match attribute.value_type {
                        VertexValueType::Mat2F => {
                            result.vertices_mat2f(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Mat3F => {
                            result.vertices_mat3f(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Mat4F => {
                            result.vertices_mat4f(&attribute.id, &items, None)?;
                        }
                        _ => {}
                    },
                    GeometryValues::Integer(items) => {
                        result.vertices_integer(&attribute.id, &items, None)?;
                    }
                    GeometryValues::Vec2I(items) => match attribute.value_type {
                        VertexValueType::Vec2I => {
                            result.vertices_vec2i(&attribute.id, &items, None)?;
                        }
                        VertexValueType::Vec3I => {
                            result.vertices_vec3i(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Vec4I => {
                            result.vertices_vec4i(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        _ => {}
                    },
                    GeometryValues::Vec3I(items) => match attribute.value_type {
                        VertexValueType::Vec2I => {
                            result.vertices_vec2i(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Vec3I => {
                            result.vertices_vec3i(&attribute.id, &items, None)?;
                        }
                        VertexValueType::Vec4I => {
                            result.vertices_vec4i(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        _ => {}
                    },
                    GeometryValues::Vec4I(items) => match attribute.value_type {
                        VertexValueType::Vec2I => {
                            result.vertices_vec2i(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Vec3I => {
                            result.vertices_vec3i(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Vec4I => {
                            result.vertices_vec4i(&attribute.id, &items, None)?;
                        }
                        _ => {}
                    },
                    GeometryValues::Mat2I(items) => match attribute.value_type {
                        VertexValueType::Mat2I => {
                            result.vertices_mat2i(&attribute.id, &items, None)?;
                        }
                        VertexValueType::Mat3I => {
                            result.vertices_mat3i(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Mat4I => {
                            result.vertices_mat4i(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        _ => {}
                    },
                    GeometryValues::Mat3I(items) => match attribute.value_type {
                        VertexValueType::Mat2I => {
                            result.vertices_mat2i(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Mat3I => {
                            result.vertices_mat3i(&attribute.id, &items, None)?;
                        }
                        VertexValueType::Mat4I => {
                            result.vertices_mat4i(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        _ => {}
                    },
                    GeometryValues::Mat4I(items) => match attribute.value_type {
                        VertexValueType::Mat2I => {
                            result.vertices_mat2i(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Mat3I => {
                            result.vertices_mat3i(
                                &attribute.id,
                                &items
                                    .into_iter()
                                    .map(|value| (*value).into())
                                    .collect::<Vec<_>>(),
                                None,
                            )?;
                        }
                        VertexValueType::Mat4I => {
                            result.vertices_mat4i(&attribute.id, &items, None)?;
                        }
                        _ => {}
                    },
                }
            }
        }
        Ok(result)
    }
}
