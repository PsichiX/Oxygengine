use crate::{
    asset_protocols::rig::RigAsset,
    material::domains::{
        screenspace::ScreenSpaceQuadFactory,
        surface::{
            circle::SurfaceCircleFactory, grid::SurfaceGridFactory, quad::SurfaceQuadFactory,
            rig2d::SurfaceRig2dFactory, triangles2d::SurfaceTriangles2dFactory, SurfaceVertexDP,
            SurfaceVertexDPC, SurfaceVertexDPT, SurfaceVertexDPTC, SurfaceVertexDSP,
            SurfaceVertexDSPC, SurfaceVertexDSPT, SurfaceVertexDSPTC, SurfaceVertexP,
            SurfaceVertexPC, SurfaceVertexPT, SurfaceVertexPTC, SurfaceVertexSP, SurfaceVertexSPC,
            SurfaceVertexSPT, SurfaceVertexSPTC,
        },
    },
    mesh::{geometry::Geometry, vertex_factory::StaticVertexFactory, MeshError},
};
use core::assets::{
    asset::Asset,
    database::AssetsDatabase,
    protocol::{AssetLoadResult, AssetProtocol, AssetVariant, Meta},
};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshVertexData {
    pub texture: bool,
    pub color: bool,
    pub skinning: bool,
    pub deforming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SurfaceFactory {
    Quad(SurfaceQuadFactory),
    Grid(SurfaceGridFactory),
    Circle(SurfaceCircleFactory),
    Triangles2d(SurfaceTriangles2dFactory),
    Rig {
        asset: AssetVariant,
        factory: SurfaceRig2dFactory,
    },
}

impl SurfaceFactory {
    pub fn factory(
        &self,
        data: MeshVertexData,
        assets: &AssetsDatabase,
    ) -> Result<StaticVertexFactory, MeshError> {
        let MeshVertexData {
            deforming,
            skinning,
            texture,
            color,
        } = data;
        match self {
            Self::Quad(factory) => match (deforming, skinning, texture, color) {
                (true, true, true, true) => factory.factory::<SurfaceVertexDSPTC>(),
                (true, true, true, false) => factory.factory::<SurfaceVertexDSPT>(),
                (true, true, false, true) => factory.factory::<SurfaceVertexDSPC>(),
                (true, true, false, false) => factory.factory::<SurfaceVertexDSP>(),
                (true, false, true, true) => factory.factory::<SurfaceVertexDPTC>(),
                (true, false, true, false) => factory.factory::<SurfaceVertexDPT>(),
                (true, false, false, true) => factory.factory::<SurfaceVertexDPC>(),
                (true, false, false, false) => factory.factory::<SurfaceVertexDP>(),
                (false, true, true, true) => factory.factory::<SurfaceVertexSPTC>(),
                (false, true, true, false) => factory.factory::<SurfaceVertexSPT>(),
                (false, true, false, true) => factory.factory::<SurfaceVertexSPC>(),
                (false, true, false, false) => factory.factory::<SurfaceVertexSP>(),
                (false, false, true, true) => factory.factory::<SurfaceVertexPTC>(),
                (false, false, true, false) => factory.factory::<SurfaceVertexPT>(),
                (false, false, false, true) => factory.factory::<SurfaceVertexPC>(),
                (false, false, false, false) => factory.factory::<SurfaceVertexP>(),
            },
            Self::Grid(factory) => match (deforming, skinning, texture, color) {
                (true, true, true, true) => factory.factory::<SurfaceVertexDSPTC>(),
                (true, true, true, false) => factory.factory::<SurfaceVertexDSPT>(),
                (true, true, false, true) => factory.factory::<SurfaceVertexDSPC>(),
                (true, true, false, false) => factory.factory::<SurfaceVertexDSP>(),
                (true, false, true, true) => factory.factory::<SurfaceVertexDPTC>(),
                (true, false, true, false) => factory.factory::<SurfaceVertexDPT>(),
                (true, false, false, true) => factory.factory::<SurfaceVertexDPC>(),
                (true, false, false, false) => factory.factory::<SurfaceVertexDP>(),
                (false, true, true, true) => factory.factory::<SurfaceVertexSPTC>(),
                (false, true, true, false) => factory.factory::<SurfaceVertexSPT>(),
                (false, true, false, true) => factory.factory::<SurfaceVertexSPC>(),
                (false, true, false, false) => factory.factory::<SurfaceVertexSP>(),
                (false, false, true, true) => factory.factory::<SurfaceVertexPTC>(),
                (false, false, true, false) => factory.factory::<SurfaceVertexPT>(),
                (false, false, false, true) => factory.factory::<SurfaceVertexPC>(),
                (false, false, false, false) => factory.factory::<SurfaceVertexP>(),
            },
            Self::Circle(factory) => match (deforming, skinning, texture, color) {
                (true, true, true, true) => factory.factory::<SurfaceVertexDSPTC>(),
                (true, true, true, false) => factory.factory::<SurfaceVertexDSPT>(),
                (true, true, false, true) => factory.factory::<SurfaceVertexDSPC>(),
                (true, true, false, false) => factory.factory::<SurfaceVertexDSP>(),
                (true, false, true, true) => factory.factory::<SurfaceVertexDPTC>(),
                (true, false, true, false) => factory.factory::<SurfaceVertexDPT>(),
                (true, false, false, true) => factory.factory::<SurfaceVertexDPC>(),
                (true, false, false, false) => factory.factory::<SurfaceVertexDP>(),
                (false, true, true, true) => factory.factory::<SurfaceVertexSPTC>(),
                (false, true, true, false) => factory.factory::<SurfaceVertexSPT>(),
                (false, true, false, true) => factory.factory::<SurfaceVertexSPC>(),
                (false, true, false, false) => factory.factory::<SurfaceVertexSP>(),
                (false, false, true, true) => factory.factory::<SurfaceVertexPTC>(),
                (false, false, true, false) => factory.factory::<SurfaceVertexPT>(),
                (false, false, false, true) => factory.factory::<SurfaceVertexPC>(),
                (false, false, false, false) => factory.factory::<SurfaceVertexP>(),
            },
            Self::Triangles2d(factory) => match (deforming, skinning, texture, color) {
                (true, true, true, true) => factory.factory::<SurfaceVertexDSPTC>(),
                (true, true, true, false) => factory.factory::<SurfaceVertexDSPT>(),
                (true, true, false, true) => factory.factory::<SurfaceVertexDSPC>(),
                (true, true, false, false) => factory.factory::<SurfaceVertexDSP>(),
                (true, false, true, true) => factory.factory::<SurfaceVertexDPTC>(),
                (true, false, true, false) => factory.factory::<SurfaceVertexDPT>(),
                (true, false, false, true) => factory.factory::<SurfaceVertexDPC>(),
                (true, false, false, false) => factory.factory::<SurfaceVertexDP>(),
                (false, true, true, true) => factory.factory::<SurfaceVertexSPTC>(),
                (false, true, true, false) => factory.factory::<SurfaceVertexSPT>(),
                (false, true, false, true) => factory.factory::<SurfaceVertexSPC>(),
                (false, true, false, false) => factory.factory::<SurfaceVertexSP>(),
                (false, false, true, true) => factory.factory::<SurfaceVertexPTC>(),
                (false, false, true, false) => factory.factory::<SurfaceVertexPT>(),
                (false, false, false, true) => factory.factory::<SurfaceVertexPC>(),
                (false, false, false, false) => factory.factory::<SurfaceVertexP>(),
            },
            Self::Rig { asset, factory } => {
                let rig = match asset {
                    AssetVariant::Id(id) => assets.asset_by_id(*id),
                    AssetVariant::Path(path) => assets.asset_by_path(path),
                }
                .and_then(|asset| asset.get::<RigAsset>());
                let rig = match rig {
                    Some(rig) => rig,
                    None => {
                        return Err(MeshError::Internal(format!(
                            "Could not find Rig asset: {:?}",
                            asset
                        )))
                    }
                };
                let rig = match rig.rig() {
                    Ok(rig) => rig,
                    Err(error) => {
                        return Err(MeshError::Internal(format!(
                            "Could not get Rig from asset: {:?}. Error: {:?}",
                            asset, error
                        )))
                    }
                };
                match (deforming, skinning, texture, color) {
                    (true, true, true, true) => factory.factory::<SurfaceVertexDSPTC>(&rig),
                    (true, true, true, false) => factory.factory::<SurfaceVertexDSPT>(&rig),
                    (true, true, false, true) => factory.factory::<SurfaceVertexDSPC>(&rig),
                    (true, true, false, false) => factory.factory::<SurfaceVertexDSP>(&rig),
                    (true, false, true, true) => factory.factory::<SurfaceVertexDPTC>(&rig),
                    (true, false, true, false) => factory.factory::<SurfaceVertexDPT>(&rig),
                    (true, false, false, true) => factory.factory::<SurfaceVertexDPC>(&rig),
                    (true, false, false, false) => factory.factory::<SurfaceVertexDP>(&rig),
                    (false, true, true, true) => factory.factory::<SurfaceVertexSPTC>(&rig),
                    (false, true, true, false) => factory.factory::<SurfaceVertexSPT>(&rig),
                    (false, true, false, true) => factory.factory::<SurfaceVertexSPC>(&rig),
                    (false, true, false, false) => factory.factory::<SurfaceVertexSP>(&rig),
                    (false, false, true, true) => factory.factory::<SurfaceVertexPTC>(&rig),
                    (false, false, true, false) => factory.factory::<SurfaceVertexPT>(&rig),
                    (false, false, false, true) => factory.factory::<SurfaceVertexPC>(&rig),
                    (false, false, false, false) => factory.factory::<SurfaceVertexP>(&rig),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceMeshAsset {
    #[serde(default)]
    pub vertex_data: MeshVertexData,
    pub factory: SurfaceFactory,
}

impl SurfaceMeshAsset {
    pub fn factory(&self, assets: &AssetsDatabase) -> Result<StaticVertexFactory, MeshError> {
        self.factory.factory(self.vertex_data, assets)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ScreenSpaceMeshAsset(pub ScreenSpaceQuadFactory);

impl ScreenSpaceMeshAsset {
    pub fn factory(&self) -> Result<StaticVertexFactory, MeshError> {
        self.0.factory()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryFactory(pub Geometry);

impl GeometryFactory {
    pub fn factory(&self, data: MeshVertexData) -> Result<StaticVertexFactory, MeshError> {
        let MeshVertexData {
            deforming,
            skinning,
            texture,
            color,
        } = data;
        match (deforming, skinning, texture, color) {
            (true, true, true, true) => self.0.factory::<SurfaceVertexDSPTC>(),
            (true, true, true, false) => self.0.factory::<SurfaceVertexDSPT>(),
            (true, true, false, true) => self.0.factory::<SurfaceVertexDSPC>(),
            (true, true, false, false) => self.0.factory::<SurfaceVertexDSP>(),
            (true, false, true, true) => self.0.factory::<SurfaceVertexDPTC>(),
            (true, false, true, false) => self.0.factory::<SurfaceVertexDPT>(),
            (true, false, false, true) => self.0.factory::<SurfaceVertexDPC>(),
            (true, false, false, false) => self.0.factory::<SurfaceVertexDP>(),
            (false, true, true, true) => self.0.factory::<SurfaceVertexSPTC>(),
            (false, true, true, false) => self.0.factory::<SurfaceVertexSPT>(),
            (false, true, false, true) => self.0.factory::<SurfaceVertexSPC>(),
            (false, true, false, false) => self.0.factory::<SurfaceVertexSP>(),
            (false, false, true, true) => self.0.factory::<SurfaceVertexPTC>(),
            (false, false, true, false) => self.0.factory::<SurfaceVertexPT>(),
            (false, false, false, true) => self.0.factory::<SurfaceVertexPC>(),
            (false, false, false, false) => self.0.factory::<SurfaceVertexP>(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryMeshAsset {
    pub vertex_data: MeshVertexData,
    pub factory: GeometryFactory,
}

impl GeometryMeshAsset {
    pub fn factory(&self) -> Result<StaticVertexFactory, MeshError> {
        self.factory.factory(self.vertex_data)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeshAsset {
    Surface(SurfaceMeshAsset),
    ScreenSpace(ScreenSpaceMeshAsset),
    Geometry(GeometryMeshAsset),
    Raw(StaticVertexFactory),
}

impl MeshAsset {
    pub fn factory(&self, assets: &AssetsDatabase) -> Result<StaticVertexFactory, MeshError> {
        match self {
            Self::Surface(surface) => surface.factory(assets),
            Self::ScreenSpace(screenspace) => screenspace.factory(),
            Self::Geometry(geometry) => geometry.factory(),
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
        } else {
            bincode::deserialize::<MeshAsset>(&data).unwrap()
        };
        if let MeshAsset::Surface(asset) = &mesh {
            if let SurfaceFactory::Rig {
                asset: AssetVariant::Path(name),
                ..
            } = &asset.factory
            {
                let to_load = vec![("rig".to_owned(), format!("rig://{}", name))];
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
        if let MeshAsset::Surface(asset) = &mut mesh {
            if let SurfaceFactory::Rig { asset, .. } = &mut asset.factory {
                *asset = AssetVariant::Id(id);
            }
        }
        AssetLoadResult::Data(Box::new(mesh))
    }

    fn on_unload(&mut self, asset: &Asset) -> Option<Vec<AssetVariant>> {
        let id = asset.id();
        let mesh = asset.get::<MeshAsset>().unwrap();
        if let MeshAsset::Surface(asset) = &mesh {
            if let SurfaceFactory::Rig { .. } = &asset.factory {
                return Some(vec![AssetVariant::Id(id)]);
            }
        }
        None
    }
}
