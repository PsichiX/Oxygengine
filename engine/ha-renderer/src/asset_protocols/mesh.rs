use crate::{
    asset_protocols::skeleton::SkeletonAsset,
    material::domains::{
        screenspace::ScreenSpaceQuadFactory,
        surface::{
            circle::SurfaceCircleFactory, grid::SurfaceGridFactory, quad::SurfaceQuadFactory,
            skinned::sprite::SurfaceSkinnedSpriteFactory, triangles2d::SurfaceTriangles2dFactory,
            SurfaceVertexP, SurfaceVertexPC, SurfaceVertexPN, SurfaceVertexPNC, SurfaceVertexPNT,
            SurfaceVertexPNTC, SurfaceVertexPT, SurfaceVertexPTC, SurfaceVertexSP,
            SurfaceVertexSPC, SurfaceVertexSPN, SurfaceVertexSPNC, SurfaceVertexSPNT,
            SurfaceVertexSPNTC, SurfaceVertexSPT, SurfaceVertexSPTC,
        },
    },
    mesh::{skeleton::Skeleton, vertex_factory::StaticVertexFactory, MeshError},
};
use core::{
    assets::{
        asset::Asset,
        database::AssetsDatabase,
        protocol::{AssetLoadResult, AssetProtocol, AssetVariant, Meta},
    },
    Ignite,
};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurfaceVertexData {
    pub normal: bool,
    pub texture: bool,
    pub color: bool,
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum SurfaceFactory {
    Quad(SurfaceQuadFactory),
    Grid(SurfaceGridFactory),
    Circle(SurfaceCircleFactory),
    Triangles2d(SurfaceTriangles2dFactory),
}

impl SurfaceFactory {
    pub fn factory(&self, data: SurfaceVertexData) -> Result<StaticVertexFactory, MeshError> {
        let SurfaceVertexData {
            normal,
            texture,
            color,
        } = data;
        match self {
            Self::Quad(factory) => match (normal, texture, color) {
                (false, false, false) => factory.factory::<SurfaceVertexP>(),
                (true, false, false) => factory.factory::<SurfaceVertexPN>(),
                (false, true, false) => factory.factory::<SurfaceVertexPT>(),
                (true, true, false) => factory.factory::<SurfaceVertexPNT>(),
                (false, false, true) => factory.factory::<SurfaceVertexPC>(),
                (true, false, true) => factory.factory::<SurfaceVertexPNC>(),
                (false, true, true) => factory.factory::<SurfaceVertexPTC>(),
                (true, true, true) => factory.factory::<SurfaceVertexPNTC>(),
            },
            Self::Grid(factory) => match (normal, texture, color) {
                (false, false, false) => factory.factory::<SurfaceVertexP>(),
                (true, false, false) => factory.factory::<SurfaceVertexPN>(),
                (false, true, false) => factory.factory::<SurfaceVertexPT>(),
                (true, true, false) => factory.factory::<SurfaceVertexPNT>(),
                (false, false, true) => factory.factory::<SurfaceVertexPC>(),
                (true, false, true) => factory.factory::<SurfaceVertexPNC>(),
                (false, true, true) => factory.factory::<SurfaceVertexPTC>(),
                (true, true, true) => factory.factory::<SurfaceVertexPNTC>(),
            },
            Self::Circle(factory) => match (normal, texture, color) {
                (false, false, false) => factory.factory::<SurfaceVertexP>(),
                (true, false, false) => factory.factory::<SurfaceVertexPN>(),
                (false, true, false) => factory.factory::<SurfaceVertexPT>(),
                (true, true, false) => factory.factory::<SurfaceVertexPNT>(),
                (false, false, true) => factory.factory::<SurfaceVertexPC>(),
                (true, false, true) => factory.factory::<SurfaceVertexPNC>(),
                (false, true, true) => factory.factory::<SurfaceVertexPTC>(),
                (true, true, true) => factory.factory::<SurfaceVertexPNTC>(),
            },
            Self::Triangles2d(factory) => match (normal, texture, color) {
                (false, false, false) => factory.factory::<SurfaceVertexP>(),
                (true, false, false) => factory.factory::<SurfaceVertexPN>(),
                (false, true, false) => factory.factory::<SurfaceVertexPT>(),
                (true, true, false) => factory.factory::<SurfaceVertexPNT>(),
                (false, false, true) => factory.factory::<SurfaceVertexPC>(),
                (true, false, true) => factory.factory::<SurfaceVertexPNC>(),
                (false, true, true) => factory.factory::<SurfaceVertexPTC>(),
                (true, true, true) => factory.factory::<SurfaceVertexPNTC>(),
            },
        }
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceMeshAsset {
    #[serde(default)]
    pub vertex_data: SurfaceVertexData,
    pub factory: SurfaceFactory,
}

impl SurfaceMeshAsset {
    pub fn factory(&self) -> Result<StaticVertexFactory, MeshError> {
        self.factory.factory(self.vertex_data)
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum SkinnedSurfaceFactory {
    Triangles2d(SurfaceTriangles2dFactory),
    Sprite {
        skeleton: AssetVariant,
        factory: SurfaceSkinnedSpriteFactory,
    },
}

impl SkinnedSurfaceFactory {
    pub fn factory(
        &self,
        data: SurfaceVertexData,
        assets: &AssetsDatabase,
    ) -> Result<StaticVertexFactory, MeshError> {
        let SurfaceVertexData {
            normal,
            texture,
            color,
        } = data;
        match self {
            Self::Triangles2d(factory) => match (normal, texture, color) {
                (false, false, false) => factory.factory::<SurfaceVertexSP>(),
                (true, false, false) => factory.factory::<SurfaceVertexSPN>(),
                (false, true, false) => factory.factory::<SurfaceVertexSPT>(),
                (true, true, false) => factory.factory::<SurfaceVertexSPNT>(),
                (false, false, true) => factory.factory::<SurfaceVertexSPC>(),
                (true, false, true) => factory.factory::<SurfaceVertexSPNC>(),
                (false, true, true) => factory.factory::<SurfaceVertexSPTC>(),
                (true, true, true) => factory.factory::<SurfaceVertexSPNTC>(),
            },
            Self::Sprite { skeleton, factory } => {
                let id = match &skeleton {
                    AssetVariant::Id(id) => *id,
                    AssetVariant::Path(path) => match assets.id_by_path(path) {
                        Some(id) => id,
                        _ => {
                            return Err(MeshError::Internal(format!(
                                "No skeleton asset found: {}",
                                path
                            )))
                        }
                    },
                };
                let asset = match assets
                    .asset_by_id(id)
                    .and_then(|asset| asset.get::<SkeletonAsset>())
                {
                    Some(asset) => asset,
                    _ => return Err(MeshError::Internal("Skeleton asset not found!".to_owned())),
                };
                let skeleton = match Skeleton::try_from(asset.get().to_owned()) {
                    Ok(skeleton) => skeleton,
                    Err(error) => {
                        return Err(MeshError::Internal(format!(
                            "Skeleton cannot be built from hierarchy: {:?}",
                            error
                        )))
                    }
                };
                match (normal, texture, color) {
                    (false, false, false) => factory.factory::<SurfaceVertexSP>(&skeleton),
                    (true, false, false) => factory.factory::<SurfaceVertexSPN>(&skeleton),
                    (false, true, false) => factory.factory::<SurfaceVertexSPT>(&skeleton),
                    (true, true, false) => factory.factory::<SurfaceVertexSPNT>(&skeleton),
                    (false, false, true) => factory.factory::<SurfaceVertexSPC>(&skeleton),
                    (true, false, true) => factory.factory::<SurfaceVertexSPNC>(&skeleton),
                    (false, true, true) => factory.factory::<SurfaceVertexSPTC>(&skeleton),
                    (true, true, true) => factory.factory::<SurfaceVertexSPNTC>(&skeleton),
                }
            }
        }
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct SkinnedSurfaceMeshAsset {
    #[serde(default)]
    pub vertex_data: SurfaceVertexData,
    pub factory: SkinnedSurfaceFactory,
}

impl SkinnedSurfaceMeshAsset {
    pub fn factory(&self, assets: &AssetsDatabase) -> Result<StaticVertexFactory, MeshError> {
        self.factory.factory(self.vertex_data, assets)
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct ScreenSpaceMeshAsset(pub ScreenSpaceQuadFactory);

impl ScreenSpaceMeshAsset {
    pub fn factory(&self) -> Result<StaticVertexFactory, MeshError> {
        self.0.factory()
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum MeshAsset {
    Surface(SurfaceMeshAsset),
    SkinnedSurface(SkinnedSurfaceMeshAsset),
    ScreenSpace(ScreenSpaceMeshAsset),
    Raw(StaticVertexFactory),
}

impl MeshAsset {
    pub fn factory(&self, assets: &AssetsDatabase) -> Result<StaticVertexFactory, MeshError> {
        match self {
            Self::Surface(surface) => surface.factory(),
            Self::SkinnedSurface(skinned_surface) => skinned_surface.factory(assets),
            Self::ScreenSpace(screenspace) => screenspace.factory(),
            Self::Raw(factory) => Ok(factory.to_owned()),
        }
    }
}

pub struct MeshAssetProtocol;

impl AssetProtocol for MeshAssetProtocol {
    fn name(&self) -> &str {
        "mesh"
    }

    fn on_load_with_path(&mut self, path: &str, data: Vec<u8>) -> AssetLoadResult {
        let mesh = if path.ends_with(".json") {
            let data = from_utf8(&data).unwrap();
            serde_json::from_str::<MeshAsset>(data).unwrap()
        } else if path.ends_with(".yaml") {
            let data = from_utf8(&data).unwrap();
            serde_yaml::from_str::<MeshAsset>(data).unwrap()
        } else {
            bincode::deserialize::<MeshAsset>(&data).unwrap()
        };
        if let MeshAsset::SkinnedSurface(asset) = &mesh {
            if let SkinnedSurfaceFactory::Sprite {
                skeleton: AssetVariant::Path(name),
                ..
            } = &asset.factory
            {
                let to_load = vec![("skeleton".to_owned(), format!("skeleton://{}", name))];
                return AssetLoadResult::Yield(Some(Box::new(mesh)), to_load);
            }
        }
        AssetLoadResult::Data(Box::new(mesh))
    }

    // on_load_with_path() handles loading so this is not needed, so we just make it unreachable.
    fn on_load(&mut self, _data: Vec<u8>) -> AssetLoadResult {
        unreachable!()
    }

    fn on_resume(&mut self, meta: Meta, list: &[(&str, &Asset)]) -> AssetLoadResult {
        let id = list.get(0).unwrap().1.id();
        let mut mesh = *meta.unwrap().downcast::<MeshAsset>().unwrap();
        if let MeshAsset::SkinnedSurface(asset) = &mut mesh {
            if let SkinnedSurfaceFactory::Sprite { skeleton, .. } = &mut asset.factory {
                *skeleton = AssetVariant::Id(id);
            }
        }
        AssetLoadResult::Data(Box::new(mesh))
    }

    fn on_unload(&mut self, asset: &Asset) -> Option<Vec<AssetVariant>> {
        if let MeshAsset::SkinnedSurface(asset) = asset.get::<MeshAsset>().unwrap() {
            if let SkinnedSurfaceFactory::Sprite { skeleton, .. } = &asset.factory {
                return Some(vec![skeleton.to_owned()]);
            }
        }
        None
    }
}
