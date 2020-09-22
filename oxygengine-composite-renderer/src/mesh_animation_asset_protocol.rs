use crate::math::lerp;
use core::{
    assets::protocol::{AssetLoadResult, AssetProtocol},
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::from_utf8};

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct MeshAnimationKeyFrame {
    pub time: Scalar,
    pub value: Scalar,
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct MeshAnimationSequence {
    #[serde(default)]
    pub submesh_alpha: HashMap<usize, Vec<MeshAnimationKeyFrame>>,
    #[serde(default)]
    pub submesh_order: HashMap<usize, Vec<MeshAnimationKeyFrame>>,
    #[serde(default)]
    pub bone_position_x: HashMap<String, Vec<MeshAnimationKeyFrame>>,
    #[serde(default)]
    pub bone_position_y: HashMap<String, Vec<MeshAnimationKeyFrame>>,
    #[serde(default)]
    pub bone_rotation: HashMap<String, Vec<MeshAnimationKeyFrame>>,
    #[serde(default)]
    pub bone_scale_x: HashMap<String, Vec<MeshAnimationKeyFrame>>,
    #[serde(default)]
    pub bone_scale_y: HashMap<String, Vec<MeshAnimationKeyFrame>>,
    #[serde(skip)]
    #[ignite(ignore)]
    length: Scalar,
}

impl MeshAnimationSequence {
    pub fn initialize(&mut self) {
        for key_frames in self.submesh_alpha.values_mut() {
            key_frames.retain(|v| v.time >= 0.0);
        }
        for key_frames in self.submesh_order.values_mut() {
            key_frames.retain(|v| v.time >= 0.0);
        }
        for key_frames in self.bone_position_x.values_mut() {
            key_frames.retain(|v| v.time >= 0.0);
        }
        for key_frames in self.bone_position_y.values_mut() {
            key_frames.retain(|v| v.time >= 0.0);
        }
        for key_frames in self.bone_rotation.values_mut() {
            key_frames.retain(|v| v.time >= 0.0);
        }
        for key_frames in self.bone_scale_x.values_mut() {
            key_frames.retain(|v| v.time >= 0.0);
        }
        for key_frames in self.bone_scale_y.values_mut() {
            key_frames.retain(|v| v.time >= 0.0);
        }
        self.submesh_alpha.retain(|_, v| !v.is_empty());
        self.submesh_order.retain(|_, v| !v.is_empty());
        self.bone_position_x.retain(|_, v| !v.is_empty());
        self.bone_position_y.retain(|_, v| !v.is_empty());
        self.bone_rotation.retain(|_, v| !v.is_empty());
        self.bone_scale_x.retain(|_, v| !v.is_empty());
        self.bone_scale_y.retain(|_, v| !v.is_empty());
        for key_frames in self.submesh_alpha.values_mut() {
            key_frames.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        }
        for key_frames in self.submesh_order.values_mut() {
            key_frames.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        }
        for key_frames in self.bone_position_x.values_mut() {
            key_frames.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        }
        for key_frames in self.bone_position_y.values_mut() {
            key_frames.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        }
        for key_frames in self.bone_rotation.values_mut() {
            key_frames.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        }
        for key_frames in self.bone_scale_x.values_mut() {
            key_frames.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        }
        for key_frames in self.bone_scale_y.values_mut() {
            key_frames.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        }
        self.length = 0.0;
        for key_frames in self.submesh_alpha.values() {
            for v in key_frames {
                self.length = self.length.max(v.time);
            }
        }
        for key_frames in self.submesh_order.values() {
            for v in key_frames {
                self.length = self.length.max(v.time);
            }
        }
        for key_frames in self.bone_position_x.values() {
            for v in key_frames {
                self.length = self.length.max(v.time);
            }
        }
        for key_frames in self.bone_position_y.values() {
            for v in key_frames {
                self.length = self.length.max(v.time);
            }
        }
        for key_frames in self.bone_rotation.values() {
            for v in key_frames {
                self.length = self.length.max(v.time);
            }
        }
        for key_frames in self.bone_scale_x.values() {
            for v in key_frames {
                self.length = self.length.max(v.time);
            }
        }
        for key_frames in self.bone_scale_y.values() {
            for v in key_frames {
                self.length = self.length.max(v.time);
            }
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

    fn sample_key_frames(
        key_frames: &[MeshAnimationKeyFrame],
        time: Scalar,
        current: Scalar,
    ) -> Scalar {
        if key_frames.is_empty() {
            return current;
        }
        match key_frames.binary_search_by(|v| v.time.partial_cmp(&time).unwrap()) {
            Ok(index) => key_frames[index].value,
            Err(index) => {
                if index == 0 {
                    key_frames.first().unwrap().value
                } else if index >= key_frames.len() {
                    key_frames.last().unwrap().value
                } else {
                    let a = &key_frames[index - 1];
                    let b = &key_frames[index];
                    let f = (time - a.time) / (b.time - a.time);
                    lerp(a.value, b.value, f)
                }
            }
        }
    }

    fn sample_indexed(
        database: &HashMap<usize, Vec<MeshAnimationKeyFrame>>,
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
        database: &HashMap<String, Vec<MeshAnimationKeyFrame>>,
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
