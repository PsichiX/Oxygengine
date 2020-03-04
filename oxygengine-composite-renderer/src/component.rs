use crate::{
    composite_renderer::{Command, Effect, Image, Renderable, Text},
    math::{Mat2d, Rect, Vec2},
    resource::{CompositeUiInteractibles, CompositeUiThemes, UiThemed},
};
use core::{
    ecs::{Component, DenseVecStorage, FlaggedStorage, HashMapStorage, VecStorage},
    prefab::{Prefab, PrefabComponent},
    Scalar,
};
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "scalar64"))]
use std::f32::consts::PI;
#[cfg(feature = "scalar64")]
use std::f64::consts::PI;
use std::{borrow::Cow, collections::HashMap};
use utils::grid_2d::Grid2d;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CompositeVisibility(pub bool);

impl Component for CompositeVisibility {
    type Storage = VecStorage<Self>;
}

impl Default for CompositeVisibility {
    fn default() -> Self {
        Self(true)
    }
}

impl Prefab for CompositeVisibility {}
impl PrefabComponent for CompositeVisibility {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeSurfaceCache {
    name: Cow<'static, str>,
    width: usize,
    height: usize,
    #[serde(skip)]
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

impl Component for CompositeSurfaceCache {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

impl Prefab for CompositeSurfaceCache {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}
impl PrefabComponent for CompositeSurfaceCache {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeRenderable(pub Renderable<'static>);

impl Component for CompositeRenderable {
    type Storage = DenseVecStorage<Self>;
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeRenderableStroke(pub Scalar);

impl Default for CompositeRenderableStroke {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Component for CompositeRenderableStroke {
    type Storage = VecStorage<Self>;
}

impl Prefab for CompositeRenderableStroke {}
impl PrefabComponent for CompositeRenderableStroke {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeTransform {
    #[serde(default)]
    translation: Vec2,
    #[serde(default)]
    rotation: Scalar,
    #[serde(default = "CompositeTransform::default_scale")]
    scale: Vec2,
    #[serde(skip)]
    cached: Mat2d,
}

impl Component for CompositeTransform {
    type Storage = DenseVecStorage<Self>;
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

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CompositeRenderLayer(pub usize);

impl Component for CompositeRenderLayer {
    type Storage = VecStorage<Self>;
}

impl Prefab for CompositeRenderLayer {}
impl PrefabComponent for CompositeRenderLayer {}

#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CompositeRenderDepth(pub Scalar);

impl Component for CompositeRenderDepth {
    type Storage = VecStorage<Self>;
}

impl Prefab for CompositeRenderDepth {}
impl PrefabComponent for CompositeRenderDepth {}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CompositeRenderAlpha(pub Scalar);

impl Component for CompositeRenderAlpha {
    type Storage = VecStorage<Self>;
}

impl Default for CompositeRenderAlpha {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Prefab for CompositeRenderAlpha {}
impl PrefabComponent for CompositeRenderAlpha {}

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositeCameraAlignment(pub Vec2);

impl Component for CompositeCameraAlignment {
    type Storage = VecStorage<Self>;
}

impl Prefab for CompositeCameraAlignment {}
impl PrefabComponent for CompositeCameraAlignment {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeEffect(pub Effect);

impl Component for CompositeEffect {
    type Storage = VecStorage<Self>;
}

impl Prefab for CompositeEffect {}
impl PrefabComponent for CompositeEffect {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeCamera {
    #[serde(default)]
    pub scaling: CompositeScalingMode,
    #[serde(default)]
    pub scaling_target: CompositeScalingTarget,
    #[serde(default)]
    pub tags: Vec<Cow<'static, str>>,
}

impl Component for CompositeCamera {
    type Storage = HashMapStorage<Self>;
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeSprite {
    #[serde(default)]
    pub alignment: Vec2,
    sheet_frame: Option<(Cow<'static, str>, Cow<'static, str>)>,
    #[serde(skip)]
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

impl Component for CompositeSprite {
    type Storage = VecStorage<Self>;
}

impl Prefab for CompositeSprite {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}
impl PrefabComponent for CompositeSprite {}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeSpriteAnimation {
    pub animations: HashMap<Cow<'static, str>, SpriteAnimation>,
    // (name, phase, speed, looped)
    #[serde(default)]
    pub(crate) current: Option<(Cow<'static, str>, Scalar, Scalar, bool)>,
    #[serde(skip)]
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

impl Component for CompositeSpriteAnimation {
    type Storage = VecStorage<Self>;
}

impl Prefab for CompositeSpriteAnimation {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}
impl PrefabComponent for CompositeSpriteAnimation {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeTilemap {
    tileset: Option<Cow<'static, str>>,
    grid: Grid2d<TileCell>,
    #[serde(skip)]
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

impl Component for CompositeTilemap {
    type Storage = VecStorage<Self>;
}

impl Prefab for CompositeTilemap {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}
impl PrefabComponent for CompositeTilemap {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeTilemapAnimation {
    pub animations: HashMap<Cow<'static, str>, TilemapAnimation>,
    // (name, phase, speed, looped)
    #[serde(default)]
    pub(crate) current: Option<(Cow<'static, str>, Scalar, Scalar, bool)>,
    #[serde(skip)]
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

impl Component for CompositeTilemapAnimation {
    type Storage = VecStorage<Self>;
}

impl Prefab for CompositeTilemapAnimation {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}
impl PrefabComponent for CompositeTilemapAnimation {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CompositeMapChunk {
    map_name: Cow<'static, str>,
    layer_name: Cow<'static, str>,
    #[serde(default)]
    offset: (usize, usize),
    #[serde(default)]
    size: Option<(usize, usize)>,
    #[serde(skip)]
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

impl Component for CompositeMapChunk {
    type Storage = VecStorage<Self>;
}

impl Prefab for CompositeMapChunk {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}
impl PrefabComponent for CompositeMapChunk {}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct UiMargin {
    #[serde(default)]
    pub left: Scalar,
    #[serde(default)]
    pub right: Scalar,
    #[serde(default)]
    pub top: Scalar,
    #[serde(default)]
    pub bottom: Scalar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiImagePath {
    Single(Cow<'static, str>),
    Set(Box<[Option<Cow<'static, str>>; 9]>),
}

impl UiImagePath {
    pub fn get(&self, index: usize) -> Option<&str> {
        match self {
            Self::Single(p) => Some(p),
            Self::Set(p) => {
                if let Some(p) = &p[index] {
                    Some(p)
                } else {
                    None
                }
            }
        }
    }
}

impl Default for UiImagePath {
    fn default() -> Self {
        Self::Single(Default::default())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiImage {
    #[serde(default)]
    pub image_path: UiImagePath,
    #[serde(default)]
    pub source_rect: Rect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiElementType {
    None,
    Image(Box<UiImage>),
    Text(Text<'static>),
}

impl Default for UiElementType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeUiElement {
    #[serde(default)]
    pub id: Option<Cow<'static, str>>,
    #[serde(default)]
    pub theme: Option<Cow<'static, str>>,
    #[serde(default)]
    pub camera_name: Cow<'static, str>,
    #[serde(default)]
    pub interactive: Option<Cow<'static, str>>,
    #[serde(default)]
    pub element_type: UiElementType,
    #[serde(default)]
    pub margin: UiMargin,
    #[serde(default)]
    pub padding: UiMargin,
    #[serde(default)]
    pub left_anchor: Scalar,
    #[serde(default)]
    pub right_anchor: Scalar,
    #[serde(default)]
    pub top_anchor: Scalar,
    #[serde(default)]
    pub bottom_anchor: Scalar,
    #[serde(default)]
    pub alignment: Vec2,
    #[serde(default)]
    pub offset: Vec2,
    #[serde(default)]
    pub fixed_width: Option<Scalar>,
    #[serde(default)]
    pub fixed_height: Option<Scalar>,
    #[serde(default = "CompositeUiElement::default_scale")]
    pub scale: Vec2,
    #[serde(default = "CompositeUiElement::default_alpha")]
    pub alpha: Scalar,
    #[serde(default)]
    pub children: Vec<CompositeUiElement>,
    #[serde(skip)]
    pub(crate) dirty: bool,
}

impl Default for CompositeUiElement {
    fn default() -> Self {
        Self {
            id: None,
            theme: None,
            camera_name: "".into(),
            interactive: None,
            element_type: Default::default(),
            margin: Default::default(),
            padding: Default::default(),
            left_anchor: 0.0,
            right_anchor: 0.0,
            top_anchor: 0.0,
            bottom_anchor: 0.0,
            alignment: Default::default(),
            offset: Default::default(),
            fixed_width: None,
            fixed_height: None,
            scale: Self::default_scale(),
            alpha: Self::default_alpha(),
            children: Default::default(),
            dirty: true,
        }
    }
}

impl CompositeUiElement {
    fn default_scale() -> Vec2 {
        Vec2::one()
    }

    fn default_alpha() -> Scalar {
        1.0
    }

    pub fn rebuild(&mut self) {
        self.dirty = true;
    }

    pub fn find(&self, id: &str) -> Option<&CompositeUiElement> {
        if let Some(index) = id.find('/') {
            let part = &id[0..index];
            let test = self.children.iter().enumerate().find(|(i, c)| {
                if let Some(name) = &c.id {
                    name == part
                } else {
                    i.to_string() == part
                }
            });
            if let Some((_, child)) = test {
                if index < id.len() {
                    return child.find(&id[(index + 1)..]);
                } else {
                    return Some(child);
                }
            }
        } else {
            let part = id;
            let test = self.children.iter().enumerate().find(|(i, c)| {
                if let Some(name) = &c.id {
                    name == part
                } else {
                    i.to_string() == part
                }
            });
            if let Some((_, child)) = test {
                return Some(child);
            }
        }
        None
    }

    pub fn find_mut(&mut self, id: &str) -> Option<&mut CompositeUiElement> {
        if let Some(index) = id.find('/') {
            let part = &id[0..index];
            let test = self.children.iter_mut().enumerate().find(|(i, c)| {
                if let Some(name) = &c.id {
                    name == part
                } else {
                    i.to_string() == part
                }
            });
            if let Some((_, child)) = test {
                if index < id.len() {
                    return child.find_mut(&id[(index + 1)..]);
                } else {
                    return Some(child);
                }
            }
        } else {
            let part = id;
            let test = self.children.iter_mut().enumerate().find(|(i, c)| {
                if let Some(name) = &c.id {
                    name == part
                } else {
                    i.to_string() == part
                }
            });
            if let Some((_, child)) = test {
                return Some(child);
            }
        }
        None
    }
}

impl Component for CompositeUiElement {
    type Storage = VecStorage<Self>;
}

impl Prefab for CompositeUiElement {
    fn post_from_prefab(&mut self) {
        self.dirty = true;
    }
}
impl PrefabComponent for CompositeUiElement {}

impl CompositeUiElement {
    pub fn calculate_rect(&self, parent_rect: Rect, themes: &CompositeUiThemes) -> Rect {
        let margin = if let Some(name) = &self.theme {
            if let Some(UiThemed::Image { image_margin, .. }) = themes.themes.get(name) {
                *image_margin
            } else {
                self.margin
            }
        } else {
            self.margin
        };

        let min_width = margin.left + margin.right;
        let min_height = margin.top + margin.bottom;
        let (x, width) = {
            let (x, w) = if let Some(fixed_width) = self.fixed_width {
                let f = parent_rect.w * self.left_anchor + self.padding.left;
                let t = parent_rect.w * self.right_anchor - self.padding.right;
                let o = (f + t) * 0.5;
                (o, fixed_width)
            } else {
                let f = parent_rect.w * self.left_anchor + self.padding.left;
                let t = parent_rect.w * self.right_anchor - self.padding.right;
                (f, (t - f))
            };
            (x, w.max(min_width) * self.scale.x)
        };
        let (y, height) = {
            let (y, h) = if let Some(fixed_height) = self.fixed_height {
                let f = parent_rect.h * self.top_anchor + self.padding.top;
                let t = parent_rect.h * self.bottom_anchor - self.padding.bottom;
                let o = (f + t) * 0.5;
                (o, fixed_height)
            } else {
                let f = parent_rect.h * self.top_anchor + self.padding.top;
                let t = parent_rect.h * self.bottom_anchor - self.padding.bottom;
                (f, (t - f))
            };
            (y, h.max(min_height) * self.scale.y)
        };
        Rect {
            x: self.offset.x + x + parent_rect.x - self.alignment.x * width,
            y: self.offset.y + y + parent_rect.y - self.alignment.y * height,
            w: width,
            h: height,
        }
    }

    pub fn build_commands(
        &self,
        parent_rect: Rect,
        interactibles: &mut CompositeUiInteractibles,
        themes: &CompositeUiThemes,
    ) -> (Vec<Command<'static>>, Rect) {
        let rect = self.calculate_rect(parent_rect, themes);
        if let Some(name) = &self.interactive {
            interactibles.bounding_boxes.insert(name.clone(), rect);
        }
        let mut commands = match &self.element_type {
            UiElementType::Text(text) => {
                let mut text = text.clone();
                text.position = Vec2::new(rect.x, rect.y);
                text.max_width = Some(rect.w);
                let alpha = if let Some(name) = &self.theme {
                    if let Some(UiThemed::Text {
                        font_name,
                        font_size,
                        alpha,
                    }) = themes.themes.get(name)
                    {
                        text.font = font_name.clone();
                        text.size = *font_size;
                        *alpha
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };
                vec![
                    Command::Store,
                    Command::Alpha(alpha),
                    Command::Draw(text.into()),
                    Command::Restore,
                ]
            }
            UiElementType::Image(image) => {
                let (margin, source_rect, image_path, alpha) = if let Some(name) = &self.theme {
                    if let Some(UiThemed::Image {
                        image_margin,
                        image_path,
                        source_rect,
                        alpha,
                    }) = themes.themes.get(name)
                    {
                        (*image_margin, *source_rect, image_path, *alpha)
                    } else {
                        (self.margin, image.source_rect, &image.image_path, 1.0)
                    }
                } else {
                    (self.margin, image.source_rect, &image.image_path, 1.0)
                };

                let sx1 = source_rect.x;
                let sx2 = source_rect.x + margin.left;
                let sx3 = source_rect.x + source_rect.w - margin.right;
                let sy1 = source_rect.y;
                let sy2 = source_rect.y + margin.top;
                let sy3 = source_rect.y + source_rect.h - margin.bottom;

                let sw1 = margin.left;
                let sw2 = source_rect.w - margin.left - margin.right;
                let sw3 = margin.right;
                let sh1 = margin.top;
                let sh2 = source_rect.h - margin.top - margin.bottom;
                let sh3 = margin.bottom;

                let dx1 = rect.x;
                let dx2 = rect.x + margin.left;
                let dx3 = rect.x + rect.w - margin.right;
                let dy1 = rect.y;
                let dy2 = rect.y + margin.top;
                let dy3 = rect.y + rect.h - margin.bottom;

                let dw1 = margin.left;
                let dw2 = rect.w - margin.left - margin.right;
                let dw3 = margin.right;
                let dh1 = margin.top;
                let dh2 = rect.h - margin.top - margin.bottom;
                let dh3 = margin.bottom;

                vec![
                    Command::Store,
                    Command::Alpha(alpha),
                    if let Some(path) = image_path.get(0) {
                        Command::Draw(
                            Image {
                                image: path.to_owned().into(),
                                source: Some(Rect {
                                    x: sx1,
                                    y: sy1,
                                    w: sw1,
                                    h: sh1,
                                }),
                                destination: Some(Rect {
                                    x: dx1,
                                    y: dy1,
                                    w: dw1,
                                    h: dh1,
                                }),
                                alignment: Vec2::new(0.0, 0.0),
                            }
                            .into(),
                        )
                    } else {
                        Command::None
                    },
                    if let Some(path) = image_path.get(1) {
                        Command::Draw(
                            Image {
                                image: path.to_owned().into(),
                                source: Some(Rect {
                                    x: sx2,
                                    y: sy1,
                                    w: sw2,
                                    h: sh1,
                                }),
                                destination: Some(Rect {
                                    x: dx2,
                                    y: dy1,
                                    w: dw2,
                                    h: dh1,
                                }),
                                alignment: Vec2::new(0.0, 0.0),
                            }
                            .into(),
                        )
                    } else {
                        Command::None
                    },
                    if let Some(path) = image_path.get(2) {
                        Command::Draw(
                            Image {
                                image: path.to_owned().into(),
                                source: Some(Rect {
                                    x: sx3,
                                    y: sy1,
                                    w: sw3,
                                    h: sh1,
                                }),
                                destination: Some(Rect {
                                    x: dx3,
                                    y: dy1,
                                    w: dw3,
                                    h: dh1,
                                }),
                                alignment: Vec2::new(0.0, 0.0),
                            }
                            .into(),
                        )
                    } else {
                        Command::None
                    },
                    if let Some(path) = image_path.get(3) {
                        Command::Draw(
                            Image {
                                image: path.to_owned().into(),
                                source: Some(Rect {
                                    x: sx1,
                                    y: sy2,
                                    w: sw1,
                                    h: sh2,
                                }),
                                destination: Some(Rect {
                                    x: dx1,
                                    y: dy2,
                                    w: dw1,
                                    h: dh2,
                                }),
                                alignment: Vec2::new(0.0, 0.0),
                            }
                            .into(),
                        )
                    } else {
                        Command::None
                    },
                    if let Some(path) = image_path.get(4) {
                        Command::Draw(
                            Image {
                                image: path.to_owned().into(),
                                source: Some(Rect {
                                    x: sx2,
                                    y: sy2,
                                    w: sw2,
                                    h: sh2,
                                }),
                                destination: Some(Rect {
                                    x: dx2,
                                    y: dy2,
                                    w: dw2,
                                    h: dh2,
                                }),
                                alignment: Vec2::new(0.0, 0.0),
                            }
                            .into(),
                        )
                    } else {
                        Command::None
                    },
                    if let Some(path) = image_path.get(5) {
                        Command::Draw(
                            Image {
                                image: path.to_owned().into(),
                                source: Some(Rect {
                                    x: sx3,
                                    y: sy2,
                                    w: sw3,
                                    h: sh2,
                                }),
                                destination: Some(Rect {
                                    x: dx3,
                                    y: dy2,
                                    w: dw3,
                                    h: dh2,
                                }),
                                alignment: Vec2::new(0.0, 0.0),
                            }
                            .into(),
                        )
                    } else {
                        Command::None
                    },
                    if let Some(path) = image_path.get(6) {
                        Command::Draw(
                            Image {
                                image: path.to_owned().into(),
                                source: Some(Rect {
                                    x: sx1,
                                    y: sy3,
                                    w: sw1,
                                    h: sh3,
                                }),
                                destination: Some(Rect {
                                    x: dx1,
                                    y: dy3,
                                    w: dw1,
                                    h: dh3,
                                }),
                                alignment: Vec2::new(0.0, 0.0),
                            }
                            .into(),
                        )
                    } else {
                        Command::None
                    },
                    if let Some(path) = image_path.get(7) {
                        Command::Draw(
                            Image {
                                image: path.to_owned().into(),
                                source: Some(Rect {
                                    x: sx2,
                                    y: sy3,
                                    w: sw2,
                                    h: sh3,
                                }),
                                destination: Some(Rect {
                                    x: dx2,
                                    y: dy3,
                                    w: dw2,
                                    h: dh3,
                                }),
                                alignment: Vec2::new(0.0, 0.0),
                            }
                            .into(),
                        )
                    } else {
                        Command::None
                    },
                    if let Some(path) = image_path.get(8) {
                        Command::Draw(
                            Image {
                                image: path.to_owned().into(),
                                source: Some(Rect {
                                    x: sx3,
                                    y: sy3,
                                    w: sw3,
                                    h: sh3,
                                }),
                                destination: Some(Rect {
                                    x: dx3,
                                    y: dy3,
                                    w: dw3,
                                    h: dh3,
                                }),
                                alignment: Vec2::new(0.0, 0.0),
                            }
                            .into(),
                        )
                    } else {
                        Command::None
                    },
                    Command::Restore,
                ]
            }
            _ => vec![],
        };
        for child in &self.children {
            commands.extend(child.build_commands(rect, interactibles, themes).0);
        }
        (commands, rect)
    }
}
