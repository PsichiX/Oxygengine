use anims::phase::Phase;
use core::{
    assets::protocol::{AssetLoadResult, AssetProtocol},
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::from_utf8};

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct MeshAnimationSequence {
    #[serde(default)]
    pub submesh_alpha: HashMap<usize, Phase>,
    #[serde(default)]
    pub submesh_order: HashMap<usize, Phase>,
    #[serde(default)]
    pub bone_position_x: HashMap<String, Phase>,
    #[serde(default)]
    pub bone_position_y: HashMap<String, Phase>,
    #[serde(default)]
    pub bone_rotation: HashMap<String, Phase>,
    #[serde(default)]
    pub bone_scale_x: HashMap<String, Phase>,
    #[serde(default)]
    pub bone_scale_y: HashMap<String, Phase>,
    #[serde(skip)]
    #[ignite(ignore)]
    length: Scalar,
}

impl MeshAnimationSequence {
    pub fn initialize(&mut self) {
        self.length = 0.0;
        for key_frames in self.submesh_alpha.values() {
            self.length = self.length.max(key_frames.duration());
        }
        for key_frames in self.submesh_order.values() {
            self.length = self.length.max(key_frames.duration());
        }
        for key_frames in self.bone_position_x.values() {
            self.length = self.length.max(key_frames.duration());
        }
        for key_frames in self.bone_position_y.values() {
            self.length = self.length.max(key_frames.duration());
        }
        for key_frames in self.bone_rotation.values() {
            self.length = self.length.max(key_frames.duration());
        }
        for key_frames in self.bone_scale_x.values() {
            self.length = self.length.max(key_frames.duration());
        }
        for key_frames in self.bone_scale_y.values() {
            self.length = self.length.max(key_frames.duration());
        }
    }

    pub fn length(&self) -> Scalar {
        self.length
    }

    pub fn sample_submesh_alpha(&self, time: Scalar, index: usize, current: Scalar) -> Scalar {
        Self::sample_indexed(&self.submesh_alpha, time, index, current)
    }

    pub fn sample_submesh_order(&self, time: Scalar, index: usize, current: Scalar) -> Scalar {
        Self::sample_indexed(&self.submesh_order, time, index, current)
    }

    pub fn sample_bone_position_x(&self, time: Scalar, name: &str, current: Scalar) -> Scalar {
        Self::sample_named(&self.bone_position_x, time, name, current)
    }

    pub fn sample_bone_position_y(&self, time: Scalar, name: &str, current: Scalar) -> Scalar {
        Self::sample_named(&self.bone_position_y, time, name, current)
    }

    pub fn sample_bone_rotation(&self, time: Scalar, name: &str, current: Scalar) -> Scalar {
        Self::sample_named(&self.bone_rotation, time, name, current)
    }

    pub fn sample_bone_scale_x(&self, time: Scalar, name: &str, current: Scalar) -> Scalar {
        Self::sample_named(&self.bone_scale_x, time, name, current)
    }

    pub fn sample_bone_scale_y(&self, time: Scalar, name: &str, current: Scalar) -> Scalar {
        Self::sample_named(&self.bone_scale_y, time, name, current)
    }

    fn sample_key_frames(key_frames: &Phase, time: Scalar, current: Scalar) -> Scalar {
        if key_frames.points().is_empty() {
            current
        } else {
            key_frames.sample(time)
        }
    }

    fn sample_indexed(
        database: &HashMap<usize, Phase>,
        time: Scalar,
        index: usize,
        current: Scalar,
    ) -> Scalar {
        database
            .get(&index)
            .map(|key_frames| Self::sample_key_frames(key_frames, time, current))
            .unwrap_or(current)
    }

    fn sample_named(
        database: &HashMap<String, Phase>,
        time: Scalar,
        name: &str,
        current: Scalar,
    ) -> Scalar {
        database
            .get(name)
            .map(|key_frames| Self::sample_key_frames(key_frames, time, current))
            .unwrap_or(current)
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct MeshAnimation {
    pub sequences: HashMap<String, MeshAnimationSequence>,
}

impl MeshAnimation {
    pub fn initialize(&mut self) {
        for seq in self.sequences.values_mut() {
            seq.initialize();
        }
    }
}

pub struct MeshAnimationAsset(MeshAnimation);

impl MeshAnimationAsset {
    pub fn animation(&self) -> &MeshAnimation {
        &self.0
    }
}

pub struct MeshAnimationAssetProtocol;

impl AssetProtocol for MeshAnimationAssetProtocol {
    fn name(&self) -> &str {
        "mesh-anim"
    }

    fn on_load_with_path(&mut self, path: &str, data: Vec<u8>) -> AssetLoadResult {
        let mut anim = if path.ends_with(".json") {
            let data = from_utf8(&data).unwrap();
            serde_json::from_str::<MeshAnimation>(&data).unwrap()
        } else if path.ends_with(".yaml") {
            let data = from_utf8(&data).unwrap();
            serde_yaml::from_str::<MeshAnimation>(&data).unwrap()
        } else {
            bincode::deserialize::<MeshAnimation>(&data).unwrap()
        };
        anim.initialize();
        AssetLoadResult::Data(Box::new(MeshAnimationAsset(anim)))
    }

    // on_load_with_path() handles loading so this is not needed, so we just make it unreachable.
    fn on_load(&mut self, _data: Vec<u8>) -> AssetLoadResult {
        unreachable!()
    }
}
