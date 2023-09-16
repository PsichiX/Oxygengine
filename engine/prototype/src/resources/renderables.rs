use crate::resources::*;
use oxygengine_core::prelude::*;
use oxygengine_ha_renderer::prelude::*;

#[cfg(not(feature = "scalar64"))]
use std::f32::consts::TAU;
#[cfg(feature = "scalar64")]
use std::f64::consts::TAU;

pub enum Renderable {
    PushTransform(Transform2d),
    PopTransform,
    PushBlending(MaterialBlending),
    PopBlending,
    PushScissor(Rect),
    PopScissor,
    Mesh(MeshRenderable),
    Sprite(SpriteRenderable),
    Text(TextRenderable),
    Shape(ShapeRenderable),
}

impl From<MeshRenderable> for Renderable {
    fn from(other: MeshRenderable) -> Self {
        Self::Mesh(other)
    }
}

impl From<SpriteRenderable> for Renderable {
    fn from(other: SpriteRenderable) -> Self {
        Self::Sprite(other)
    }
}

impl From<TextRenderable> for Renderable {
    fn from(other: TextRenderable) -> Self {
        Self::Text(other)
    }
}

impl From<ShapeRenderable> for Renderable {
    fn from(other: ShapeRenderable) -> Self {
        Self::Shape(other)
    }
}

#[derive(Debug, Default, Clone)]
pub struct MeshRenderable {
    pub transform: Transform2d,
    pub mesh: HaMeshInstance,
    pub material: HaMaterialInstance,
}

impl MeshRenderable {
    pub fn transform(mut self, value: Transform2d) -> Self {
        self.transform = value;
        self
    }

    pub fn mesh(mut self, value: HaMeshInstance) -> Self {
        self.mesh = value;
        self
    }

    pub fn material(mut self, value: HaMaterialInstance) -> Self {
        self.material = value;
        self
    }
}

#[derive(Debug, Clone)]
pub struct SpriteRenderable {
    pub transform: Transform2d,
    pub image: ImageReference,
    pub tint: Rgba,
    pub tiling: Vec2,
    pub region: Option<Rect>,
}

impl Default for SpriteRenderable {
    fn default() -> Self {
        Self {
            transform: Default::default(),
            image: Default::default(),
            tint: Rgba::white(),
            tiling: Vec2::one(),
            region: None,
        }
    }
}

impl SpriteRenderable {
    pub fn new(name: impl ToString) -> Self {
        Self::default().image(ImageReference::Asset(name.to_string()))
    }

    pub fn transform(mut self, value: Transform2d) -> Self {
        self.transform = value;
        self
    }

    pub fn position(mut self, value: impl Into<Vec2>) -> Self {
        self.transform.position = value.into();
        self
    }

    pub fn rotation(mut self, value: Scalar) -> Self {
        self.transform.rotation = value;
        self
    }

    pub fn size(mut self, value: impl Into<Vec2>) -> Self {
        self.transform.scale = value.into();
        self
    }

    pub fn image(mut self, value: ImageReference) -> Self {
        self.image = value;
        self
    }

    pub fn tint(mut self, value: impl Into<Rgba>) -> Self {
        self.tint = value.into();
        self
    }

    pub fn tiling(mut self, value: impl Into<Vec2>) -> Self {
        self.tiling = value.into();
        self
    }

    pub fn region(mut self, value: impl Into<Rect>) -> Self {
        self.region = Some(value.into());
        self
    }

    pub fn region_from_animation_frame(
        self,
        mut frame: usize,
        mut cols: usize,
        mut rows: usize,
    ) -> Self {
        cols = cols.max(1);
        rows = rows.max(1);
        frame = frame % cols * rows;
        let col = frame % cols;
        let row = frame / rows;
        self.region_from_tile_cell(col, row, cols, rows)
    }

    pub fn region_from_tile_cell(
        self,
        mut col: usize,
        mut row: usize,
        mut cols: usize,
        mut rows: usize,
    ) -> Self {
        cols = cols.max(1);
        rows = rows.max(1);
        col = col.min(cols - 1);
        row = row.min(rows - 1);
        let width = cols as Scalar;
        let height = rows as Scalar;
        self.region(rect(
            col as Scalar / width,
            row as Scalar / height,
            1.0 / width,
            1.0 / height,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct TextRenderable {
    pub transform: Transform2d,
    pub content: HaTextContent,
    pub font: String,
    pub size: Scalar,
    pub color: Rgba,
    pub alignment: Vec2,
    pub bounds_width: Option<Scalar>,
    pub bounds_height: Option<Scalar>,
    pub wrapping: HaTextWrapping,
}

impl Default for TextRenderable {
    fn default() -> Self {
        Self {
            transform: Default::default(),
            content: Default::default(),
            font: Default::default(),
            size: 32.0,
            color: Rgba::white(),
            alignment: 0.0.into(),
            bounds_width: None,
            bounds_height: None,
            wrapping: Default::default(),
        }
    }
}

impl TextRenderable {
    pub fn new(font: impl ToString, content: impl Into<HaTextContent>) -> Self {
        Self::default().font(font).content(content)
    }

    pub fn transform(mut self, value: Transform2d) -> Self {
        self.transform = value;
        self
    }

    pub fn position(mut self, value: impl Into<Vec2>) -> Self {
        self.transform.position = value.into();
        self
    }

    pub fn rotation(mut self, value: Scalar) -> Self {
        self.transform.rotation = value;
        self
    }

    pub fn scale(mut self, value: impl Into<Vec2>) -> Self {
        self.transform.scale = value.into();
        self
    }

    pub fn content(mut self, value: impl Into<HaTextContent>) -> Self {
        self.content = value.into();
        self
    }

    pub fn font(mut self, value: impl ToString) -> Self {
        self.font = value.to_string();
        self
    }

    pub fn size(mut self, value: Scalar) -> Self {
        self.size = value;
        self
    }

    pub fn color(mut self, value: Rgba) -> Self {
        self.color = value;
        self
    }

    pub fn alignment(mut self, value: impl Into<Vec2>) -> Self {
        self.alignment = value.into();
        self
    }

    pub fn bounds_width(mut self, value: Option<Scalar>) -> Self {
        self.bounds_width = value;
        self
    }

    pub fn bounds_height(mut self, value: Option<Scalar>) -> Self {
        self.bounds_height = value;
        self
    }

    pub fn wrapping(mut self, value: HaTextWrapping) -> Self {
        self.wrapping = value;
        self
    }

    pub fn to_text_instance(&self) -> HaTextInstance {
        let mut result = HaTextInstance::default();
        result.set_content(self.content.to_owned());
        result.set_font(self.font.to_owned());
        result.set_size(self.size);
        result.set_color(self.color);
        result.set_alignment(self.alignment);
        result.set_pivot(self.alignment);
        result.set_bounds_width(self.bounds_width);
        result.set_bounds_height(self.bounds_height);
        result.set_wrapping(self.wrapping.to_owned());
        result
    }
}

#[derive(Debug, Clone)]
pub enum ShapeRenderable {
    Points {
        tint: Rgba,
        points: Vec<Vec2>,
        size: Scalar,
    },
    Lines {
        tint: Rgba,
        points: Vec<Vec2>,
        size: Scalar,
    },
    Polygon {
        tint: Rgba,
        points: Vec<Vec2>,
    },
    Triangles {
        tint: Rgba,
        points: Vec<[Vec2; 3]>,
    },
    Circle {
        tint: Rgba,
        center: Vec2,
        radius: Scalar,
        segments: usize,
    },
}

impl ShapeRenderable {
    pub fn factory(&self) -> Result<StaticVertexFactory, MeshError> {
        let (vertices, triangles) = match self {
            ShapeRenderable::Points { tint, points, size } => {
                let color = Vec4::from(*tint);
                let half_size = size * 0.5;
                let vertices = points
                    .iter()
                    .copied()
                    .flat_map(|point| {
                        let point = Vec3::from(point);
                        [
                            SurfaceVertexPC {
                                position: point + vec3(-half_size, -half_size, 0.0),
                                color,
                            },
                            SurfaceVertexPC {
                                position: point + vec3(half_size, -half_size, 0.0),
                                color,
                            },
                            SurfaceVertexPC {
                                position: point + vec3(half_size, half_size, 0.0),
                                color,
                            },
                            SurfaceVertexPC {
                                position: point + vec3(-half_size, half_size, 0.0),
                                color,
                            },
                        ]
                    })
                    .collect::<Vec<_>>();
                let triangles = (0..points.len())
                    .flat_map(|index| {
                        let offset = index as u32 * 4;
                        [
                            (offset, offset + 1, offset + 2),
                            (offset + 2, offset + 3, offset),
                        ]
                    })
                    .collect::<Vec<_>>();
                (vertices, triangles)
            }
            ShapeRenderable::Lines { tint, points, size } => {
                let color = Vec4::from(*tint);
                let half_size = size * 0.5;
                let vertices = points
                    .windows(2)
                    .flat_map(|points| {
                        let from = points[0];
                        let to = points[1];
                        let direction = to - from;
                        let forward = direction.normalized();
                        let right = vec2(-forward.y, forward.x);
                        [
                            SurfaceVertexPC {
                                position: (from - right * half_size).into(),
                                color,
                            },
                            SurfaceVertexPC {
                                position: (to - right * half_size).into(),
                                color,
                            },
                            SurfaceVertexPC {
                                position: (to + right * half_size).into(),
                                color,
                            },
                            SurfaceVertexPC {
                                position: (from + right * half_size).into(),
                                color,
                            },
                        ]
                    })
                    .chain(points.windows(3).map(|points| SurfaceVertexPC {
                        position: points[1].into(),
                        color,
                    }))
                    .collect::<Vec<_>>();
                let triangles = (0..(points.len() - 1))
                    .flat_map(|index| {
                        let offset = index as u32 * 4;
                        [
                            (offset, offset + 1, offset + 2),
                            (offset + 2, offset + 3, offset),
                        ]
                    })
                    .chain((0..(points.len() - 2)).flat_map(|index| {
                        let index = index as u32;
                        let knots_offset = (points.len() - 1) as u32 * 4;
                        let bar_offset = index * 4;
                        [
                            (knots_offset + index, bar_offset + 1, bar_offset + 4),
                            (knots_offset + index, bar_offset + 2, bar_offset + 7),
                        ]
                    }))
                    .collect::<Vec<_>>();
                (vertices, triangles)
            }
            ShapeRenderable::Polygon { tint, points } => {
                let color = Vec4::from(*tint);
                let vertices = points
                    .iter()
                    .copied()
                    .map(|point| SurfaceVertexPC {
                        position: Vec3::from(point),
                        color,
                    })
                    .collect::<Vec<_>>();
                let triangles = (0..points.len() - 2)
                    .map(|index| (0u32, index as u32 + 1, index as u32 + 2))
                    .collect::<Vec<_>>();
                (vertices, triangles)
            }
            ShapeRenderable::Triangles { tint, points } => {
                let color = Vec4::from(*tint);
                let vertices = points
                    .iter()
                    .flat_map(|[a, b, c]| {
                        [
                            SurfaceVertexPC {
                                position: Vec3::from(*a),
                                color,
                            },
                            SurfaceVertexPC {
                                position: Vec3::from(*b),
                                color,
                            },
                            SurfaceVertexPC {
                                position: Vec3::from(*c),
                                color,
                            },
                        ]
                    })
                    .collect::<Vec<_>>();
                let triangles = (0..points.len())
                    .map(|index| {
                        let index = index as u32 * 3;
                        (index, index + 1, index + 2)
                    })
                    .collect::<Vec<_>>();
                (vertices, triangles)
            }
            ShapeRenderable::Circle {
                tint,
                center,
                radius,
                segments,
            } => {
                let color = Vec4::from(*tint);
                let center = *center;
                let radius = *radius;
                let segments = *segments;
                let vertices = std::iter::once(SurfaceVertexPC {
                    position: center.into(),
                    color,
                })
                .chain((0..segments).map(|index| {
                    let angle = index as Scalar * TAU / segments as Scalar;
                    let (y, x) = angle.sin_cos();
                    SurfaceVertexPC {
                        position: (center + vec2(x, y) * radius).into(),
                        color,
                    }
                }))
                .collect::<Vec<_>>();
                let triangles = (0..segments)
                    .map(|index| {
                        (
                            0u32,
                            (index % segments) as u32 + 1,
                            ((index + 1) % segments) as u32 + 1,
                        )
                    })
                    .collect::<Vec<_>>();
                (vertices, triangles)
            }
        };
        let mut result = StaticVertexFactory::new(
            SurfaceVertexPC::vertex_layout()?,
            vertices.len(),
            triangles.len(),
            MeshDrawMode::Triangles,
        );
        result.vertices(&vertices, None)?;
        result.triangles(&triangles, None)?;
        Ok(result)
    }
}

pub struct Renderables {
    pub(crate) buffer_stack: Vec<Vec<Renderable>>,
    pub buffer_resize_count: usize,
    pub sprite_mesh_reference: MeshReference,
    pub sprite_material_reference: MaterialReference,
    pub sprite_filtering: ImageFiltering,
    pub text_material_reference: MaterialReference,
    pub text_pool_resize_count: usize,
    pub shape_material_reference: MaterialReference,
    pub shape_pool_resize_count: usize,
}

impl Default for Renderables {
    fn default() -> Self {
        Self {
            buffer_stack: vec![Default::default()],
            buffer_resize_count: 1024,
            sprite_mesh_reference: MeshReference::Asset("@mesh/surface/quad/pt".to_owned()),
            sprite_material_reference: MaterialReference::Asset(
                "@material/graph/prototype/sprite".to_owned(),
            ),
            sprite_filtering: ImageFiltering::Linear,
            text_material_reference: MaterialReference::Asset(
                "@material/graph/surface/flat/text".to_owned(),
            ),
            text_pool_resize_count: 64,
            shape_material_reference: MaterialReference::Asset(
                "@material/graph/surface/flat/color".to_owned(),
            ),
            shape_pool_resize_count: 64,
        }
    }
}

impl Renderables {
    pub fn depth(&self) -> usize {
        self.buffer_stack.len()
    }

    pub fn begin(&mut self) {
        self.buffer_stack.push(Default::default());
    }

    pub fn end(&mut self) {
        if let Some(buffer) = self.consume() {
            self.extend(buffer);
        }
    }

    pub fn collapse(&mut self) {
        while let Some(buffer) = self.consume() {
            self.extend(buffer);
        }
    }

    pub fn consume(&mut self) -> Option<Vec<Renderable>> {
        if self.buffer_stack.len() > 1 {
            self.buffer_stack.pop()
        } else {
            None
        }
    }

    pub fn extend(&mut self, buffer: impl IntoIterator<Item = Renderable>) {
        if let Some(last) = self.buffer_stack.last_mut() {
            last.extend(buffer);
        }
    }

    pub fn scope<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Self),
    {
        self.begin();
        f(self);
        self.end();
    }

    pub fn draw(&mut self, renderable: impl Into<Renderable>) {
        if self.buffer_stack.is_empty() {
            self.buffer_stack.push(Default::default());
        }
        let last = self.buffer_stack.last_mut().unwrap();
        if last.len() == last.capacity() && self.buffer_resize_count > 0 {
            last.reserve(self.buffer_resize_count);
        }
        last.push(renderable.into());
    }
}
