use crate::{
    components::transform::HaTransform,
    image::{Image, ImageError},
    math::Mat4,
    mesh::skeleton::Skeleton,
};
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct HaSkeletonInstance {
    #[serde(default)]
    bone_transforms: HashMap<String, HaTransform>,
    #[serde(skip)]
    #[ignite(ignore)]
    hierarchy_matrices: Vec<Mat4>,
    #[serde(skip)]
    #[ignite(ignore)]
    bone_matrices: Vec<Mat4>,
    #[serde(default)]
    skeleton: String,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) dirty: bool,
}

impl Default for HaSkeletonInstance {
    fn default() -> Self {
        Self {
            bone_transforms: Default::default(),
            hierarchy_matrices: Default::default(),
            bone_matrices: Default::default(),
            skeleton: Default::default(),
            dirty: true,
        }
    }
}

impl HaSkeletonInstance {
    pub fn bone_transforms(&self) -> impl Iterator<Item = (&str, &HaTransform)> {
        self.bone_transforms.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn has_bone_transform(&self, name: &str) -> bool {
        self.bone_transforms.contains_key(name)
    }

    pub fn bone_transform(&self, name: &str) -> Option<&HaTransform> {
        self.bone_transforms.get(name)
    }

    pub fn set_bone_transform(&mut self, name: impl ToString, transform: HaTransform) {
        self.bone_transforms.insert(name.to_string(), transform);
        self.dirty = true;
    }

    pub fn unset_bone_transform(&mut self, name: &str) {
        self.bone_transforms.remove(name);
        self.dirty = true;
    }

    pub fn with_existing_bone_transforms<F>(&mut self, mut f: F)
    where
        F: FnMut(&str, &mut HaTransform),
    {
        for (name, transform) in self.bone_transforms.iter_mut() {
            f(name, transform);
        }
        self.dirty = true;
    }

    pub fn with_bone_transform<F>(&mut self, name: String, mut f: F)
    where
        F: FnMut(&mut HaTransform),
    {
        let transform = self.bone_transforms.entry(name).or_default();
        f(transform);
        self.dirty = true;
    }

    pub fn bone_matrices(&self) -> &[Mat4] {
        &self.bone_matrices
    }

    pub fn skeleton(&self) -> &str {
        &self.skeleton
    }

    pub fn set_skeleton(&mut self, skeleton: impl ToString) {
        self.skeleton = skeleton.to_string();
        self.dirty = true;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub(crate) fn try_initialize_bone_transforms(&mut self, skeleton: &Skeleton) {
        for bone in skeleton.bones().iter() {
            if !self.bone_transforms.contains_key(bone.name()) {
                self.bone_transforms
                    .insert(bone.name().to_owned(), bone.transform().to_owned());
            }
        }
    }

    pub(crate) fn recalculate_bone_matrices(&mut self, skeleton: &Skeleton) {
        self.hierarchy_matrices.clear();
        self.hierarchy_matrices.reserve(skeleton.bones().len());
        self.bone_matrices.clear();
        self.bone_matrices.reserve(skeleton.bones().len());
        for bone in skeleton.bones().iter() {
            let local_matrix = self
                .bone_transforms
                .get(bone.name())
                .map(|transform| transform.local_matrix())
                .unwrap_or_default();
            let parent_matrix = bone
                .parent()
                .and_then(|index| self.hierarchy_matrices.get(index))
                .copied()
                .unwrap_or_default();
            self.hierarchy_matrices.push(parent_matrix * local_matrix);
        }
        for (matrix, bone) in self.hierarchy_matrices.iter().zip(skeleton.bones().iter()) {
            self.bone_matrices
                .push(*matrix * bone.bind_pose_inverse_matrix());
        }
    }

    pub(crate) fn apply_bone_matrices_data(&mut self, image: &mut Image) -> Result<(), ImageError> {
        let mut data = Vec::with_capacity(self.bone_matrices.len() * image.format().bytesize() * 4);
        for matrix in &self.bone_matrices {
            let values = matrix.as_col_slice();
            let bytes = unsafe { values.align_to::<u8>().1 };
            data.extend(bytes);
        }
        image.overwrite(4, self.bone_matrices.len(), 1, data)
    }
}

impl Prefab for HaSkeletonInstance {}
impl PrefabComponent for HaSkeletonInstance {}
