use crate::{
    components::tilemap_instance::HaTileMapInstance,
    image::VirtualImage,
    material::domains::surface::SurfaceTexturedDomain,
    math::*,
    mesh::{vertex_factory::StaticVertexFactory, MeshDrawMode, MeshError},
    Resources,
};
use core::Scalar;

#[derive(Debug, Copy, Clone)]
pub struct SurfaceTileMapFactory {}

impl SurfaceTileMapFactory {
    pub fn factory<T>(
        tilemap: &HaTileMapInstance,
        resources: &Resources<VirtualImage>,
    ) -> Result<StaticVertexFactory, MeshError>
    where
        T: SurfaceTexturedDomain,
    {
        if tilemap.cols() == 0 || tilemap.rows() == 0 || tilemap.tiles().is_empty() {
            return Err(MeshError::ZeroSize);
        }
        let vertex_layout = T::vertex_layout()?;
        if !T::has_attribute("position") {
            return Err(MeshError::MissingRequiredLayoutAttribute(
                vertex_layout,
                "position".to_owned(),
            ));
        }
        if !T::has_attribute("textureCoord") {
            return Err(MeshError::MissingRequiredLayoutAttribute(
                vertex_layout,
                "textureCoord".to_owned(),
            ));
        }
        let virtual_image = match resources.get_named(tilemap.atlas()) {
            Some(virtual_image) => virtual_image,
            None => {
                return Err(MeshError::Internal(format!(
                    "Could not find atlas virtual image: {}",
                    tilemap.atlas()
                )))
            }
        };
        let tiles = tilemap
            .tiles()
            .iter()
            .filter_map(|tile| {
                virtual_image
                    .named_image_uvs(&tile.atlas_item)
                    .map(|uvs| (tile, uvs))
            })
            .collect::<Vec<_>>();

        let count = tiles.len();
        let vertex_count = count * 4;
        let triangles_count = count * 2;
        let mut result = StaticVertexFactory::new(
            vertex_layout,
            vertex_count,
            triangles_count,
            MeshDrawMode::Triangles,
        );
        let offset = Vec2::new(tilemap.cols() as Scalar, tilemap.rows() as Scalar)
            * tilemap.cell_size()
            * tilemap.pivot();
        let cell_size = tilemap.cell_size();
        let mut position = Vec::with_capacity(vertex_count);
        for (tile, _) in &tiles {
            let from = Vec2::new(tile.col as Scalar, tile.row as Scalar) * cell_size - offset;
            let to =
                Vec2::new((tile.col + 1) as Scalar, (tile.row + 1) as Scalar) * cell_size - offset;
            position.push(vec3(from.x, from.y, 0.0));
            position.push(vec3(to.x, from.y, 0.0));
            position.push(vec3(to.x, to.y, 0.0));
            position.push(vec3(from.x, to.y, 0.0));
        }
        result.vertices_vec3f("position", &position, None)?;
        let mut texture_coord = Vec::with_capacity(vertex_count);
        for (_, (uvs, layer)) in &tiles {
            texture_coord.push(vec3(uvs.x, uvs.y, *layer as _));
            texture_coord.push(vec3(uvs.x + uvs.w, uvs.y, *layer as _));
            texture_coord.push(vec3(uvs.x + uvs.w, uvs.y + uvs.h, *layer as _));
            texture_coord.push(vec3(uvs.x, uvs.y + uvs.h, *layer as _));
        }
        result.vertices_vec3f("textureCoord", &texture_coord, None)?;
        let mut triangles = Vec::with_capacity(triangles_count);
        for i in 0..count {
            let i = i * 4;
            let tl = i as u32;
            let tr = i as u32 + 1;
            let br = i as u32 + 2;
            let bl = i as u32 + 3;
            triangles.push((tl, tr, br));
            triangles.push((br, bl, tl));
        }
        result.triangles(&triangles, None)?;
        Ok(result)
    }
}
