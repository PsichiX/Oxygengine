use crate::math::{Color, Rect, Scalar, Vec2};
use core::error::*;
use std::ops::Range;

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

#[derive(Debug, Default, Clone)]
pub struct Text<'a> {
    pub color: Color,
    pub font: &'a str,
    pub align: TextAlign,
    pub text: &'a str,
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
    pub image: &'a str,
    pub source: Rect,
    pub destination: Rect,
}

#[derive(Debug, Clone)]
pub enum Renderable<'a> {
    Rectangle(Rectangle),
    Text(Text<'a>),
    Path(Path),
    Image(Image<'a>),
}

#[derive(Debug, Clone)]
pub enum Transformation {
    Translate(Vec2),
    Rotate(Scalar),
    Scale(Vec2),
    /// (a, b, c, d, e, f)
    Transform(Scalar, Scalar, Scalar, Scalar, Scalar, Scalar),
}

#[derive(Debug, Clone)]
pub enum Command<'a> {
    None,
    Draw(Renderable<'a>),
    /// (line width, renderable)
    Stroke(Scalar, Renderable<'a>),
    Transform(Transformation),
    Store,
    Restore,
}

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub render_ops: usize,
    pub renderables: usize,
}

#[derive(Debug, Clone)]
pub struct State {
    pub clear_color: Option<Color>,
    stats: Stats,
}

impl Default for State {
    fn default() -> Self {
        Self {
            clear_color: Some(Color::black()),
            stats: Stats::default(),
        }
    }
}

impl State {
    pub fn new(clear_color: Option<Color>) -> Self {
        Self {
            clear_color,
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

pub trait CompositeRenderer {
    fn execute<'a, I>(&mut self, commands: I) -> Result<()>
    where
        I: IntoIterator<Item = Command<'a>>;

    fn state(&self) -> &State;

    fn state_mut(&mut self) -> &mut State;

    fn viewport(&self) -> Rect;

    fn update_state(&mut self);

    // fn register_image<T>(name: &str, image: T);
    //
    // fn unregister_image(name: &str);
}
