use crate::math::{Color, Rect, Scalar, Vec2};
use core::{assets::database::AssetsDatabase, error::*};
use std::{borrow::Cow, ops::Range};

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Default, Clone)]
pub struct Text<'a> {
    pub color: Color,
    pub font: Cow<'a, str>,
    pub align: TextAlign,
    pub text: Cow<'a, str>,
    pub position: Vec2,
    pub size: Scalar,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Default, Clone)]
pub struct Path {
    pub color: Color,
    pub elements: Vec<PathElement>,
}

#[derive(Debug, Default, Clone)]
pub struct Image<'a> {
    pub image: Cow<'a, str>,
    pub source: Option<Rect>,
    pub destination: Option<Rect>,
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

#[derive(Debug, Clone)]
pub enum Renderable<'a> {
    None,
    Rectangle(Rectangle),
    Text(Text<'a>),
    Path(Path),
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

impl<'a> From<Image<'a>> for Renderable<'a> {
    fn from(image: Image<'a>) -> Self {
        Renderable::Image(image)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub view_size: Vec2,
    pub render_ops: usize,
    pub renderables: usize,
    pub fps: f64,
    pub delta_time: f64,
    pub images_count: usize,
    pub surfaces_count: usize,
}

#[derive(Debug, Clone)]
pub struct RenderState {
    pub clear_color: Option<Color>,
    pub image_smoothing: bool,
    stats: Stats,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            clear_color: Some(Color::black()),
            image_smoothing: true,
            stats: Stats::default(),
        }
    }
}

impl RenderState {
    pub fn new(clear_color: Option<Color>) -> Self {
        Self {
            clear_color,
            image_smoothing: true,
            stats: Stats::default(),
        }
    }

    pub fn with_image_smoothing(clear_color: Option<Color>, image_smoothing: bool) -> Self {
        Self {
            clear_color,
            image_smoothing,
            stats: Stats::default(),
        }
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
