use crate::{
    components::transform::HaTransform,
    math::{Mat4, Vec3},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkeletonError {
    /// (parent index, bones count)
    InvalidParentIndex(usize, usize),
    /// (parent index, bone index)
    InvalidBonesOrder(usize, usize),
    /// (bone name)
    DuplicateBoneName(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkeletonHierarchy {
    bone_name: String,
    bone_target: Vec3,
    transform: HaTransform,
    children: Vec<SkeletonHierarchy>,
}

impl SkeletonHierarchy {
    pub fn new(bone_name: impl ToString) -> Self {
        Self {
            bone_name: bone_name.to_string(),
            bone_target: Default::default(),
            transform: Default::default(),
            children: vec![],
        }
    }

    pub fn target(mut self, offset: Vec3) -> Self {
        self.bone_target = offset;
        self
    }

    pub fn transform(mut self, transform: HaTransform) -> Self {
        self.transform = transform;
        self
    }

    pub fn child(mut self, hierarchy: Self) -> Self {
        self.children.push(hierarchy);
        self
    }

    fn count(&self) -> usize {
        self.children.iter().fold(1, |a, v| a + v.count())
    }

    fn visit<F>(&self, parent: Option<usize>, f: &mut F)
    where
        F: FnMut(&Self, Option<usize>) -> Option<usize>,
    {
        if let Some(parent) = f(self, parent) {
            for child in &self.children {
                child.visit(Some(parent), f);
            }
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SkeletonBone {
    parent: Option<usize>,
    name: String,
    target: Vec3,
    transform: HaTransform,
}

impl SkeletonBone {
    pub fn parent(&self) -> Option<usize> {
        self.parent
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn target(&self) -> Vec3 {
        self.target
    }

    pub fn transform(&self) -> &HaTransform {
        &self.transform
    }

    pub fn local_matrix(&self) -> Mat4 {
        self.transform.local_matrix()
    }

    pub fn bind_pose_matrix(&self) -> Mat4 {
        self.transform.world_matrix()
    }

    pub fn bind_pose_inverse_matrix(&self) -> Mat4 {
        self.transform.inverse_world_matrix()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Skeleton {
    bones: Vec<SkeletonBone>,
}

impl Skeleton {
    pub fn new(bones: Vec<SkeletonBone>) -> Result<Self, SkeletonError> {
        for (index, bone) in bones.iter().enumerate() {
            if let Some(parent) = bone.parent() {
                if parent >= index {
                    return Err(SkeletonError::InvalidBonesOrder(parent, index));
                }
                if parent >= bones.len() {
                    return Err(SkeletonError::InvalidParentIndex(parent, bones.len()));
                }
            }
            if bones
                .iter()
                .enumerate()
                .any(|(i, other)| i != index && other.name == bone.name)
            {
                return Err(SkeletonError::DuplicateBoneName(bone.name.to_owned()));
            }
        }
        Ok(Self { bones })
    }

    pub fn root_bone(&self) -> Option<&SkeletonBone> {
        self.bones.get(0)
    }

    pub fn has_bone_by_index(&self, index: usize) -> bool {
        index < self.bones.len()
    }

    pub fn bone_by_index(&self, index: usize) -> Option<&SkeletonBone> {
        self.bones.get(index)
    }

    pub fn has_bone_by_name(&self, name: &str) -> bool {
        self.bones.iter().any(|bone| bone.name == name)
    }

    pub fn bone_by_name(&self, name: &str) -> Option<&SkeletonBone> {
        self.bones.iter().find(|bone| bone.name == name)
    }

    pub fn bone_index(&self, name: &str) -> Option<usize> {
        self.bones.iter().position(|bone| bone.name == name)
    }

    pub fn bone_with_index(&self, name: &str) -> Option<(&SkeletonBone, usize)> {
        self.bones.iter().enumerate().find_map(|(index, bone)| {
            if bone.name == name {
                Some((bone, index))
            } else {
                None
            }
        })
    }

    pub fn bones(&self) -> &[SkeletonBone] {
        &self.bones
    }

    pub fn children_by_index(&self, index: usize) -> impl Iterator<Item = &SkeletonBone> {
        self.bones.iter().filter(move |bone| {
            bone.parent
                .map(|parent| parent == index)
                .unwrap_or_default()
        })
    }
}

impl TryFrom<SkeletonHierarchy> for Skeleton {
    type Error = SkeletonError;

    fn try_from(hierarchy: SkeletonHierarchy) -> Result<Self, Self::Error> {
        let count = hierarchy.count();
        let mut bones = Vec::<SkeletonBone>::with_capacity(count);

        hierarchy.visit(None, &mut |hierarchy, parent| {
            if bones.iter().any(|bone| bone.name == hierarchy.bone_name) {
                return None;
            }

            let mut transform = hierarchy.transform.to_owned();
            transform.rebuild_world_matrix(parent.map(|index| bones[index].transform()));
            bones.push(SkeletonBone {
                parent,
                name: hierarchy.bone_name.to_owned(),
                target: hierarchy.bone_target,
                transform,
            });
            Some(bones.len() - 1)
        });

        Self::new(bones)
    }
}
