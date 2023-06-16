use crate::mesh::{
    geometry::{Geometry, GeometryValues, GeometryVerticesColumn},
    rig::deformer::Deformer,
    MeshError,
};

pub fn apply_deformer(mut geometry: Geometry, deformer: &Deformer) -> Result<Geometry, MeshError> {
    let positions = geometry.vertices.column("position")?.iter();
    let deformer_areas = geometry.vertices.column("@deformer-area")?.iter();
    let meta = positions
        .zip(deformer_areas)
        .map(|(position, deformer_area)| {
            let deformer_area: String = deformer_area.try_into()?;
            let (area, cells_offset) =
                deformer
                    .find_area_cells_offset(&deformer_area)
                    .ok_or_else(|| {
                        MeshError::Internal(format!("Deformer has no area: {}", &deformer_area))
                    })?;
            let (position, _, _, cell_index) = area.local_to_area(position.try_into()?);
            let curves_index = (cells_offset + cell_index) as i32;
            Ok((position, curves_index))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (positions, curves_indices): (Vec<_>, Vec<_>) = meta.into_iter().unzip();
    geometry.vertices.set_column(GeometryVerticesColumn::new(
        "position",
        GeometryValues::Vec2F(positions),
    ));
    geometry.vertices.set_column(GeometryVerticesColumn::new(
        "curvesIndex",
        GeometryValues::Integer(curves_indices),
    ));
    Ok(geometry)
}
