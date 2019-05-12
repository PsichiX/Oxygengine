use crate::{
    composite_renderer::{Effect, Renderable},
    math::{Grid2d, Mat2d, Rect, Scalar, Vec2},
};
use core::ecs::{Component, DenseVecStorage, HashMapStorage, VecStorage};
use std::{borrow::Cow, f32::consts::PI};

#[derive(Debug, Copy, Clone)]
pub struct CompositeVisibility(pub bool);

impl Component for CompositeVisibility {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct CompositeRenderable(pub Renderable<'static>);

impl Component for CompositeRenderable {
    type Storage = DenseVecStorage<Self>;
}

impl From<Renderable<'static>> for CompositeRenderable {
    fn from(value: Renderable<'static>) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub struct CompositeRenderableStroke(pub Scalar);

impl Component for CompositeRenderableStroke {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompositeTransform {
    translation: Vec2,
    rotation: Scalar,
    scale: Vec2,
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

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct CompositeRenderDepth(pub Scalar);

impl Component for CompositeRenderDepth {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct CompositeRenderAlpha(pub Scalar);

impl Component for CompositeRenderAlpha {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Default, Clone)]
pub struct CompositeEffect(pub Effect);

impl Component for CompositeEffect {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct CompositeCamera {
    pub scaling: CompositeScalingMode,
    pub tags: Vec<Cow<'static, str>>,
}

impl Component for CompositeCamera {
    type Storage = HashMapStorage<Self>;
}

impl Default for CompositeCamera {
    fn default() -> Self {
        Self {
            scaling: CompositeScalingMode::None,
            tags: vec![],
        }
    }
}

impl CompositeCamera {
    pub fn new(scaling: CompositeScalingMode) -> Self {
        Self {
            scaling,
            tags: vec![],
        }
    }

    pub fn view_matrix(&self, transform: &CompositeTransform, screen_size: Vec2) -> Mat2d {
        let wh = screen_size.x * 0.5;
        let hh = screen_size.y * 0.5;
        let scale = if screen_size.x > screen_size.y {
            screen_size.y
        } else {
            screen_size.x
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
}

#[derive(Debug, Clone)]
pub struct CompositeSprite {
    sheet_frame: Option<(Cow<'static, str>, Cow<'static, str>)>,
    pub(crate) dirty: bool,
}

impl Default for CompositeSprite {
    fn default() -> Self {
        Self {
            sheet_frame: None,
            dirty: false,
        }
    }
}

impl CompositeSprite {
    pub fn new(sheet: Cow<'static, str>, frame: Cow<'static, str>) -> Self {
        Self {
            sheet_frame: Some((sheet, frame)),
            dirty: true,
        }
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
}

impl Component for CompositeSprite {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Debug, Default, Copy, Clone)]
pub struct TileCell {
    pub col: usize,
    pub row: usize,
    pub flip_x: bool,
    pub flip_y: bool,
    pub rotate: TileRotate,
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

    pub fn is_abnormal(&self) -> bool {
        self.flip_x || self.flip_y || self.rotate != TileRotate::Degrees0
    }

    pub fn matrix(&self) -> Mat2d {
        match self.rotate {
            TileRotate::Degrees0 => Mat2d::default(),
            TileRotate::Degrees90 => Mat2d::rotation(PI * 0.5),
            TileRotate::Degrees180 => Mat2d::rotation(PI),
            TileRotate::Degrees270 => Mat2d::rotation(PI * 1.5),
        }
    }

    pub fn destination(&self, col: usize, row: usize, width: Scalar, height: Scalar) -> Rect {
        match self.rotate {
            TileRotate::Degrees0 => {
                let x = col as Scalar;
                let y = row as Scalar;
                [x * width, y * height, width, height].into()
            }
            TileRotate::Degrees90 => {
                let x = row as Scalar;
                let y = -(col as Scalar + 1.0);
                [x * width, y * height, width, height].into()
            }
            TileRotate::Degrees180 => {
                let x = -(col as Scalar + 1.0);
                let y = -(row as Scalar + 1.0);
                [x * width, y * height, width, height].into()
            }
            TileRotate::Degrees270 => {
                let x = -(row as Scalar + 1.0);
                let y = col as Scalar;
                [x * width, y * height, width, height].into()
            }
        }
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

#[derive(Debug, Clone)]
pub struct CompositeTilemap {
    tileset: Option<Cow<'static, str>>,
    grid: Grid2d<TileCell>,
    pub(crate) dirty: bool,
}

impl Default for CompositeTilemap {
    fn default() -> Self {
        Self {
            tileset: None,
            grid: Default::default(),
            dirty: false,
        }
    }
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
}

impl Component for CompositeTilemap {
    type Storage = VecStorage<Self>;
}
