use crate::{
    composite_renderer::{Effect, Renderable},
    math::{Mat2d, Rect, Vec2},
    mesh_animation_asset_protocol::{MeshAnimation, MeshAnimationSequence},
    mesh_asset_protocol::MeshBone,
};
use core::{
    prefab::{Prefab, PrefabComponent},
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "scalar64"))]
use std::f32::consts::PI;
#[cfg(feature = "scalar64")]
use std::f64::consts::PI;
use std::{borrow::Cow, collections::HashMap};
use utils::grid_2d::Grid2d;

#[derive(Ignite, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CompositeVisibility(pub bool);

impl Default for CompositeVisibility {
    fn default() -> Self {
        Self(true)
    }
}

impl Prefab for CompositeVisibility {}
impl PrefabComponent for CompositeVisibility {}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeSurfaceCache {
    name: Cow<'static, str>,
    width: usize,
    height: usize,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) dirty: bool,
}

impl CompositeSurfaceCache {
    pub fn new(name: Cow<'static, str>, width: usize, height: usize) -> Self {
        Self {
            name,
            width,
            height,
            dirty: true,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn set_width(&mut self, width: usize) {
        self.width = width;
        self.dirty = true;
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn set_height(&mut self, height: usize) {
        self.height = height;
        self.dirty = true;
    }

    pub fn rebuild(&mut self) {
        self.dirty = true;
    }

    pub fn is_cached(&self) -> bool {
        !self.dirty
    }
}

impl Prefab for CompositeSurfaceCache {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}
impl PrefabComponent for CompositeSurfaceCache {}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct CompositeRenderable(pub Renderable<'static>);

impl Default for CompositeRenderable {
    fn default() -> Self {
        Self(().into())
    }
}

impl From<Renderable<'static>> for CompositeRenderable {
    fn from(value: Renderable<'static>) -> Self {
        Self(value)
    }
}

impl Prefab for CompositeRenderable {}
impl PrefabComponent for CompositeRenderable {}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct CompositeRenderableStroke(pub Scalar);

impl Default for CompositeRenderableStroke {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Prefab for CompositeRenderableStroke {}
impl PrefabComponent for CompositeRenderableStroke {}

#[derive(Ignite, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeTransform {
    #[serde(default)]
    translation: Vec2,
    #[serde(default)]
    rotation: Scalar,
    #[serde(default = "CompositeTransform::default_scale")]
    scale: Vec2,
    #[serde(skip)]
    #[ignite(ignore)]
    cached: Mat2d,
}

impl Default for CompositeTransform {
    fn default() -> Self {
        Self {
            translation: Vec2::zero(),
            rotation: 0.0,
            scale: Vec2::one(),
            cached: Default::default(),
        }
    }
}

impl CompositeTransform {
    fn default_scale() -> Vec2 {
        1.0.into()
    }

    pub fn new(translation: Vec2, rotation: Scalar, scale: Vec2) -> Self {
        let mut result = Self {
            translation,
            rotation,
            scale,
            cached: Default::default(),
        };
        result.rebuild();
        result
    }

    pub fn translation(v: Vec2) -> Self {
        Self::default().with_translation(v)
    }

    pub fn rotation(v: Scalar) -> Self {
        Self::default().with_rotation(v)
    }

    pub fn scale(v: Vec2) -> Self {
        Self::default().with_scale(v)
    }

    pub fn with_translation(mut self, v: Vec2) -> Self {
        self.translation = v;
        self.rebuild();
        self
    }

    pub fn with_rotation(mut self, v: Scalar) -> Self {
        self.rotation = v;
        self.rebuild();
        self
    }

    pub fn with_scale(mut self, v: Vec2) -> Self {
        self.scale = v;
        self.rebuild();
        self
    }

    pub fn get_translation(&self) -> Vec2 {
        self.translation
    }

    pub fn get_rotation(&self) -> Scalar {
        self.rotation
    }

    pub fn get_direction(&self) -> Vec2 {
        let (y, x) = self.rotation.sin_cos();
        Vec2::new(x, y)
    }

    pub fn get_scale(&self) -> Vec2 {
        self.scale
    }

    pub fn set_translation(&mut self, v: Vec2) {
        self.translation = v;
        self.rebuild();
    }

    pub fn set_rotation(&mut self, v: Scalar) {
        self.rotation = v;
        self.rebuild();
    }

    pub fn set_scale(&mut self, v: Vec2) {
        self.scale = v;
        self.rebuild();
    }

    pub fn apply(&mut self, translation: Vec2, rotation: Scalar, scale: Vec2) {
        self.translation = translation;
        self.rotation = rotation;
        self.scale = scale;
        self.rebuild();
    }

    pub fn matrix(&self) -> Mat2d {
        self.cached
    }

    fn rebuild(&mut self) {
        let t = Mat2d::translation(self.translation);
        let r = Mat2d::rotation(self.rotation);
        let s = Mat2d::scale(self.scale);
        self.cached = t * r * s;
    }
}

impl Prefab for CompositeTransform {
    fn post_from_prefab(&mut self) {
        self.rebuild();
    }
}
impl PrefabComponent for CompositeTransform {}

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CompositeRenderLayer(pub usize);

impl Prefab for CompositeRenderLayer {}
impl PrefabComponent for CompositeRenderLayer {}

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CompositeRenderDepth(pub Scalar);

impl Prefab for CompositeRenderDepth {}
impl PrefabComponent for CompositeRenderDepth {}

#[derive(Ignite, Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CompositeRenderAlpha(pub Scalar);

impl Default for CompositeRenderAlpha {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Prefab for CompositeRenderAlpha {}
impl PrefabComponent for CompositeRenderAlpha {}

#[derive(Ignite, Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeCameraAlignment(pub Vec2);

impl Prefab for CompositeCameraAlignment {}
impl PrefabComponent for CompositeCameraAlignment {}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeEffect(pub Effect);

impl Prefab for CompositeEffect {}
impl PrefabComponent for CompositeEffect {}

#[derive(Ignite, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompositeScalingMode {
    None,
    Center,
    Aspect,
    CenterAspect,
}

impl Default for CompositeScalingMode {
    fn default() -> Self {
        CompositeScalingMode::None
    }
}

#[derive(Ignite, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CompositeScalingTarget {
    Width,
    Height,
    Both,
    BothMinimum,
    Cover(Scalar, Scalar),
}

impl Default for CompositeScalingTarget {
    fn default() -> Self {
        CompositeScalingTarget::Both
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeCamera {
    #[serde(default)]
    pub scaling: CompositeScalingMode,
    #[serde(default)]
    pub scaling_target: CompositeScalingTarget,
    #[serde(default)]
    pub tags: Vec<Cow<'static, str>>,
}

impl CompositeCamera {
    pub fn new(scaling: CompositeScalingMode) -> Self {
        Self {
            scaling,
            scaling_target: CompositeScalingTarget::default(),
            tags: vec![],
        }
    }

    pub fn with_scaling_target(
        scaling: CompositeScalingMode,
        target: CompositeScalingTarget,
    ) -> Self {
        Self {
            scaling,
            scaling_target: target,
            tags: vec![],
        }
    }

    pub fn tag(mut self, tag: Cow<'static, str>) -> Self {
        self.tags.push(tag);
        self
    }

    pub fn tags(mut self, tags: Vec<Cow<'static, str>>) -> Self {
        self.tags = tags;
        self
    }

    pub fn view_matrix(&self, transform: &CompositeTransform, screen_size: Vec2) -> Mat2d {
        let wh = screen_size.x * 0.5;
        let hh = screen_size.y * 0.5;
        let scale = match self.scaling_target {
            CompositeScalingTarget::Width => screen_size.x,
            CompositeScalingTarget::Height => screen_size.y,
            CompositeScalingTarget::Both => screen_size.x.min(screen_size.y),
            CompositeScalingTarget::BothMinimum => screen_size.x.max(screen_size.y),
            CompositeScalingTarget::Cover(width, height) => {
                (screen_size.x / width).max(screen_size.y / height)
            }
        };
        let s = Mat2d::scale(Vec2::one() / transform.get_scale());
        let ss = Mat2d::scale(Vec2::new(scale, scale) / transform.get_scale());
        let r = Mat2d::rotation(-transform.get_rotation());
        let t = Mat2d::translation(-transform.get_translation());
        let tt = Mat2d::translation([wh, hh].into());
        match self.scaling {
            CompositeScalingMode::None => s * r * t,
            CompositeScalingMode::Center => tt * s * r * t,
            CompositeScalingMode::Aspect => ss * r * t,
            CompositeScalingMode::CenterAspect => tt * ss * r * t,
        }
    }

    pub fn view_box(&self, transform: &CompositeTransform, screen_size: Vec2) -> Option<Rect> {
        if let Some(inv_mat) = !self.view_matrix(transform, screen_size) {
            let points = &[
                Vec2::zero() * inv_mat,
                Vec2::new(screen_size.x, 0.0) * inv_mat,
                screen_size * inv_mat,
                Vec2::new(0.0, screen_size.y) * inv_mat,
            ];
            Rect::bounding(points)
        } else {
            None
        }
    }
}

impl Prefab for CompositeCamera {}
impl PrefabComponent for CompositeCamera {}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeSprite {
    #[serde(default)]
    pub alignment: Vec2,
    sheet_frame: Option<(Cow<'static, str>, Cow<'static, str>)>,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) dirty: bool,
}

impl CompositeSprite {
    pub fn new(sheet: Cow<'static, str>, frame: Cow<'static, str>) -> Self {
        Self {
            alignment: 0.0.into(),
            sheet_frame: Some((sheet, frame)),
            dirty: true,
        }
    }

    pub fn align(mut self, value: Vec2) -> Self {
        self.alignment = value;
        self
    }

    pub fn sheet_frame(&self) -> Option<(&str, &str)> {
        if let Some((sheet, frame)) = &self.sheet_frame {
            Some((sheet, frame))
        } else {
            None
        }
    }

    pub fn set_sheet_frame(&mut self, sheet_frame: Option<(Cow<'static, str>, Cow<'static, str>)>) {
        self.sheet_frame = sheet_frame;
        self.dirty = true;
    }

    pub fn sheet(&self) -> Option<&str> {
        if let Some((sheet, _)) = &self.sheet_frame {
            Some(sheet)
        } else {
            None
        }
    }

    pub fn set_sheet(&mut self, sheet: Option<Cow<'static, str>>) {
        if let Some(sheet) = sheet {
            if let Some(sheet_frame) = &mut self.sheet_frame {
                sheet_frame.0 = sheet;
            } else {
                self.sheet_frame = Some((sheet, "".into()));
            }
        } else {
            self.sheet_frame = None;
        }
    }

    pub fn frame(&self) -> Option<&str> {
        if let Some((_, frame)) = &self.sheet_frame {
            Some(frame)
        } else {
            None
        }
    }

    pub fn set_frame(&mut self, frame: Option<Cow<'static, str>>) {
        if let Some(frame) = frame {
            if let Some(sheet_frame) = &mut self.sheet_frame {
                sheet_frame.1 = frame;
            } else {
                self.sheet_frame = Some(("".into(), frame));
            }
        } else {
            self.sheet_frame = None;
        }
    }

    pub fn apply(&mut self) {
        self.dirty = true;
    }
}

impl Prefab for CompositeSprite {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}
impl PrefabComponent for CompositeSprite {}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct SpriteAnimation {
    pub sheet: Cow<'static, str>,
    pub frames: Vec<Cow<'static, str>>,
}

impl SpriteAnimation {
    pub fn new(sheet: Cow<'static, str>, frames: Vec<Cow<'static, str>>) -> Self {
        Self { sheet, frames }
    }
}

impl From<(Cow<'static, str>, Vec<Cow<'static, str>>)> for SpriteAnimation {
    fn from((sheet, frames): (Cow<'static, str>, Vec<Cow<'static, str>>)) -> Self {
        Self::new(sheet, frames)
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeSpriteAnimation {
    pub animations: HashMap<Cow<'static, str>, SpriteAnimation>,
    // (name, phase, speed, looped)?
    #[serde(default)]
    pub(crate) current: Option<(Cow<'static, str>, Scalar, Scalar, bool)>,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) dirty: bool,
}

impl CompositeSpriteAnimation {
    pub fn new(animations: HashMap<Cow<'static, str>, SpriteAnimation>) -> Self {
        Self {
            animations,
            current: None,
            dirty: false,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_iter<I>(animations: I) -> Self
    where
        I: IntoIterator<Item = (Cow<'static, str>, SpriteAnimation)>,
    {
        Self {
            animations: animations.into_iter().collect::<HashMap<_, _>>(),
            current: None,
            dirty: false,
        }
    }

    pub fn autoplay(mut self, name: &str, speed: Scalar, looped: bool) -> Self {
        self.play(name, speed, looped);
        self
    }

    pub fn play(&mut self, name: &str, speed: Scalar, looped: bool) -> bool {
        if self.animations.contains_key(name) {
            self.current = Some((name.to_owned().into(), 0.0, speed, looped));
            self.dirty = true;
            true
        } else {
            self.current = None;
            false
        }
    }

    pub fn stop(&mut self) {
        self.current = None;
    }

    pub fn is_playing(&self) -> bool {
        self.current.is_some()
    }

    pub fn current(&self) -> Option<&str> {
        if let Some((name, _, _, _)) = &self.current {
            Some(name)
        } else {
            None
        }
    }

    pub fn phase(&self) -> Option<Scalar> {
        if let Some((_, phase, _, _)) = &self.current {
            Some(*phase)
        } else {
            None
        }
    }

    pub fn set_phase(&mut self, value: Scalar) -> bool {
        if let Some((_, phase, _, _)) = &mut self.current {
            *phase = value.max(0.0);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn speed(&self) -> Option<Scalar> {
        if let Some((_, _, speed, _)) = &self.current {
            Some(*speed)
        } else {
            None
        }
    }

    pub fn set_speed(&mut self, value: Scalar) -> bool {
        if let Some((_, _, speed, _)) = &mut self.current {
            *speed = value;
            true
        } else {
            false
        }
    }

    pub fn looped(&self) -> Option<bool> {
        if let Some((_, _, _, looped)) = &self.current {
            Some(*looped)
        } else {
            None
        }
    }

    pub fn set_looped(&mut self, value: bool) -> bool {
        if let Some((_, _, _, looped)) = &mut self.current {
            *looped = value;
            true
        } else {
            false
        }
    }

    pub fn apply(&mut self) {
        self.dirty = true;
    }

    pub(crate) fn process(&mut self, delta_time: Scalar) {
        if let Some((name, phase, speed, looped)) = &mut self.current {
            if let Some(animation) = self.animations.get(name) {
                let prev = phase.max(0.0) as usize;
                *phase += *speed * delta_time;
                let next = phase.max(0.0) as usize;
                if next >= animation.frames.len() {
                    if *looped {
                        *phase = 0.0;
                        self.dirty = true;
                    } else {
                        self.current = None;
                    }
                } else if prev != next {
                    self.dirty = true;
                }
            }
        }
    }
}

impl Prefab for CompositeSpriteAnimation {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}
impl PrefabComponent for CompositeSpriteAnimation {}

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileRotate {
    Degrees0,
    Degrees90,
    Degrees180,
    Degrees270,
}

impl Default for TileRotate {
    fn default() -> Self {
        TileRotate::Degrees0
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct TileCell {
    pub col: usize,
    pub row: usize,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default)]
    pub rotate: TileRotate,
    #[serde(default)]
    pub visible: bool,
}

impl TileCell {
    pub fn new(col: usize, row: usize) -> Self {
        Self {
            col,
            row,
            flip_x: false,
            flip_y: false,
            rotate: Default::default(),
            visible: true,
        }
    }

    pub fn flip(mut self, x: bool, y: bool) -> Self {
        self.flip_x = x;
        self.flip_y = y;
        self
    }

    pub fn rotate(mut self, value: TileRotate) -> Self {
        self.rotate = value;
        self
    }

    pub fn visible(mut self, value: bool) -> Self {
        self.visible = value;
        self
    }

    #[allow(clippy::many_single_char_names)]
    pub fn matrix(&self, col: usize, row: usize, width: Scalar, height: Scalar) -> Mat2d {
        let hw = width * 0.5;
        let hh = height * 0.5;
        let a = Mat2d::translation([-hw, -hh].into());
        let sx = if self.flip_x { -1.0 } else { 1.0 };
        let sy = if self.flip_y { -1.0 } else { 1.0 };
        let b = Mat2d::scale([sx, sy].into());
        let c = match self.rotate {
            TileRotate::Degrees0 => Mat2d::default(),
            TileRotate::Degrees90 => Mat2d::rotation(PI * 0.5),
            TileRotate::Degrees180 => Mat2d::rotation(PI),
            TileRotate::Degrees270 => Mat2d::rotation(PI * 1.5),
        };
        let d = Mat2d::translation([hw, hh].into());
        let e = Mat2d::translation([col as Scalar * width, row as Scalar * height].into());
        e * d * c * b * a
    }
}

impl From<(usize, usize)> for TileCell {
    fn from((col, row): (usize, usize)) -> Self {
        Self::new(col, row)
    }
}

impl From<(usize, usize, bool)> for TileCell {
    fn from((col, row, visible): (usize, usize, bool)) -> Self {
        Self::new(col, row).visible(visible)
    }
}

impl From<(usize, usize, bool, bool)> for TileCell {
    fn from((col, row, flip_x, flip_y): (usize, usize, bool, bool)) -> Self {
        Self::new(col, row).flip(flip_x, flip_y)
    }
}

impl From<(usize, usize, bool, bool, bool)> for TileCell {
    fn from((col, row, flip_x, flip_y, visible): (usize, usize, bool, bool, bool)) -> Self {
        Self::new(col, row).flip(flip_x, flip_y).visible(visible)
    }
}

impl From<(usize, usize, TileRotate)> for TileCell {
    fn from((col, row, rotate): (usize, usize, TileRotate)) -> Self {
        Self::new(col, row).rotate(rotate)
    }
}

impl From<(usize, usize, TileRotate, bool)> for TileCell {
    fn from((col, row, rotate, visible): (usize, usize, TileRotate, bool)) -> Self {
        Self::new(col, row).rotate(rotate).visible(visible)
    }
}

impl From<(usize, usize, bool, bool, TileRotate)> for TileCell {
    fn from((col, row, flip_x, flip_y, rotate): (usize, usize, bool, bool, TileRotate)) -> Self {
        Self::new(col, row).flip(flip_x, flip_y).rotate(rotate)
    }
}

impl From<(usize, usize, bool, bool, TileRotate, bool)> for TileCell {
    fn from(
        (col, row, flip_x, flip_y, rotate, visible): (usize, usize, bool, bool, TileRotate, bool),
    ) -> Self {
        Self::new(col, row)
            .flip(flip_x, flip_y)
            .rotate(rotate)
            .visible(visible)
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeTilemap {
    tileset: Option<Cow<'static, str>>,
    grid: Grid2d<TileCell>,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) dirty: bool,
}

impl CompositeTilemap {
    pub fn new(tileset: Cow<'static, str>, grid: Grid2d<TileCell>) -> Self {
        Self {
            tileset: Some(tileset),
            grid,
            dirty: true,
        }
    }

    pub fn tileset(&self) -> Option<&str> {
        if let Some(tileset) = &self.tileset {
            Some(tileset)
        } else {
            None
        }
    }

    pub fn set_tileset(&mut self, tileset: Option<Cow<'static, str>>) {
        self.tileset = tileset;
        self.dirty = true;
    }

    pub fn grid(&self) -> &Grid2d<TileCell> {
        &self.grid
    }

    pub fn grid_mut(&mut self) -> &mut Grid2d<TileCell> {
        self.dirty = true;
        &mut self.grid
    }

    pub fn set_grid(&mut self, grid: Grid2d<TileCell>) {
        self.grid = grid;
        self.dirty = true;
    }

    pub fn apply(&mut self) {
        self.dirty = true;
    }
}

impl Prefab for CompositeTilemap {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}
impl PrefabComponent for CompositeTilemap {}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct TilemapAnimation {
    pub tileset: Cow<'static, str>,
    pub frames: Vec<Grid2d<TileCell>>,
}

impl TilemapAnimation {
    pub fn new(tileset: Cow<'static, str>, frames: Vec<Grid2d<TileCell>>) -> Self {
        Self { tileset, frames }
    }
}

impl From<(Cow<'static, str>, Vec<Grid2d<TileCell>>)> for TilemapAnimation {
    fn from((tileset, frames): (Cow<'static, str>, Vec<Grid2d<TileCell>>)) -> Self {
        Self::new(tileset, frames)
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeTilemapAnimation {
    pub animations: HashMap<Cow<'static, str>, TilemapAnimation>,
    // (name, phase, speed, looped)
    #[serde(default)]
    pub(crate) current: Option<(Cow<'static, str>, Scalar, Scalar, bool)>,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) dirty: bool,
}

impl CompositeTilemapAnimation {
    pub fn new(animations: HashMap<Cow<'static, str>, TilemapAnimation>) -> Self {
        Self {
            animations,
            current: None,
            dirty: false,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_iter<I>(animations: I) -> Self
    where
        I: IntoIterator<Item = (Cow<'static, str>, TilemapAnimation)>,
    {
        Self {
            animations: animations.into_iter().collect::<HashMap<_, _>>(),
            current: None,
            dirty: false,
        }
    }

    pub fn play(&mut self, name: &str, speed: Scalar, looped: bool) -> bool {
        if self.animations.contains_key(name) {
            self.current = Some((name.to_owned().into(), 0.0, speed, looped));
            self.dirty = true;
            true
        } else {
            self.current = None;
            false
        }
    }

    pub fn stop(&mut self) {
        self.current = None;
    }

    pub fn is_playing(&self) -> bool {
        self.current.is_some()
    }

    pub fn current(&self) -> Option<&str> {
        if let Some((name, _, _, _)) = &self.current {
            Some(name)
        } else {
            None
        }
    }

    pub fn phase(&self) -> Option<Scalar> {
        if let Some((_, phase, _, _)) = &self.current {
            Some(*phase)
        } else {
            None
        }
    }

    pub fn set_phase(&mut self, value: Scalar) -> bool {
        if let Some((_, phase, _, _)) = &mut self.current {
            *phase = value.max(0.0);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn speed(&self) -> Option<Scalar> {
        if let Some((_, _, speed, _)) = &self.current {
            Some(*speed)
        } else {
            None
        }
    }

    pub fn set_speed(&mut self, value: Scalar) -> bool {
        if let Some((_, _, speed, _)) = &mut self.current {
            *speed = value;
            true
        } else {
            false
        }
    }

    pub fn looped(&self) -> Option<bool> {
        if let Some((_, _, _, looped)) = &self.current {
            Some(*looped)
        } else {
            None
        }
    }

    pub fn set_looped(&mut self, value: bool) -> bool {
        if let Some((_, _, _, looped)) = &mut self.current {
            *looped = value;
            true
        } else {
            false
        }
    }

    pub fn apply(&mut self) {
        self.dirty = true;
    }

    pub(crate) fn process(&mut self, delta_time: Scalar) {
        if let Some((name, phase, speed, looped)) = &mut self.current {
            if let Some(animation) = self.animations.get(name) {
                let prev = phase.max(0.0) as usize;
                *phase += *speed * delta_time;
                let next = phase.max(0.0) as usize;
                if next >= animation.frames.len() {
                    if *looped {
                        *phase = 0.0;
                        self.dirty = true;
                    } else {
                        self.current = None;
                    }
                } else if prev != next {
                    self.dirty = true;
                }
            }
        }
    }
}

impl Prefab for CompositeTilemapAnimation {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}
impl PrefabComponent for CompositeTilemapAnimation {}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeMapChunk {
    map_name: Cow<'static, str>,
    layer_name: Cow<'static, str>,
    #[serde(default)]
    offset: (usize, usize),
    #[serde(default)]
    size: Option<(usize, usize)>,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) dirty: bool,
}

impl CompositeMapChunk {
    pub fn new(map_name: Cow<'static, str>, layer_name: Cow<'static, str>) -> Self {
        Self {
            map_name,
            layer_name,
            offset: (0, 0),
            size: None,
            dirty: true,
        }
    }

    pub fn map_name(&self) -> &str {
        &self.map_name
    }

    pub fn set_map_name(&mut self, map_name: Cow<'static, str>) {
        self.map_name = map_name;
        self.dirty = true;
    }

    pub fn layer_name(&self) -> &str {
        &self.layer_name
    }

    pub fn set_layer_name(&mut self, layer_name: Cow<'static, str>) {
        self.layer_name = layer_name;
        self.dirty = true;
    }

    pub fn offset(&self) -> (usize, usize) {
        self.offset
    }

    pub fn set_offset(&mut self, offset: (usize, usize)) {
        self.offset = offset;
        self.dirty = true;
    }

    pub fn size(&self) -> Option<(usize, usize)> {
        self.size
    }

    pub fn set_size(&mut self, size: Option<(usize, usize)>) {
        self.size = size;
        self.dirty = true;
    }

    pub fn apply(&mut self) {
        self.dirty = true;
    }

    pub fn is_cached(&self) -> bool {
        !self.dirty
    }
}

impl Prefab for CompositeMapChunk {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}
impl PrefabComponent for CompositeMapChunk {}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct MeshMaterial {
    pub image: Cow<'static, str>,
    #[serde(default = "MeshMaterial::default_alpha")]
    pub alpha: Scalar,
    #[serde(default)]
    pub order: Scalar,
}

impl MeshMaterial {
    fn default_alpha() -> Scalar {
        1.0
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeMesh {
    mesh: Cow<'static, str>,
    materials: Vec<MeshMaterial>,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) bones_local_transform: HashMap<String, CompositeTransform>,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) bones_model_space: HashMap<String, Mat2d>,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) dirty_mesh: bool,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) dirty_visuals: bool,
}

impl CompositeMesh {
    pub fn new(mesh: Cow<'static, str>, materials: Vec<MeshMaterial>) -> Self {
        Self {
            mesh,
            materials,
            bones_local_transform: Default::default(),
            bones_model_space: Default::default(),
            dirty_mesh: true,
            dirty_visuals: true,
        }
    }

    pub fn mesh(&self) -> &str {
        &self.mesh
    }

    pub fn set_mesh(&mut self, mesh: Cow<'static, str>) {
        self.mesh = mesh;
        self.dirty_mesh = true;
        self.dirty_visuals = true;
    }

    pub fn materials(&self) -> &[MeshMaterial] {
        &self.materials
    }

    pub fn materials_mut(&mut self) -> &mut [MeshMaterial] {
        &mut self.materials
    }

    pub fn set_materials(&mut self, materials: Vec<MeshMaterial>) {
        self.materials = materials;
        self.dirty_visuals = true;
    }

    pub fn with_materials<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Vec<MeshMaterial>),
    {
        f(&mut self.materials);
        self.dirty_visuals = true;
    }

    pub fn bone_local_transform(&self, name: &str) -> Option<&CompositeTransform> {
        self.bones_local_transform.get(name)
    }

    pub fn bone_local_transform_mut(&mut self, name: &str) -> Option<&mut CompositeTransform> {
        self.bones_local_transform.get_mut(name)
    }

    pub fn set_bone_local_transform(&mut self, name: &str, transform: CompositeTransform) {
        if let Some(t) = self.bones_local_transform.get_mut(name) {
            *t = transform;
            self.dirty_visuals = true;
        }
    }

    pub fn with_bone_local_transform<F>(&mut self, name: &str, mut f: F)
    where
        F: FnMut(&mut CompositeTransform),
    {
        if let Some(t) = self.bones_local_transform.get_mut(name) {
            f(t);
            self.dirty_visuals = true;
        }
    }

    pub fn apply(&mut self) {
        self.dirty_visuals = true;
    }

    pub(crate) fn rebuild_model_space(&mut self, root: &MeshBone) {
        self.rebuild_bone_model_space(root, "", Mat2d::default())
    }

    fn rebuild_bone_model_space(&mut self, bone: &MeshBone, name: &str, parent: Mat2d) {
        if let Some(t) = self.bones_local_transform.get(name) {
            let m = parent * t.matrix();
            self.bones_model_space.insert(name.to_owned(), m);
            for (name, bone) in &bone.children {
                self.rebuild_bone_model_space(bone, name, m);
            }
        }
    }

    pub(crate) fn setup_bones_from_rig(&mut self, root: &MeshBone) {
        let count = root.bones_count();
        let mut bones_local_transform = HashMap::with_capacity(count);
        let mut bones_model_space = HashMap::with_capacity(count);
        Self::register_bone(
            root,
            "",
            Mat2d::default(),
            &mut bones_local_transform,
            &mut bones_model_space,
        );
        self.bones_local_transform = bones_local_transform;
        self.bones_model_space = bones_model_space;
    }

    fn register_bone(
        bone: &MeshBone,
        name: &str,
        parent: Mat2d,
        bones_local_transform: &mut HashMap<String, CompositeTransform>,
        bones_model_space: &mut HashMap<String, Mat2d>,
    ) {
        bones_local_transform.insert(name.to_owned(), bone.transform.clone());
        let m = parent * bone.transform.matrix();
        bones_model_space.insert(name.to_owned(), m);
        for (name, bone) in &bone.children {
            Self::register_bone(bone, name, m, bones_local_transform, bones_model_space);
        }
    }
}

impl Prefab for CompositeMesh {
    fn post_from_prefab(&mut self) {
        self.dirty_mesh = true;
        self.dirty_visuals = true;
    }
}

impl PrefabComponent for CompositeMesh {}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeMeshAnimation {
    animation: Cow<'static, str>,
    /// {name: (phase, speed, looped, blend weight)}
    #[serde(default)]
    pub(crate) current: HashMap<String, (Scalar, Scalar, bool, Scalar)>,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) dirty: bool,
}

impl CompositeMeshAnimation {
    pub fn animation(&self) -> &str {
        &self.animation
    }

    pub fn autoplay(
        mut self,
        name: &str,
        speed: Scalar,
        looped: bool,
        blend_weight: Scalar,
    ) -> Self {
        self.play(name, speed, looped, blend_weight);
        self
    }

    pub fn play(&mut self, name: &str, speed: Scalar, looped: bool, blend_weight: Scalar) {
        self.current
            .insert(name.to_owned(), (0.0, speed, looped, blend_weight));
        self.dirty = true;
    }

    pub fn stop(&mut self, name: &str) {
        self.current.remove(name);
    }

    pub fn is_playing_any(&self) -> bool {
        !self.current.is_empty()
    }

    pub fn is_playing(&self, name: &str) -> bool {
        self.current.contains_key(name)
    }

    pub fn current(&self) -> impl Iterator<Item = &String> {
        self.current.keys()
    }

    pub fn phase(&self, name: &str) -> Option<Scalar> {
        self.current.get(name).map(|(phase, _, _, _)| *phase)
    }

    pub fn set_phase(&mut self, name: &str, value: Scalar) -> bool {
        if let Some((phase, _, _, _)) = self.current.get_mut(name) {
            *phase = value.max(0.0);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn speed(&self, name: &str) -> Option<Scalar> {
        self.current.get(name).map(|(_, speed, _, _)| *speed)
    }

    pub fn set_speed(&mut self, name: &str, value: Scalar) -> bool {
        if let Some((_, speed, _, _)) = self.current.get_mut(name) {
            *speed = value;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn looped(&self, name: &str) -> Option<bool> {
        self.current.get(name).map(|(_, _, looped, _)| *looped)
    }

    pub fn set_looped(&mut self, name: &str, value: bool) -> bool {
        if let Some((_, _, looped, _)) = self.current.get_mut(name) {
            *looped = value;
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn blend_weight(&self, name: &str) -> Option<Scalar> {
        self.current
            .get(name)
            .map(|(_, _, _, blend_weight)| *blend_weight)
    }

    pub fn set_blend_weight(&mut self, name: &str, value: Scalar) -> bool {
        if let Some((_, _, _, blend_weight)) = self.current.get_mut(name) {
            *blend_weight = value.max(0.0);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn apply(&mut self) {
        self.dirty = true;
    }

    pub(crate) fn process(
        &mut self,
        delta_time: Scalar,
        animation: &MeshAnimation,
        mesh: &mut CompositeMesh,
    ) {
        if !self.dirty || self.current.is_empty() {
            return;
        }
        self.dirty = false;
        for (name, (phase, speed, looped, _)) in &mut self.current {
            if let Some(sequence) = animation.sequences.get(name) {
                let old = *phase;
                *phase = (*phase + *speed * delta_time)
                    .max(0.0)
                    .min(sequence.length());
                if (*phase - old).abs() > 0.0 {
                    self.dirty = true;
                } else if *looped {
                    if *phase > 0.0 {
                        *phase = 0.0;
                    } else {
                        *phase = sequence.length();
                    }
                    self.dirty = true;
                }
            }
        }
        if !self.dirty {
            return;
        }
        let meta = self
            .current
            .iter()
            .filter_map(|(n, (p, _, _, bw))| animation.sequences.get(n).map(|s| (s, *p, *bw)))
            .collect::<Vec<_>>();
        if meta.is_empty() {
            return;
        }
        let total_blend_weight = meta.iter().fold(0.0, |a, v| a + v.2);
        for (index, material) in mesh.materials.iter_mut().enumerate() {
            Self::apply_mesh_material(material, index, &meta, total_blend_weight);
        }
        for (name, transform) in mesh.bones_local_transform.iter_mut() {
            Self::apply_mesh_bone_transform(transform, name, &meta, total_blend_weight);
        }
        mesh.apply();
    }

    fn apply_mesh_material(
        material: &mut MeshMaterial,
        index: usize,
        // [(sequence, phase, blend weight)]
        meta: &[(&MeshAnimationSequence, Scalar, Scalar)],
        total_blend_weight: Scalar,
    ) {
        material.alpha = meta.iter().fold(0.0, |a, v| {
            a + v.0.sample_submesh_alpha(v.1, index, material.alpha) * v.2
        }) / total_blend_weight;
        material.order = meta.iter().fold(0.0, |a, v| {
            a + v.0.sample_submesh_order(v.1, index, material.order) * v.2
        }) / total_blend_weight;
    }

    fn apply_mesh_bone_transform(
        transform: &mut CompositeTransform,
        name: &str,
        // [(sequence, phase, blend weight)]
        meta: &[(&MeshAnimationSequence, Scalar, Scalar)],
        total_blend_weight: Scalar,
    ) {
        let position = transform.get_translation();
        let rotation = transform.get_rotation();
        let scale = transform.get_scale();
        let px = meta.iter().fold(0.0, |a, v| {
            a + v.0.sample_bone_position_x(v.1, name, position.x) * v.2
        }) / total_blend_weight;
        let py = meta.iter().fold(0.0, |a, v| {
            a + v.0.sample_bone_position_y(v.1, name, position.y) * v.2
        }) / total_blend_weight;
        let rt = meta.iter().fold(0.0, |a, v| {
            a + v.0.sample_bone_rotation(v.1, name, rotation) * v.2
        }) / total_blend_weight;
        let sx = meta.iter().fold(0.0, |a, v| {
            a + v.0.sample_bone_scale_x(v.1, name, scale.x) * v.2
        }) / total_blend_weight;
        let sy = meta.iter().fold(0.0, |a, v| {
            a + v.0.sample_bone_scale_y(v.1, name, scale.x) * v.2
        }) / total_blend_weight;
        *transform = CompositeTransform::new((px, py).into(), rt, (sx, sy).into());
    }
}

impl Prefab for CompositeMeshAnimation {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}

impl PrefabComponent for CompositeMeshAnimation {}
