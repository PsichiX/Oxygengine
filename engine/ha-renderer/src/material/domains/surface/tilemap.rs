use crate::{
    components::tilemap_instance::HaTileMapInstance,
    image::VirtualImage,
    material::domains::surface::SurfaceTexturedDomain,
    math::*,
    mesh::{
        geometry::{
            Geometry, GeometryPrimitives, GeometryTriangle, GeometryVertices,
            GeometryVerticesColumn,
        },
        vertex_factory::StaticVertexFactory,
        MeshError,
    },
    Resources,
};
use core::Scalar;

#[derive(Debug, Copy, Clone)]
pub struct SurfaceTileMapFactory;

impl SurfaceTileMapFactory {
    pub fn geometry(
        tilemap: &HaTileMapInstance,
        resources: &Resources<VirtualImage>,
        meta: bool,
    ) -> Result<Geometry, MeshError> {
        if tilemap.cols() == 0 || tilemap.rows() == 0 || tilemap.tiles().is_empty() {
            return Err(MeshError::ZeroSize);
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
        let offset = Vec2::new(tilemap.cols() as Scalar, tilemap.rows() as Scalar)
            * tilemap.cell_size()
            * tilemap.pivot();
        let cell_size = tilemap.cell_size();
        Ok(Geometry::new(
            GeometryVertices::default().with_columns([
                GeometryVerticesColumn::new(
                    "position",
                    tiles
                        .iter()
                        .flat_map(|(tile, _)| {
                            let from = Vec2::new(tile.col as Scalar, tile.row as Scalar)
                                * cell_size
                                - offset;
                            let to = Vec2::new((tile.col + 1) as Scalar, (tile.row + 1) as Scalar)
                                * cell_size
                                - offset;
                            [
                                vec2(from.x, from.y),
                                vec2(to.x, from.y),
                                vec2(to.x, to.y),
                                vec2(from.x, to.y),
                            ]
                        })
                        .collect(),
                ),
                GeometryVerticesColumn::new(
                    "textureCoord",
                    tiles
                        .iter()
                        .flat_map(|(_, (uvs, layer))| {
                            [
                                vec3(uvs.x, uvs.y, *layer as _),
                                vec3(uvs.x + uvs.w, uvs.y, *layer as _),
                                vec3(uvs.x + uvs.w, uvs.y + uvs.h, *layer as _),
                                vec3(uvs.x, uvs.y + uvs.h, *layer as _),
                            ]
                        })
                        .collect(),
                ),
            ])?,
            GeometryPrimitives::triangles(
                tiles
                    .iter()
                    .enumerate()
                    .flat_map(|(index, (tile, _))| {
                        let i = index * 4;
                        let tl = i;
                        let tr = i + 1;
                        let br = i + 2;
                        let bl = i + 3;
                        let mut a = GeometryTriangle::new([tl, tr, br]);
                        let mut b = GeometryTriangle::new([br, bl, tl]);
                        if meta {
                            a.attributes.set("index", index as i32);
                            b.attributes.set("index", index as i32);
                            a.attributes.set("col", tile.col as i32);
                            b.attributes.set("col", tile.col as i32);
                            a.attributes.set("row", tile.row as i32);
                            b.attributes.set("row", tile.row as i32);
                            a.attributes.set("atlas-item", &tile.atlas_item);
                            b.attributes.set("atlas-item", &tile.atlas_item);
                        }
                        [a, b]
                    })
                    .collect::<Vec<_>>(),
            ),
        ))
    }

    pub fn factory<T>(
        tilemap: &HaTileMapInstance,
        resources: &Resources<VirtualImage>,
    ) -> Result<StaticVertexFactory, MeshError>
    where
        T: SurfaceTexturedDomain,
    {
        Self::geometry(tilemap, resources, false)?.factory::<T>()
    }
}
