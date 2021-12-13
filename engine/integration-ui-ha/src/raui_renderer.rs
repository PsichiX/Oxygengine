use crate::systems::render_ui_stage::HaRenderUiStageSystemCache;
use oxygengine_core::prelude::*;
use oxygengine_ha_renderer::prelude::*;
use oxygengine_user_interface::raui::core::{
    layout::{CoordsMapping, Layout},
    renderer::Renderer,
    widget::unit::{
        text::{TextBoxHorizontalAlign, TextBoxVerticalAlign},
        WidgetUnit,
    },
};
use raui_tesselate_renderer::{
    renderer::TesselateRenderer,
    tesselation::{Batch, Tesselation, TesselationVerticesFormat},
    Error as RauiError,
};
use std::{collections::HashMap, ops::Range};

#[derive(Debug, Clone)]
pub enum Error {
    Raui(RauiError),
    Mesh(MeshError),
}

impl From<RauiError> for Error {
    fn from(error: RauiError) -> Self {
        Self::Raui(error)
    }
}

impl From<MeshError> for Error {
    fn from(error: MeshError) -> Self {
        Self::Mesh(error)
    }
}

pub struct RauiRenderer<'a> {
    renderer: TesselateRenderer<'a>,
    clip_stack: Vec<Rect>,
    out_batches: &'a mut Vec<RenderBatch>,
    images_map: &'a HashMap<String, AssetId>,
    fonts_map: &'a HashMap<String, AssetId>,
    assets: &'a AssetsDatabase,
}

impl<'a> RauiRenderer<'a> {
    pub fn new(
        cache: &'a mut HaRenderUiStageSystemCache,
        assets: &'a AssetsDatabase,
        out_batches: &'a mut Vec<RenderBatch>,
    ) -> Self {
        Self {
            renderer: TesselateRenderer::with_capacity(
                TesselationVerticesFormat::Interleaved,
                (),
                &cache.atlas_mapping,
                &cache.image_sizes,
                64,
            ),
            clip_stack: Vec::with_capacity(64),
            out_batches,
            images_map: &cache.images_map,
            fonts_map: &cache.fonts_map,
            assets,
        }
    }
}

impl<'a> Renderer<StreamingVertexFactory, Error> for RauiRenderer<'a> {
    fn render(
        &mut self,
        tree: &WidgetUnit,
        mapping: &CoordsMapping,
        layout: &Layout,
    ) -> Result<StreamingVertexFactory, Error> {
        type V = SurfaceVertexText;

        self.clip_stack.clear();
        let tesselation = self
            .renderer
            .render(tree, mapping, layout)?
            .optimized_batches();
        let layout = V::vertex_layout()?;
        let Tesselation {
            vertices,
            indices,
            batches,
        } = tesselation;
        let vertices = vertices.as_interleaved().unwrap();
        let mut stream = StreamingVertexFactory::new(layout.to_owned(), MeshDrawMode::Triangles);
        let mut factory = StaticVertexFactory::new(
            layout,
            vertices.len(),
            indices.len() / 3,
            MeshDrawMode::Triangles,
        );
        // TODO: dear gods, what an abommination - please consider taking data layout into the account.
        unsafe {
            let stride = std::mem::size_of::<V>();
            for (from, to) in vertices.iter().zip(
                factory
                    .access_raw_vertices(0)
                    .unwrap()
                    .chunks_exact_mut(stride),
            ) {
                let to = to.as_mut_ptr() as *mut V;
                (*to).position = vec3(from.position.x, from.position.y, 0.0);
                // TODO: page index should be put here.
                (*to).texture_coord = vec3(from.tex_coord.x, from.tex_coord.y, 0.0);
                (*to).color = vec4(from.color.r, from.color.g, from.color.b, from.color.a);
            }
            factory.access_raw_indices().copy_from_slice(&indices);
        }
        stream.write_from(&factory)?;
        self.out_batches.clear();
        self.out_batches.reserve(batches.len());
        let scale = mapping.scale();
        let font_scale = scale.x.max(scale.y);
        for batch in batches {
            self.out_batches.push(match batch {
                Batch::ColoredTriangles(range) => RenderBatch::Colored(range),
                Batch::ImageTriangles(name, range) => {
                    let asset_id = match self.images_map.get(&name) {
                        Some(asset_id) => *asset_id,
                        None => continue,
                    };
                    RenderBatch::Image(asset_id, range)
                }
                Batch::ExternalText(_, text) => {
                    let (asset_id, font) = match self
                        .fonts_map
                        .get(&text.font)
                        .and_then(|id| self.assets.asset_by_id(*id))
                        .and_then(|asset| asset.get::<FontAsset>())
                        // TODO: add support for multiple font pages, generating render batch per page.
                        .and_then(|font| Some((font.pages_image_assets.get(0)?, font)))
                    {
                        Some(((_, asset_id), font)) => (*asset_id, font),
                        None => continue,
                    };
                    let mut instance = HaTextInstance::default();
                    instance.set_content_lossy(build_content(&text.text));
                    instance.set_font(&text.font);
                    instance.set_size(text.size * font_scale);
                    instance.set_color(Rgba::new(
                        text.color.r,
                        text.color.g,
                        text.color.b,
                        text.color.a,
                    ));
                    instance.set_bounds_width(Some(text.box_size.x * scale.x));
                    instance.set_bounds_height(Some(text.box_size.y * scale.y));
                    instance.set_alignment(Vec2::new(
                        match text.horizontal_align {
                            TextBoxHorizontalAlign::Left => 0.0,
                            TextBoxHorizontalAlign::Center => 0.5,
                            TextBoxHorizontalAlign::Right => 1.0,
                        },
                        match text.vertical_align {
                            TextBoxVerticalAlign::Top => 0.0,
                            TextBoxVerticalAlign::Middle => 0.5,
                            TextBoxVerticalAlign::Bottom => 1.0,
                        },
                    ));
                    instance.set_wrapping(HaTextWrapping::Word);
                    let factory = match SurfaceTextFactory::factory::<V>(&instance, font) {
                        Ok(factory) => factory,
                        Err(_) => continue,
                    };
                    let from = stream.index_count();
                    if stream.write_from(&factory).is_err() {
                        continue;
                    }
                    let to = stream.index_count();
                    RenderBatch::Text(asset_id, Mat4::from_col_array(text.matrix), from..to)
                }
                _ => continue,
            });
        }

        Ok(stream)
    }
}

fn build_content(text: &str) -> HaTextContent {
    if let Some(text) = text.strip_prefix('~') {
        match HaRichTextContent::new(text) {
            Ok(content) => HaTextContent::Fragments(content.build_fragments()),
            Err(error) => HaTextContent::text(&error),
        }
    } else {
        HaTextContent::text(text)
    }
}

#[derive(Debug, Clone)]
pub enum RenderBatch {
    Colored(Range<usize>),
    Image(AssetId, Range<usize>),
    Text(AssetId, Mat4, Range<usize>),
}
