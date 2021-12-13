use crate::{
    asset_protocols::font::FontAsset,
    components::text_instance::{HaTextElement, HaTextInstance},
    material::domains::surface::SurfaceTextDomain,
    math::*,
    mesh::{vertex_factory::StaticVertexFactory, MeshDrawMode, MeshError},
};

#[derive(Debug, Clone)]
struct TextGlyph {
    character: char,
    page: usize,
    position: Vec2,
    uvs: Rect,
    size: Vec2,
    color: Rgba,
    outline: Rgba,
    thickness: f32,
    cursive_shift: f32,
    baseline: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct SurfaceTextFactory;

impl SurfaceTextFactory {
    pub fn factory<T>(
        text: &HaTextInstance,
        font: &FontAsset,
    ) -> Result<StaticVertexFactory, MeshError>
    where
        T: SurfaceTextDomain,
    {
        let vertex_layout = T::vertex_layout()?;
        if !T::has_attribute("position") {
            return Err(MeshError::MissingRequiredLayoutAttribute(
                vertex_layout,
                "position".to_owned(),
            ));
        }
        if !T::has_attribute("textureCoord") {
            return Err(MeshError::MissingRequiredLayoutAttribute(
                vertex_layout,
                "textureCoord".to_owned(),
            ));
        }
        if !T::has_attribute("color") {
            return Err(MeshError::MissingRequiredLayoutAttribute(
                vertex_layout,
                "color".to_owned(),
            ));
        }
        if !T::has_attribute("outline") {
            return Err(MeshError::MissingRequiredLayoutAttribute(
                vertex_layout,
                "outline".to_owned(),
            ));
        }
        if !T::has_attribute("thickness") {
            return Err(MeshError::MissingRequiredLayoutAttribute(
                vertex_layout,
                "thickness".to_owned(),
            ));
        }
        let count = text.glyphs_count();
        let bounds_width = text.bounds_width().unwrap_or(f32::INFINITY);
        let bounds_height = text.bounds_height().unwrap_or(f32::INFINITY);
        let extra_y = text.lines_extra_space();
        let mut line_cache = Vec::<TextGlyph>::with_capacity(count);
        let mut lines = Vec::with_capacity(text.lines_count());
        let mut x = 0.0;
        let mut y = 0.0;
        let mut width = text.bounds_width().unwrap_or(0.0);
        let mut height = text.bounds_height().unwrap_or(0.0);
        let mut line_width: f32 = 0.0;
        let mut line_height: f32 = 0.0;
        let mut line_base: f32 = 0.0;

        macro_rules! move_to_new_line {
            (@push) => {
                {
                    for glyph in &mut line_cache {
                        glyph.position.y += line_base - glyph.baseline;
                    }
                    lines.push((
                        line_width,
                        std::mem::replace(&mut line_cache, Vec::with_capacity(count)),
                    ));
                    y += line_height;
                    height = height.max(y);
                }
            };
            (@reset) => {
                {
                    x = 0.0;
                    line_width = 0.0;
                    line_height = 0.0;
                    line_base = 0.0;
                }
            };
            () => {
                {
                    move_to_new_line!(@push);
                    move_to_new_line!(@reset)
                }
            };
        }

        for element in text.iter() {
            match element {
                HaTextElement::Invalid => {}
                HaTextElement::NewLine => {
                    move_to_new_line!();
                }
                HaTextElement::Glyph {
                    character,
                    size,
                    color,
                    outline,
                    thickness,
                    cursive,
                    ..
                } => {
                    if let Some(c) = font.characters.get(&character) {
                        if let Some((page_size, _)) = font.pages_image_assets.get(c.page) {
                            let scale = size / font.line_height as f32;
                            let xadvance = c.line_advance * scale;
                            let yadvance = (font.line_height as f32 + extra_y) * scale;
                            if x + xadvance > bounds_width {
                                move_to_new_line!();
                                // TODO: use wrapping to break lines: `wrapping.can_wrap(character)`
                            }
                            if x + xadvance > bounds_width || y + yadvance > bounds_height {
                                break;
                            }
                            let baseline = (font.line_base as f32 + extra_y) * scale;
                            let size = c.image_size * scale;
                            let offset = c.offset * scale;
                            let cursive_shift = yadvance * cursive;
                            line_height = line_height.max(yadvance);
                            line_cache.push(TextGlyph {
                                character,
                                page: c.page as _,
                                position: Vec2::new(x, y) + offset,
                                uvs: Rect::new(
                                    c.image_location.x / page_size.x,
                                    c.image_location.y / page_size.y,
                                    c.image_size.x / page_size.x,
                                    c.image_size.y / page_size.y,
                                ),
                                size,
                                color,
                                outline,
                                thickness,
                                cursive_shift,
                                baseline,
                            });
                            x += xadvance;
                            line_width = line_width.max(x);
                            width = width.max(line_width);
                            line_base = line_base.max(baseline);
                        }
                    }
                }
            }
        }
        move_to_new_line!(@push);

        let yalign = (height - y) * text.alignment().y;
        let xpivot = width * text.pivot().x;
        let ypivot = height * text.pivot().y;
        for (line_width, line) in &mut lines {
            let xalign = (width - *line_width) * text.alignment().x;
            for glyph in line {
                glyph.position.x += xalign - xpivot;
                glyph.position.y += yalign - ypivot;
            }
        }
        let vertex_count = count * 4;
        let triangle_count = count * 2;
        let mut result = StaticVertexFactory::new(
            T::vertex_layout()?,
            vertex_count,
            triangle_count,
            MeshDrawMode::Triangles,
        );
        let mut positions = Vec::with_capacity(vertex_count);
        for glyph in lines.iter().flat_map(|(_, glyphs)| glyphs) {
            positions.push(Vec3::new(
                glyph.position.x + glyph.cursive_shift,
                glyph.position.y,
                0.0,
            ));
            positions.push(Vec3::new(
                glyph.position.x + glyph.size.x + glyph.cursive_shift,
                glyph.position.y,
                0.0,
            ));
            positions.push(Vec3::new(
                glyph.position.x + glyph.size.x - glyph.cursive_shift,
                glyph.position.y + glyph.size.y,
                0.0,
            ));
            positions.push(Vec3::new(
                glyph.position.x - glyph.cursive_shift,
                glyph.position.y + glyph.size.y,
                0.0,
            ));
        }
        result.vertices_vec3f("position", &positions, None)?;
        let mut texture_coords = Vec::with_capacity(vertex_count);
        for glyph in lines.iter().flat_map(|(_, glyphs)| glyphs) {
            texture_coords.push(Vec3::new(glyph.uvs.x, glyph.uvs.y, glyph.page as _));
            texture_coords.push(Vec3::new(
                glyph.uvs.x + glyph.uvs.w,
                glyph.uvs.y,
                glyph.page as _,
            ));
            texture_coords.push(Vec3::new(
                glyph.uvs.x + glyph.uvs.w,
                glyph.uvs.y + glyph.uvs.h,
                glyph.page as _,
            ));
            texture_coords.push(Vec3::new(
                glyph.uvs.x,
                glyph.uvs.y + glyph.uvs.h,
                glyph.page as _,
            ));
        }
        result.vertices_vec3f("textureCoord", &texture_coords, None)?;
        let mut colors = Vec::with_capacity(vertex_count);
        for glyph in lines.iter().flat_map(|(_, glyphs)| glyphs) {
            colors.push(glyph.color.into());
            colors.push(glyph.color.into());
            colors.push(glyph.color.into());
            colors.push(glyph.color.into());
        }
        result.vertices_vec4f("color", &colors, None)?;
        let mut outlines = Vec::with_capacity(vertex_count);
        for glyph in lines.iter().flat_map(|(_, glyphs)| glyphs) {
            outlines.push(glyph.outline.into());
            outlines.push(glyph.outline.into());
            outlines.push(glyph.outline.into());
            outlines.push(glyph.outline.into());
        }
        result.vertices_vec4f("outline", &outlines, None)?;
        let indices = (0..triangle_count)
            .map(|index| {
                let i = 4 * (index / 2) as u32;
                if index % 2 == 0 {
                    (i, i + 1, i + 2)
                } else {
                    (i + 2, i + 3, i)
                }
            })
            .collect::<Vec<_>>();
        result.triangles(&indices, None)?;
        Ok(result)
    }
}
