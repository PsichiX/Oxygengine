use crate::mesh::geometry::{Geometry, GeometryAttributes, GeometryPrimitives};
use std::fmt::{Error, Write};
use tabled::{builder::Builder, settings::Style, Table};

pub fn debug(geometry: &Geometry) -> Result<String, Error> {
    let mut result = String::default();
    if !geometry.attributes.is_empty() {
        writeln!(&mut result, "* Geometry attributes:")?;
        writeln!(&mut result, "{}", debug_attributes(&geometry.attributes))?;
    }
    if !geometry.vertices.attributes.is_empty() {
        writeln!(&mut result, "* Geometry vertices attributes:")?;
        writeln!(
            &mut result,
            "{}",
            debug_attributes(&geometry.vertices.attributes)
        )?;
    }
    writeln!(&mut result, "* Geometry vertices:")?;
    let mut builder = Builder::default();
    builder.push_record(
        std::iter::once("#".to_owned()).chain(
            geometry
                .vertices
                .columns_types()
                .map(|(name, value_type)| format!("{}\n({:?})", name, value_type)),
        ),
    );
    for (index, attributes) in geometry.vertices.iter().enumerate() {
        builder.push_record(
            std::iter::once(index.to_string()).chain(
                geometry
                    .vertices
                    .columns_types()
                    .map(|(name, _)| attributes.get(name).unwrap().to_string()),
            ),
        );
    }
    let mut table = builder.build();
    table.with(Style::modern());
    writeln!(&mut result, "{}", table)?;
    match &geometry.primitives {
        GeometryPrimitives::Triangles(triangles) => {
            if !triangles.attributes.is_empty() {
                writeln!(&mut result, "* Geometry triangles attributes:")?;
                writeln!(&mut result, "{}", debug_attributes(&triangles.attributes))?;
            }
            writeln!(&mut result, "* Geometry triangles:")?;
            let mut builder = Builder::default();
            builder.push_record(["Index A", "Index B", "Index C", "Attributes"]);
            for point in &triangles.items {
                builder.push_record([
                    point.indices[0].to_string(),
                    point.indices[1].to_string(),
                    point.indices[2].to_string(),
                    format!("{}", debug_attributes(&point.attributes)),
                ]);
            }
            let mut table = builder.build();
            table.with(Style::modern());
            writeln!(&mut result, "{}", table)?;
        }
        GeometryPrimitives::Lines(lines) => {
            if !lines.attributes.is_empty() {
                writeln!(&mut result, "* Geometry lines attributes:")?;
                writeln!(&mut result, "{}", debug_attributes(&lines.attributes))?;
            }
            writeln!(&mut result, "* Geometry lines:")?;
            let mut builder = Builder::default();
            builder.push_record(["Index A", "Index B", "Attributes"]);
            for point in &lines.items {
                builder.push_record([
                    point.indices[0].to_string(),
                    point.indices[1].to_string(),
                    format!("{}", debug_attributes(&point.attributes)),
                ]);
            }
            let mut table = builder.build();
            table.with(Style::modern());
            writeln!(&mut result, "{}", table)?;
        }
        GeometryPrimitives::Points(points) => {
            if !points.attributes.is_empty() {
                writeln!(&mut result, "* Geometry points attributes:")?;
                writeln!(&mut result, "{}", debug_attributes(&points.attributes))?;
            }
            writeln!(&mut result, "* Geometry points:")?;
            let mut builder = Builder::default();
            builder.push_record(["Index", "Attributes"]);
            for point in &points.items {
                builder.push_record([
                    point.index.to_string(),
                    format!("{}", debug_attributes(&point.attributes)),
                ]);
            }
            let mut table = builder.build();
            table.with(Style::modern());
            writeln!(&mut result, "{}", table)?;
        }
    }
    Ok(result)
}

fn debug_attributes(attributes: &GeometryAttributes) -> Table {
    let mut builder = Builder::default();
    for (name, value) in attributes.iter() {
        builder.push_record([name.to_string(), format!("{:?}", value)]);
    }
    let mut table = builder.build();
    table.with(Style::modern());
    table
}
