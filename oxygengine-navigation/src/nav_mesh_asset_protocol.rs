use crate::resource::{NavMesh, NavResult, NavTriangle, NavVec3};
use bincode::deserialize;
use core::{Ignite, assets::protocol::{AssetLoadResult, AssetProtocol}};
use serde::{Deserialize, Serialize};

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct NavMeshAsset {
    #[serde(default)]
    vertices: Vec<NavVec3>,
    #[serde(default)]
    triangles: Vec<NavTriangle>,
}

impl NavMeshAsset {
    pub fn vertices(&self) -> &[NavVec3] {
        &self.vertices
    }

    pub fn triangles(&self) -> &[NavTriangle] {
        &self.triangles
    }

    pub fn build_nav_mesh(&self) -> NavResult<NavMesh> {
        NavMesh::new(self.vertices.clone(), self.triangles.clone())
    }
}

pub struct NavMeshAssetProtocol;

impl AssetProtocol for NavMeshAssetProtocol {
    fn name(&self) -> &str {
        "navmesh"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        match deserialize::<NavMeshAsset>(&data) {
            Ok(asset) => AssetLoadResult::Data(Box::new(asset)),
            Err(error) => {
                AssetLoadResult::Error(format!("Error loading navmesh asset: {:?}", error))
            }
        }
    }
}
