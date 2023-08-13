use crate::{
    components::transform::HaTransform,
    image::{Image, ImageError},
    math::*,
    mesh::rig::{deformer::DeformerArea, skeleton::Skeleton, Rig},
};
use core::{
    prefab::{Prefab, PrefabComponent},
    scripting::intuicio::core::object::{DynamicObject, TypedDynamicObject},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    ops::{Deref, DerefMut},
};
use utils::prelude::Grid2d;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaRigSkeleton {
    #[serde(default)]
    bone_transforms: HashMap<String, HaTransform>,
    #[serde(skip)]
    hierarchy_matrices: Vec<Mat4>,
    #[serde(skip)]
    bone_matrices: Vec<Mat4>,
    #[serde(skip)]
    dirty: bool,
}

impl Default for HaRigSkeleton {
    fn default() -> Self {
        Self {
            bone_transforms: Default::default(),
            hierarchy_matrices: Default::default(),
            bone_matrices: Default::default(),
            dirty: true,
        }
    }
}

impl HaRigSkeleton {
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

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub(crate) fn unmark_dirty(&mut self) {
        self.dirty = false;
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

    pub(crate) fn apply_bone_matrices_data(&self, image: &mut Image) -> Result<(), ImageError> {
        let mut data = Vec::with_capacity(self.bone_matrices.len() * image.format().bytesize() * 4);
        for matrix in &self.bone_matrices {
            let values = matrix.as_col_slice();
            let bytes = unsafe { values.align_to::<u8>().1 };
            data.extend(bytes);
        }
        image.overwrite(4, self.bone_matrices.len(), 1, data)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HaDeformerTangent {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum HaDeformerTangents {
    Single(Vec2),
    Mirrored {
        horizontal: Vec2,
        vertical: Vec2,
    },
    Cardinal {
        north: Vec2,
        east: Vec2,
        south: Vec2,
        west: Vec2,
    },
}

impl HaDeformerTangents {
    pub fn to_single(&self) -> Self {
        match self {
            Self::Mirrored {
                horizontal,
                vertical,
            } => Self::Single((*horizontal + vec2(vertical.y, -vertical.x)) * 0.5),
            Self::Cardinal {
                north,
                east,
                south,
                west,
            } => Self::Single(
                (*east - *west + vec2(south.y, -south.x) + vec2(-north.y, north.x)) * 0.25,
            ),
            value => *value,
        }
    }

    pub fn to_mirrored(&self) -> Self {
        match self {
            Self::Single(tangent) => Self::Mirrored {
                horizontal: *tangent,
                vertical: vec2(-tangent.y, tangent.x),
            },
            Self::Cardinal {
                north,
                east,
                south,
                west,
            } => Self::Mirrored {
                horizontal: (*east - *west) * 0.5,
                vertical: (*south - *north) * 0.5,
            },
            value => *value,
        }
    }

    pub fn to_cardinal(&self) -> Self {
        match self {
            Self::Single(tangent) => Self::Cardinal {
                north: vec2(tangent.y, -tangent.x),
                east: *tangent,
                south: vec2(-tangent.y, tangent.x),
                west: -*tangent,
            },
            Self::Mirrored {
                horizontal,
                vertical,
            } => Self::Cardinal {
                north: -*vertical,
                east: *horizontal,
                south: *vertical,
                west: -*horizontal,
            },
            value => *value,
        }
    }

    pub fn tangent(&self, direction: HaDeformerTangent) -> Vec2 {
        match self {
            Self::Single(tangent) => match direction {
                HaDeformerTangent::North => vec2(tangent.y, -tangent.x),
                HaDeformerTangent::East => *tangent,
                HaDeformerTangent::South => vec2(-tangent.y, tangent.x),
                HaDeformerTangent::West => -*tangent,
            },
            Self::Mirrored {
                horizontal,
                vertical,
            } => match direction {
                HaDeformerTangent::North => -*vertical,
                HaDeformerTangent::East => *horizontal,
                HaDeformerTangent::South => *vertical,
                HaDeformerTangent::West => -*horizontal,
            },
            Self::Cardinal {
                north,
                east,
                south,
                west,
            } => match direction {
                HaDeformerTangent::North => *north,
                HaDeformerTangent::East => *east,
                HaDeformerTangent::South => *south,
                HaDeformerTangent::West => *west,
            },
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct HaDeformerControlPoint {
    pub position: Vec2,
    pub tangents: HaDeformerTangents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaDeformerArea {
    control_points: Grid2d<HaDeformerControlPoint>,
    dirty: bool,
}

impl HaDeformerArea {
    pub fn new(area: &DeformerArea) -> Self {
        let cols = area.cols.max(1);
        let rows = area.rows.max(1);
        let tangent_horizontal = vec2(area.rectangle.w / 3.0 / cols as f32, 0.0);
        let tangent_vertical = vec2(0.0, area.rectangle.h / 3.0 / rows as f32);
        let cells = (0..=rows)
            .flat_map(|row| {
                let py = area.rectangle.h * row as f32 / rows as f32;
                (0..=cols).map(move |col| {
                    let px = area.rectangle.w * col as f32 / cols as f32;
                    HaDeformerControlPoint {
                        position: area.rectangle.position() + vec2(px, py),
                        tangents: HaDeformerTangents::Mirrored {
                            horizontal: tangent_horizontal,
                            vertical: tangent_vertical,
                        },
                    }
                })
            })
            .collect();
        Self {
            control_points: Grid2d::with_cells(cols + 1, cells),
            dirty: true,
        }
    }

    pub fn get(&self, col: usize, row: usize) -> Option<HaDeformerControlPoint> {
        self.control_points.get(col, row)
    }

    pub fn set(&mut self, col: usize, row: usize, control_point: HaDeformerControlPoint) {
        self.control_points.set(col, row, control_point);
        self.dirty = true;
    }

    pub fn read(&mut self, col: usize, row: usize) -> Option<&HaDeformerControlPoint> {
        self.control_points.cell(col, row)
    }

    pub fn write(&mut self, col: usize, row: usize) -> Option<&mut HaDeformerControlPoint> {
        self.control_points.cell_mut(col, row)
    }

    pub fn cells_count(&self) -> usize {
        (self.control_points.cols() - 1) * (self.control_points.rows() - 1)
    }

    fn apply_data(&self, buffer: &mut [f32]) {
        let mut offset = 0;
        for row in 0..(self.control_points.rows() - 1) {
            for col in 0..(self.control_points.cols() - 1) {
                let tl = self.control_points.cell(col, row).unwrap();
                let tr = self.control_points.cell(col + 1, row).unwrap();
                let br = self.control_points.cell(col + 1, row + 1).unwrap();
                let bl = self.control_points.cell(col, row + 1).unwrap();
                let tle = tl.tangents.tangent(HaDeformerTangent::East);
                let tls = tl.tangents.tangent(HaDeformerTangent::South);
                let trw = tr.tangents.tangent(HaDeformerTangent::West);
                let trs = tr.tangents.tangent(HaDeformerTangent::South);
                let brw = br.tangents.tangent(HaDeformerTangent::West);
                let brn = br.tangents.tangent(HaDeformerTangent::North);
                let ble = bl.tangents.tangent(HaDeformerTangent::East);
                let bln = bl.tangents.tangent(HaDeformerTangent::North);
                let values = [
                    // top x
                    tl.position.x,
                    tl.position.x + tle.x,
                    tr.position.x + trw.x,
                    tr.position.x,
                    // top y
                    tl.position.y,
                    tl.position.y + tle.y,
                    tr.position.y + trw.y,
                    tr.position.y,
                    // bottom x
                    bl.position.x,
                    bl.position.x + ble.x,
                    br.position.x + brw.x,
                    br.position.x,
                    // bottom y
                    bl.position.y,
                    bl.position.y + ble.y,
                    br.position.y + brw.y,
                    br.position.y,
                    // left x
                    tl.position.x,
                    tl.position.x + tls.x,
                    bl.position.x + bln.x,
                    bl.position.x,
                    // left y
                    tl.position.y,
                    tl.position.y + tls.y,
                    bl.position.y + bln.y,
                    bl.position.y,
                    // right x
                    tr.position.x,
                    tr.position.x + trs.x,
                    br.position.x + brn.x,
                    br.position.x,
                    // right y
                    tr.position.y,
                    tr.position.y + trs.y,
                    br.position.y + brn.y,
                    br.position.y,
                ];
                buffer[offset..(offset + 32)].copy_from_slice(&values);
                offset += 32;
            }
        }
    }
}

pub struct HaDeformerAreaAccess<'a> {
    area: &'a mut HaDeformerArea,
    buffer: &'a mut [f32],
}

impl<'a> Deref for HaDeformerAreaAccess<'a> {
    type Target = HaDeformerArea;

    fn deref(&self) -> &Self::Target {
        self.area
    }
}

impl<'a> DerefMut for HaDeformerAreaAccess<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.area
    }
}

impl<'a> Drop for HaDeformerAreaAccess<'a> {
    fn drop(&mut self) {
        self.area.dirty = true;
        self.area.apply_data(self.buffer);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaRigDeformer {
    /// {name: (area, buffer offset, buffer items)}
    #[serde(default)]
    areas: BTreeMap<String, (HaDeformerArea, usize, usize)>,
    #[serde(skip)]
    buffer: Vec<f32>,
    #[serde(skip)]
    dirty: bool,
}

impl Default for HaRigDeformer {
    fn default() -> Self {
        Self {
            areas: Default::default(),
            buffer: Default::default(),
            dirty: true,
        }
    }
}

impl HaRigDeformer {
    pub fn areas(&self) -> impl Iterator<Item = (&str, &HaDeformerArea)> {
        self.areas
            .iter()
            .map(|(name, (area, _, _))| (name.as_str(), area))
    }

    pub fn has_area(&self, name: &str) -> bool {
        self.areas.contains_key(name)
    }

    pub fn read_area(&self, name: &str) -> Option<&HaDeformerArea> {
        self.areas.get(name).map(|(area, _, _)| area)
    }

    pub fn write_area(&mut self, name: &str) -> Option<HaDeformerAreaAccess> {
        self.areas
            .get_mut(name)
            .map(|(area, offset, count)| HaDeformerAreaAccess {
                area,
                buffer: &mut self.buffer[*offset..(*offset + *count)],
            })
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty || self.areas.values().any(|(area, _, _)| area.dirty)
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub(crate) fn unmark_dirty(&mut self) {
        self.dirty = false;
        for (area, _, _) in self.areas.values_mut() {
            area.dirty = false;
        }
    }

    pub(crate) fn apply_areas_data(&self, image: &mut Image) -> Result<(), ImageError> {
        let count = self
            .areas
            .values()
            .map(|(area, _, _)| area.cells_count())
            .sum();
        let bytes = unsafe { self.buffer.align_to::<u8>().1 };
        image.overwrite(8, count, 1, bytes.to_owned())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaRigControlNode {
    #[serde(default)]
    transform: HaTransform,
    #[serde(default)]
    parent_bone: Option<String>,
    #[serde(skip)]
    dirty: bool,
}

impl HaRigControlNode {
    pub fn with_transform(mut self, transform: HaTransform) -> Self {
        self.set_transform(transform);
        self
    }

    pub fn with_parent_bone(mut self, bone: Option<String>) -> Self {
        self.set_parent_bone(bone);
        self
    }

    pub fn transform(&self) -> &HaTransform {
        &self.transform
    }

    pub fn set_transform(&mut self, transform: HaTransform) {
        self.transform = transform;
        self.dirty = true;
    }

    pub fn parent_node(&self) -> Option<&str> {
        self.parent_bone.as_deref()
    }

    pub fn set_parent_bone(&mut self, bone: Option<String>) {
        self.parent_bone = bone;
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

#[derive(Serialize, Deserialize)]
pub struct HaRigControl {
    #[serde(default)]
    controls: HashMap<String, HaRigControlNode>,
    #[serde(skip)]
    pub named_properties: DynamicObject,
    #[serde(skip)]
    pub typed_properties: TypedDynamicObject,
    #[serde(skip)]
    dirty: bool,
}

impl Default for HaRigControl {
    fn default() -> Self {
        Self {
            controls: Default::default(),
            named_properties: Default::default(),
            typed_properties: Default::default(),
            dirty: true,
        }
    }
}

impl HaRigControl {
    pub fn is_dirty(&self) -> bool {
        self.dirty || self.controls.values().any(|node| node.is_dirty())
    }

    pub fn control(&self, name: &str) -> Option<&HaRigControlNode> {
        self.controls.get(name)
    }

    pub fn control_mut(&mut self, name: &str) -> Option<&mut HaRigControlNode> {
        self.dirty = true;
        self.controls.get_mut(name)
    }

    pub fn set_control(
        &mut self,
        name: impl ToString,
        node: HaRigControlNode,
    ) -> Option<HaRigControlNode> {
        self.dirty = true;
        self.controls.insert(name.to_string(), node)
    }
}

impl Clone for HaRigControl {
    fn clone(&self) -> Self {
        Self {
            controls: self.controls.clone(),
            named_properties: Default::default(),
            typed_properties: Default::default(),
            dirty: true,
        }
    }
}

impl std::fmt::Debug for HaRigControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HaRigControl")
            .field("controls", &self.controls)
            .field(
                "properties",
                &self.named_properties.property_names().collect::<Vec<_>>(),
            )
            .field("dirty", &self.dirty)
            .finish()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HaRigInstance {
    #[serde(default)]
    pub skeleton: HaRigSkeleton,
    #[serde(default)]
    pub deformer: HaRigDeformer,
    #[serde(default)]
    pub control: HaRigControl,
    #[serde(default)]
    asset: String,
    #[serde(skip)]
    ready: bool,
}

impl HaRigInstance {
    pub fn asset(&self) -> &str {
        &self.asset
    }

    pub fn set_asset(&mut self, asset: impl ToString) {
        self.asset = asset.to_string();
        self.skeleton.dirty = true;
        self.deformer.dirty = true;
        self.ready = false;
    }

    pub(crate) fn try_initialize(&mut self, rig: &Rig) {
        if self.ready {
            return;
        }
        self.ready = true;
        self.skeleton.bone_transforms = rig
            .skeleton
            .bones()
            .iter()
            .map(|bone| (bone.name().to_owned(), bone.transform().to_owned()))
            .collect();
        let mut offset = 0;
        self.deformer.areas = rig
            .deformer
            .areas
            .iter()
            .map(|(name, area)| {
                let area = HaDeformerArea::new(area);
                let count = area.cells_count() * 32;
                let o = offset;
                offset += count;
                (name.to_owned(), (area, o, count))
            })
            .collect();
        self.deformer.buffer = vec![0.0; offset];
        for (area, offset, count) in self.deformer.areas.values_mut() {
            area.apply_data(&mut self.deformer.buffer[*offset..(*offset + *count)]);
        }
    }
}

impl Prefab for HaRigInstance {}
impl PrefabComponent for HaRigInstance {}
