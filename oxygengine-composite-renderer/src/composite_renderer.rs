use crate::math::{Color, Rect, Vec2};
use core::{
    assets::{asset::AssetID, database::AssetsDatabase},
    error::*,
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, ops::Range};

#[derive(Ignite, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl Default for TextAlign {
    fn default() -> Self {
        TextAlign::Left
    }
}

#[derive(Ignite, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TextBaseLine {
    Top,
    Middle,
    Bottom,
    Alphabetic,
    Hanging,
}

impl Default for TextBaseLine {
    fn default() -> Self {
        TextBaseLine::Top
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Rectangle {
    pub color: Color,
    pub rect: Rect,
}

impl Rectangle {
    pub fn align(mut self, factor: Vec2) -> Self {
        self.rect = self.rect.align(factor);
        self
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Text<'a> {
    #[serde(default)]
    pub color: Color,
    #[serde(default)]
    pub font: Cow<'a, str>,
    #[serde(default)]
    pub align: TextAlign,
    #[serde(default)]
    pub baseline: TextBaseLine,
    #[serde(default)]
    pub text: Cow<'a, str>,
    #[serde(default)]
    pub position: Vec2,
    #[serde(default = "Text::default_size")]
    pub size: Scalar,
    #[serde(default)]
    pub max_width: Option<Scalar>,
}

impl<'a> Text<'a> {
    fn default_size() -> Scalar {
        32.0
    }

    pub fn new(font: &'a str, text: &'a str) -> Self {
        Self {
            color: Default::default(),
            font: font.into(),
            align: Default::default(),
            baseline: Default::default(),
            text: text.into(),
            position: 0.0.into(),
            size: 32.0,
            max_width: None,
        }
    }

    pub fn new_owned(font: String, text: String) -> Self {
        Self {
            color: Default::default(),
            font: font.into(),
            align: Default::default(),
            baseline: Default::default(),
            text: text.into(),
            position: 0.0.into(),
            size: 32.0,
            max_width: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn baseline(mut self, baseline: TextBaseLine) -> Self {
        self.baseline = baseline;
        self
    }

    pub fn position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    pub fn size(mut self, size: Scalar) -> Self {
        self.size = size;
        self
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum PathElement {
    MoveTo(Vec2),
    LineTo(Vec2),
    /// (control point A, control point B, point)
    BezierCurveTo(Vec2, Vec2, Vec2),
    /// (control point, point)
    QuadraticCurveTo(Vec2, Vec2),
    /// (point, radius, angles range)
    Arc(Vec2, Scalar, Range<Scalar>),
    /// (point, radius, rotation, angles range)
    Ellipse(Vec2, Vec2, Scalar, Range<Scalar>),
    Rectangle(Rect),
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Path {
    #[serde(default)]
    pub color: Color,
    #[serde(default)]
    pub elements: Vec<PathElement>,
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Mask {
    #[serde(default)]
    pub elements: Vec<PathElement>,
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct Image<'a> {
    #[serde(default)]
    pub image: Cow<'a, str>,
    #[serde(default)]
    pub source: Option<Rect>,
    #[serde(default)]
    pub destination: Option<Rect>,
    #[serde(default)]
    pub alignment: Vec2,
}

impl<'a> Image<'a> {
    pub fn new(image: &'a str) -> Self {
        Self {
            image: image.into(),
            source: None,
            destination: None,
            alignment: 0.0.into(),
        }
    }

    pub fn new_owned(image: String) -> Self {
        Self {
            image: image.into(),
            source: None,
            destination: None,
            alignment: 0.0.into(),
        }
    }

    pub fn source(mut self, rect: Option<Rect>) -> Self {
        self.source = rect;
        self
    }

    pub fn destination(mut self, rect: Option<Rect>) -> Self {
        self.destination = rect;
        self
    }

    pub fn align(mut self, factor: Vec2) -> Self {
        self.alignment = factor;
        self
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum Renderable<'a> {
    None,
    Rectangle(Rectangle),
    FullscreenRectangle(Color),
    Text(Text<'a>),
    Path(Path),
    Mask(Mask),
    Image(Image<'a>),
    Commands(Vec<Command<'a>>),
}

impl<'a> From<()> for Renderable<'a> {
    fn from(_: ()) -> Self {
        Renderable::None
    }
}

impl<'a> From<Rectangle> for Renderable<'a> {
    fn from(rect: Rectangle) -> Self {
        Renderable::Rectangle(rect)
    }
}

impl<'a> From<Text<'a>> for Renderable<'a> {
    fn from(text: Text<'a>) -> Self {
        Renderable::Text(text)
    }
}

impl<'a> From<Path> for Renderable<'a> {
    fn from(path: Path) -> Self {
        Renderable::Path(path)
    }
}

impl<'a> From<Mask> for Renderable<'a> {
    fn from(mask: Mask) -> Self {
        Renderable::Mask(mask)
    }
}

impl<'a> From<Image<'a>> for Renderable<'a> {
    fn from(image: Image<'a>) -> Self {
        Renderable::Image(image)
    }
}

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    SourceOver,
    SourceIn,
    SourceOut,
    SourceAtop,
    DestinationOver,
    DestinationIn,
    DestinationOut,
    DestinationAtop,
    Lighter,
    Copy,
    Xor,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl Default for Effect {
    fn default() -> Self {
        Effect::SourceOver
    }
}

impl ToString for Effect {
    fn to_string(&self) -> String {
        use Effect::*;
        match self {
            SourceOver => "source-over",
            SourceIn => "source-in",
            SourceOut => "source-out",
            SourceAtop => "source-atop",
            DestinationOver => "destination-over",
            DestinationIn => "destination-in",
            DestinationOut => "destination-out",
            DestinationAtop => "destination-atop",
            Lighter => "lighter",
            Copy => "copy",
            Xor => "xor",
            Multiply => "multiply",
            Screen => "screen",
            Overlay => "overlay",
            Darken => "darken",
            Lighten => "lighten",
            ColorDodge => "color-dodge",
            ColorBurn => "color-burn",
            HardLight => "hard-light",
            SoftLight => "soft-light",
            Difference => "difference",
            Exclusion => "exclusion",
            Hue => "hue",
            Saturation => "saturation",
            Color => "color",
            Luminosity => "luminosity",
        }
        .to_owned()
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum Command<'a> {
    None,
    Draw(Renderable<'a>),
    /// (line width, renderable)
    Stroke(Scalar, Renderable<'a>),
    /// (a, b, c, d, e, f)
    Transform(Scalar, Scalar, Scalar, Scalar, Scalar, Scalar),
    Effect(Effect),
    Alpha(Scalar),
    Store,
    Restore,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub view_size: Vec2,
    pub render_ops: usize,
    pub renderables: usize,
    pub fps: Scalar,
    pub delta_time: Scalar,
    pub images_count: usize,
    pub fontfaces_count: usize,
    pub surfaces_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderState {
    pub clear_color: Option<Color>,
    pub image_smoothing: bool,
    pub image_source_inner_margin: Scalar,
    stats: Stats,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            clear_color: Some(Color::black()),
            image_smoothing: true,
            image_source_inner_margin: 0.0,
            stats: Stats::default(),
        }
    }
}

impl RenderState {
    pub fn new(clear_color: Option<Color>) -> Self {
        Self {
            clear_color,
            image_smoothing: true,
            image_source_inner_margin: 0.0,
            stats: Stats::default(),
        }
    }

    pub fn clear_color(mut self, clear_color: Option<Color>) -> Self {
        self.clear_color = clear_color;
        self
    }

    pub fn image_smoothing(mut self, image_smoothing: bool) -> Self {
        self.image_smoothing = image_smoothing;
        self
    }

    pub fn image_source_inner_margin(mut self, image_source_inner_margin: Scalar) -> Self {
        self.image_source_inner_margin = image_source_inner_margin;
        self
    }

    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    pub fn set_stats(&mut self, stats: Stats) {
        self.stats = stats;
    }
}

pub trait CompositeRenderer: Send + Sync {
    // -> (render ops, renderables)
    fn execute<'a, I>(&mut self, commands: I) -> Result<(usize, usize)>
    where
        I: IntoIterator<Item = Command<'a>>;

    fn images_count(&self) -> usize {
        0
    }

    fn fontfaces_count(&self) -> usize {
        0
    }

    fn surfaces_count(&self) -> usize {
        0
    }

    fn state(&self) -> &RenderState;

    fn state_mut(&mut self) -> &mut RenderState;

    fn view_size(&self) -> Vec2;

    fn update_state(&mut self) {}

    fn update_cache(&mut self, _assets: &AssetsDatabase) {}

    fn create_surface(&mut self, name: &str, width: usize, height: usize) -> bool;

    fn destroy_surface(&mut self, name: &str) -> bool;

    fn has_surface(&mut self, name: &str) -> bool;

    fn get_surface_size(&self, name: &str) -> Option<(usize, usize)>;

    fn update_surface<'a, I>(&mut self, name: &str, commands: I) -> Result<(usize, usize)>
    where
        I: IntoIterator<Item = Command<'a>>;
}

pub trait CompositeRendererResources<T> {
    fn add_resource(&mut self, id: String, resource: T) -> Result<AssetID>;

    fn remove_resource(&mut self, id: AssetID) -> Result<T>;
}
