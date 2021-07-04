use crate::{component::CompositeTransform, composite_renderer::TriangleFace, math::Vec2};
use core::{
    assets::protocol::{AssetLoadResult, AssetProtocol},
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::from_utf8};

#[derive(Ignite, Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct MeshFace {
    pub a: usize,
    pub b: usize,
    pub c: usize,
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct MeshVertexBoneInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub weight: Scalar,
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct MeshVertex {
    pub position: Vec2,
    pub tex_coord: Vec2,
    #[serde(default)]
    pub bone_info: Vec<MeshVertexBoneInfo>,
}

impl MeshVertex {
    pub fn fix_bone_info(&mut self) {
        let total_weight = self.bone_info.iter().map(|i| i.weight).sum::<Scalar>();
        if total_weight > 0.0 {
            for info in &mut self.bone_info {
                info.weight /= total_weight;
            }
        }
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct MeshBone {
    pub transform: CompositeTransform,
    #[serde(default)]
    pub children: HashMap<String, MeshBone>,
}

impl MeshBone {
    pub fn bones_count(&self) -> usize {
        self.children
            .values()
            .map(|child| child.bones_count())
            .sum::<usize>()
            + 1
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct SubMesh {
    pub faces: Vec<MeshFace>,
    #[serde(default)]
    pub masks: Vec<usize>,
    #[serde(skip)]
    #[ignite(ignore)]
    cached_faces: Vec<TriangleFace>,
}

impl SubMesh {
    pub fn cached_faces(&self) -> &[TriangleFace] {
        &self.cached_faces
    }

    pub fn cache(&mut self) {
        self.cached_faces = self
            .faces
            .iter()
            .map(|f| TriangleFace::new(f.a, f.b, f.c))
            .collect::<Vec<_>>()
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct MeshMask {
    pub indices: Vec<usize>,
    #[serde(default = "MeshMask::default_enabled")]
    pub enabled: bool,
}

impl MeshMask {
    fn default_enabled() -> bool {
        true
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: Vec<MeshVertex>,
    pub submeshes: Vec<SubMesh>,
    #[serde(default)]
    pub masks: Vec<MeshMask>,
    #[serde(default)]
    pub rig: Option<MeshBone>,
}

impl Mesh {
    pub fn initialize(&mut self) {
        for vertex in &mut self.vertices {
            vertex.fix_bone_info();
        }
        for submesh in &mut self.submeshes {
            submesh.cache();
        }
    }
}

pub struct MeshAsset(Mesh);

impl MeshAsset {
    pub fn mesh(&self) -> &Mesh {
        &self.0
    }
}

pub struct MeshAssetProtocol;

impl AssetProtocol for MeshAssetProtocol {
    fn name(&self) -> &str {
        "mesh"
    }

    fn on_load_with_path(&mut self, path: &str, data: Vec<u8>) -> AssetLoadResult {
        let mut mesh = if path.ends_with(".json") {
            let data = from_utf8(&data).unwrap();
            serde_json::from_str::<Mesh>(&data).unwrap()
        } else if path.ends_with(".yaml") {
            let data = from_utf8(&data).unwrap();
            serde_yaml::from_str::<Mesh>(&data).unwrap()
        } else {
            bincode::deserialize::<Mesh>(&data).unwrap()
        };
        mesh.initialize();
        AssetLoadResult::Data(Box::new(MeshAsset(mesh)))
    }

    // on_load_with_path() handles loading so this is not needed, so we just make it unreachable.
    fn on_load(&mut self, _data: Vec<u8>) -> AssetLoadResult {
        unreachable!()
    }
}
