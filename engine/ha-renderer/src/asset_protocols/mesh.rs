use crate::{
    material::domains::{
        screenspace::ScreenSpaceQuadFactory,
        surface::{
            circle::SurfaceCircleFactory, grid::SurfaceGridFactory, quad::SurfaceQuadFactory,
            triangles2d::SurfaceTriangles2dFactory, SurfaceVertexP, SurfaceVertexPC,
            SurfaceVertexPN, SurfaceVertexPNC, SurfaceVertexPNT, SurfaceVertexPNTC,
            SurfaceVertexPT, SurfaceVertexPTC,
        },
    },
    mesh::{vertex_factory::StaticVertexFactory, MeshError},
};
use core::{
    assets::protocol::{AssetLoadResult, AssetProtocol},
    Ignite,
};
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceDomainType {
    Position,
    PositionNormal,
    PositionTexture,
    PositionNormalTexture,
    PositionColor,
    PositionNormalColor,
    PositionTextureColor,
    PositionNormalTextureColor,
}

impl Default for SurfaceDomainType {
    fn default() -> Self {
        Self::Position
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum SurfaceFactory {
    Quad(SurfaceQuadFactory),
    Grid(SurfaceGridFactory),
    Circle(SurfaceCircleFactory),
    Triangles2d(SurfaceTriangles2dFactory),
}

impl Default for SurfaceFactory {
    fn default() -> Self {
        Self::Quad(Default::default())
    }
}

impl SurfaceFactory {
    pub fn factory(&self, domain: SurfaceDomainType) -> Result<StaticVertexFactory, MeshError> {
        match self {
            Self::Quad(factory) => match domain {
                SurfaceDomainType::Position => factory.factory::<SurfaceVertexP>(),
                SurfaceDomainType::PositionNormal => factory.factory::<SurfaceVertexPN>(),
                SurfaceDomainType::PositionTexture => factory.factory::<SurfaceVertexPT>(),
                SurfaceDomainType::PositionNormalTexture => factory.factory::<SurfaceVertexPNT>(),
                SurfaceDomainType::PositionColor => factory.factory::<SurfaceVertexPC>(),
                SurfaceDomainType::PositionNormalColor => factory.factory::<SurfaceVertexPNC>(),
                SurfaceDomainType::PositionTextureColor => factory.factory::<SurfaceVertexPTC>(),
                SurfaceDomainType::PositionNormalTextureColor => {
                    factory.factory::<SurfaceVertexPNTC>()
                }
            },
            Self::Grid(factory) => match domain {
                SurfaceDomainType::Position => factory.factory::<SurfaceVertexP>(),
                SurfaceDomainType::PositionNormal => factory.factory::<SurfaceVertexPN>(),
                SurfaceDomainType::PositionTexture => factory.factory::<SurfaceVertexPT>(),
                SurfaceDomainType::PositionNormalTexture => factory.factory::<SurfaceVertexPNT>(),
                SurfaceDomainType::PositionColor => factory.factory::<SurfaceVertexPC>(),
                SurfaceDomainType::PositionNormalColor => factory.factory::<SurfaceVertexPNC>(),
                SurfaceDomainType::PositionTextureColor => factory.factory::<SurfaceVertexPTC>(),
                SurfaceDomainType::PositionNormalTextureColor => {
                    factory.factory::<SurfaceVertexPNTC>()
                }
            },
            Self::Circle(factory) => match domain {
                SurfaceDomainType::Position => factory.factory::<SurfaceVertexP>(),
                SurfaceDomainType::PositionNormal => factory.factory::<SurfaceVertexPN>(),
                SurfaceDomainType::PositionTexture => factory.factory::<SurfaceVertexPT>(),
                SurfaceDomainType::PositionNormalTexture => factory.factory::<SurfaceVertexPNT>(),
                SurfaceDomainType::PositionColor => factory.factory::<SurfaceVertexPC>(),
                SurfaceDomainType::PositionNormalColor => factory.factory::<SurfaceVertexPNC>(),
                SurfaceDomainType::PositionTextureColor => factory.factory::<SurfaceVertexPTC>(),
                SurfaceDomainType::PositionNormalTextureColor => {
                    factory.factory::<SurfaceVertexPNTC>()
                }
            },
            Self::Triangles2d(factory) => match domain {
                SurfaceDomainType::Position => factory.factory::<SurfaceVertexP>(),
                SurfaceDomainType::PositionNormal => factory.factory::<SurfaceVertexPN>(),
                SurfaceDomainType::PositionTexture => factory.factory::<SurfaceVertexPT>(),
                SurfaceDomainType::PositionNormalTexture => factory.factory::<SurfaceVertexPNT>(),
                SurfaceDomainType::PositionColor => factory.factory::<SurfaceVertexPC>(),
                SurfaceDomainType::PositionNormalColor => factory.factory::<SurfaceVertexPNC>(),
                SurfaceDomainType::PositionTextureColor => factory.factory::<SurfaceVertexPTC>(),
                SurfaceDomainType::PositionNormalTextureColor => {
                    factory.factory::<SurfaceVertexPNTC>()
                }
            },
        }
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct SurfaceMeshAsset {
    pub domain: SurfaceDomainType,
    pub factory: SurfaceFactory,
}

impl SurfaceMeshAsset {
    pub fn factory(&self) -> Result<StaticVertexFactory, MeshError> {
        self.factory.factory(self.domain)
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
    ScreenSpace(ScreenSpaceMeshAsset),
    Raw(StaticVertexFactory),
}

impl Default for MeshAsset {
    fn default() -> Self {
        Self::Surface(Default::default())
    }
}

impl MeshAsset {
    pub fn factory(&self) -> Result<StaticVertexFactory, MeshError> {
        match self {
            Self::Surface(surface) => surface.factory(),
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
        AssetLoadResult::Data(Box::new(mesh))
    }

    // on_load_with_path() handles loading so this is not needed, so we just make it unreachable.
    fn on_load(&mut self, _data: Vec<u8>) -> AssetLoadResult {
        unreachable!()
    }
}
