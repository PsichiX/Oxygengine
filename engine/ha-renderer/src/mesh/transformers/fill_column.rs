use crate::mesh::{
    geometry::{Geometry, GeometryValue, GeometryValues, GeometryVerticesColumn},
    MeshError,
};
use std::ops::Range;

pub fn fill_column(
    mut geometry: Geometry,
    name: &str,
    value: impl Into<GeometryValue>,
    range: Option<Range<usize>>,
) -> Result<Geometry, MeshError> {
    let value = value.into();
    geometry
        .vertices
        .ensure_column(name, value.as_value_type())?;
    let column = geometry.vertices.column(name)?;
    let values = match (column.clone(), value.clone()) {
        (GeometryValues::Bool(mut values), GeometryValue::Bool(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Bool(values)
        }
        (GeometryValues::String(mut values), GeometryValue::String(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::String(values)
        }
        (GeometryValues::Scalar(mut values), GeometryValue::Scalar(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Scalar(values)
        }
        (GeometryValues::Vec2F(mut values), GeometryValue::Vec2F(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Vec2F(values)
        }
        (GeometryValues::Vec3F(mut values), GeometryValue::Vec3F(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Vec3F(values)
        }
        (GeometryValues::Vec4F(mut values), GeometryValue::Vec4F(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Vec4F(values)
        }
        (GeometryValues::Mat2F(mut values), GeometryValue::Mat2F(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Mat2F(values)
        }
        (GeometryValues::Mat3F(mut values), GeometryValue::Mat3F(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Mat3F(values)
        }
        (GeometryValues::Mat4F(mut values), GeometryValue::Mat4F(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Mat4F(values)
        }
        (GeometryValues::Integer(mut values), GeometryValue::Integer(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Integer(values)
        }
        (GeometryValues::Vec2I(mut values), GeometryValue::Vec2I(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Vec2I(values)
        }
        (GeometryValues::Vec3I(mut values), GeometryValue::Vec3I(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Vec3I(values)
        }
        (GeometryValues::Vec4I(mut values), GeometryValue::Vec4I(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Vec4I(values)
        }
        (GeometryValues::Mat2I(mut values), GeometryValue::Mat2I(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Mat2I(values)
        }
        (GeometryValues::Mat3I(mut values), GeometryValue::Mat3I(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Mat3I(values)
        }
        (GeometryValues::Mat4I(mut values), GeometryValue::Mat4I(value)) => {
            if let Some(range) = range {
                values[range].fill(value);
            } else {
                values.fill(value);
            };
            GeometryValues::Mat4I(values)
        }
        _ => {
            return Err(MeshError::GeometryValueTypeMismatch(
                value.as_value_type(),
                column.as_value_type(),
            ))
        }
    };
    geometry
        .vertices
        .set_column(GeometryVerticesColumn::new(name, values));
    Ok(geometry)
}
