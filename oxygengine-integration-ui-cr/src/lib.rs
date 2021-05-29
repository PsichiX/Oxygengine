use oxygengine_composite_renderer::{
    component::{CompositeCamera, CompositeRenderable, CompositeTransform},
    composite_renderer::{
        Command, CompositeRenderer, Image, Mask, PathElement, Rectangle, Renderable, Text,
        TextAlign,
    },
    jpg_image_asset_protocol::JpgImageAsset,
    math::{Color, Mat2d, Rect, Vec2},
    png_image_asset_protocol::PngImageAsset,
    sprite_sheet_asset_protocol::SpriteSheetAsset,
    svg_image_asset_protocol::SvgImageAsset,
};
use oxygengine_core::{
    app::AppBuilder,
    assets::{asset::AssetId, database::AssetsDatabase},
    ecs::{
        Component, Join, Read, ReadExpect, ReadStorage, System, VecStorage, Write, WriteStorage,
    },
    hierarchy::Name,
    prefab::{Prefab, PrefabComponent, PrefabManager},
    Ignite, Scalar,
};
use oxygengine_user_interface::{
    component::UserInterfaceView,
    raui::core::{
        layout::{CoordsMapping, CoordsMappingScaling, Layout},
        renderer::Renderer,
        widget::{
            context::WidgetContext,
            node::WidgetNode,
            unit::{
                area::{AreaBoxNode, AreaBoxRendererEffect},
                image::{ImageBoxFrame, ImageBoxImageScaling, ImageBoxMaterial},
                text::{TextBoxFont, TextBoxHorizontalAlign},
                WidgetUnit,
            },
            utils::{lerp, Color as RauiColor, Rect as RauiRect, Transform, Vec2 as RauiVec2},
        },
    },
    resource::UserInterfaceRes,
    PropsData,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, marker::PhantomData};

pub mod prelude {
    pub use crate::*;
}

pub fn bundle_installer<CR>(builder: &mut AppBuilder, _phantom: PhantomData<CR>)
where
    CR: CompositeRenderer + 'static,
{
    builder.install_thread_local_system(ApplyUserInterfaceToCompositeRenderer::<CR>::default());
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory::<UserInterfaceViewSyncCompositeRenderable>(
        "UserInterfaceViewSyncCompositeRenderable",
    );
}

#[derive(Ignite, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserInterfaceApproxRectValues {
    Exact,
    Round,
    RoundDown,
    RoundUp,
    RoundInside,
    RoundOutside,
}

impl Default for UserInterfaceApproxRectValues {
    fn default() -> Self {
        Self::Exact
    }
}

impl UserInterfaceApproxRectValues {
    pub fn calculate(self, rect: Rect) -> Rect {
        match self {
            Self::Exact => rect,
            Self::Round => Rect {
                x: rect.x.round(),
                y: rect.y.round(),
                w: rect.w.round(),
                h: rect.h.round(),
            },
            Self::RoundDown => Rect {
                x: rect.x.floor(),
                y: rect.y.floor(),
                w: rect.w.floor(),
                h: rect.h.floor(),
            },
            Self::RoundUp => Rect {
                x: rect.x.ceil(),
                y: rect.y.ceil(),
                w: rect.w.ceil(),
                h: rect.h.ceil(),
            },
            Self::RoundInside => Rect {
                x: rect.x.ceil(),
                y: rect.y.ceil(),
                w: rect.w.floor(),
                h: rect.h.floor(),
            },
            Self::RoundOutside => Rect {
                x: rect.x.floor(),
                y: rect.y.floor(),
                w: rect.w.ceil(),
                h: rect.h.ceil(),
            },
        }
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct UserInterfaceViewSyncCompositeRenderable {
    #[serde(default)]
    pub camera_name: String,
    #[serde(default)]
    pub viewport: RauiRect,
    #[serde(default)]
    pub mapping_scaling: CoordsMappingScaling,
    #[serde(default)]
    pub approx_rect_values: UserInterfaceApproxRectValues,
    #[serde(default = "UserInterfaceViewSyncCompositeRenderable::default_image_smoothing")]
    pub image_smoothing: bool,
}

impl Default for UserInterfaceViewSyncCompositeRenderable {
    fn default() -> Self {
        Self {
            camera_name: Default::default(),
            viewport: Default::default(),
            mapping_scaling: Default::default(),
            approx_rect_values: Default::default(),
            image_smoothing: Self::default_image_smoothing(),
        }
    }
}

impl UserInterfaceViewSyncCompositeRenderable {
    fn default_image_smoothing() -> bool {
        true
    }
}

impl Component for UserInterfaceViewSyncCompositeRenderable {
    type Storage = VecStorage<Self>;
}

impl Prefab for UserInterfaceViewSyncCompositeRenderable {}
impl PrefabComponent for UserInterfaceViewSyncCompositeRenderable {}

pub struct ApplyUserInterfaceToCompositeRenderer<CR>
where
    CR: CompositeRenderer,
{
    images_cache: HashMap<String, String>,
    atlas_table: HashMap<AssetId, String>,
    frames_cache: HashMap<String, HashMap<String, Rect>>,
    images_sizes_cache: HashMap<String, Vec2>,
    images_sizes_table: HashMap<AssetId, String>,
    _phantom: PhantomData<CR>,
}

impl<CR> Default for ApplyUserInterfaceToCompositeRenderer<CR>
where
    CR: CompositeRenderer,
{
    fn default() -> Self {
        Self {
            images_cache: Default::default(),
            atlas_table: Default::default(),
            frames_cache: Default::default(),
            images_sizes_cache: Default::default(),
            images_sizes_table: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<'s, CR> System<'s> for ApplyUserInterfaceToCompositeRenderer<CR>
where
    CR: CompositeRenderer + Send + Sync + 'static,
{
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Option<Read<'s, CR>>,
        ReadExpect<'s, AssetsDatabase>,
        Write<'s, UserInterfaceRes>,
        WriteStorage<'s, CompositeRenderable>,
        ReadStorage<'s, UserInterfaceView>,
        ReadStorage<'s, UserInterfaceViewSyncCompositeRenderable>,
        ReadStorage<'s, CompositeCamera>,
        ReadStorage<'s, CompositeTransform>,
        ReadStorage<'s, Name>,
    );

    fn run(
        &mut self,
        (
            renderer,
            assets,
            mut ui,
            mut renderables,
            views,
            syncs,
            cameras,
            transforms,
            names,
        ): Self::SystemData,
    ) {
        if renderer.is_none() {
            return;
        }

        for id in assets.lately_loaded_protocol("atlas") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded atlas asset");
            let path = asset.path().to_owned();
            let asset = asset
                .get::<SpriteSheetAsset>()
                .expect("trying to use non-atlas asset");
            let image = asset.info().meta.image_name();
            let frames = asset
                .info()
                .frames
                .iter()
                .map(|(k, v)| (k.to_owned(), v.frame))
                .collect();
            self.images_cache.insert(path.clone(), image);
            self.atlas_table.insert(id, path.clone());
            self.frames_cache.insert(path, frames);
        }
        for id in assets.lately_unloaded_protocol("atlas") {
            if let Some(path) = self.atlas_table.remove(id) {
                self.images_cache.remove(&path);
                self.frames_cache.remove(&path);
            }
        }
        for id in assets.lately_loaded_protocol("png") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded png asset");
            let path = asset.path().to_owned();
            let asset = asset
                .get::<PngImageAsset>()
                .expect("trying to use non-png asset");
            let width = asset.width() as Scalar;
            let height = asset.height() as Scalar;
            self.images_sizes_cache
                .insert(path.clone(), Vec2::new(width, height));
            self.images_sizes_table.insert(id, path);
        }
        for id in assets.lately_unloaded_protocol("png") {
            if let Some(path) = self.images_sizes_table.remove(id) {
                self.images_sizes_cache.remove(&path);
            }
        }
        for id in assets.lately_loaded_protocol("jpg") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded jpg asset");
            let path = asset.path().to_owned();
            let asset = asset
                .get::<JpgImageAsset>()
                .expect("trying to use non-jpg asset");
            let width = asset.width() as Scalar;
            let height = asset.height() as Scalar;
            self.images_sizes_cache
                .insert(path.clone(), Vec2::new(width, height));
            self.images_sizes_table.insert(id, path);
        }
        for id in assets.lately_unloaded_protocol("jpg") {
            if let Some(path) = self.images_sizes_table.remove(id) {
                self.images_sizes_cache.remove(&path);
            }
        }
        for id in assets.lately_loaded_protocol("svg") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded svg asset");
            let path = asset.path().to_owned();
            let asset = asset
                .get::<SvgImageAsset>()
                .expect("trying to use non-svg asset");
            let width = asset.width() as Scalar;
            let height = asset.height() as Scalar;
            self.images_sizes_cache
                .insert(path.clone(), Vec2::new(width, height));
            self.images_sizes_table.insert(id, path);
        }
        for id in assets.lately_unloaded_protocol("svg") {
            if let Some(path) = self.images_sizes_table.remove(id) {
                self.images_sizes_cache.remove(&path);
            }
        }

        let renderer = renderer.unwrap();
        let view_size = renderer.view_size();

        for (renderable, view, sync) in (&mut renderables, &views, &syncs).join() {
            let mapping = (&cameras, &names, &transforms)
                .join()
                .find_map(|(c, n, t)| {
                    if sync.camera_name == n.0 {
                        if let Some(inv_mat) = !c.view_matrix(t, view_size) {
                            let size = view_size * inv_mat;
                            let rect = RauiRect {
                                left: lerp(0.0, size.x, sync.viewport.left),
                                right: lerp(size.x, 0.0, sync.viewport.right),
                                top: lerp(0.0, size.y, sync.viewport.top),
                                bottom: lerp(size.y, 0.0, sync.viewport.bottom),
                            };
                            Some(CoordsMapping::new_scaling(rect, sync.mapping_scaling))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });
            if let (Some(mapping), Some(data)) = (mapping, ui.get_mut(view.app_id())) {
                data.coords_mapping = mapping;
                let mut raui_renderer = RauiRenderer::new(
                    &self.images_cache,
                    &self.frames_cache,
                    &self.images_sizes_cache,
                    sync.approx_rect_values,
                    sync.image_smoothing,
                    32,
                );
                if let Ok(commands) = data
                    .application
                    .render(&data.coords_mapping, &mut raui_renderer)
                {
                    renderable.0 = Renderable::Commands(commands);
                }
            }
        }
    }
}

fn raui_to_vec2(v: RauiVec2) -> Vec2 {
    Vec2::new(v.x, v.y)
}

fn raui_to_rect(v: RauiRect) -> Rect {
    Rect::new(Vec2::new(v.left, v.top), Vec2::new(v.width(), v.height()))
}

fn raui_to_color(v: RauiColor) -> Color {
    Color::rgba(
        (v.r * 255.0) as u8,
        (v.g * 255.0) as u8,
        (v.b * 255.0) as u8,
        (v.a * 255.0) as u8,
    )
}

#[derive(Debug, Copy, Clone)]
enum ImageFrame {
    None,
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl ImageFrame {
    fn source(self, rect: Rect, image_frame: Option<&ImageBoxFrame>) -> Rect {
        if let Some(image_frame) = image_frame {
            let result = match self {
                ImageFrame::None => rect,
                ImageFrame::TopLeft => Rect::new(
                    Vec2::new(rect.x, rect.y),
                    Vec2::new(image_frame.source.left, image_frame.source.top),
                ),
                ImageFrame::TopCenter => Rect::new(
                    Vec2::new(rect.x + image_frame.source.left, rect.y),
                    Vec2::new(
                        rect.w - image_frame.source.left - image_frame.source.right,
                        image_frame.source.top,
                    ),
                ),
                ImageFrame::TopRight => Rect::new(
                    Vec2::new(rect.x + rect.w - image_frame.destination.right, rect.y),
                    Vec2::new(image_frame.source.right, image_frame.source.top),
                ),
                ImageFrame::MiddleLeft => Rect::new(
                    Vec2::new(rect.x, rect.y + image_frame.source.top),
                    Vec2::new(
                        image_frame.source.left,
                        rect.h - image_frame.source.top - image_frame.source.bottom,
                    ),
                ),
                ImageFrame::MiddleCenter => Rect::new(
                    Vec2::new(
                        rect.x + image_frame.source.left,
                        rect.y + image_frame.source.top,
                    ),
                    Vec2::new(
                        rect.w - image_frame.source.left - image_frame.source.right,
                        rect.h - image_frame.source.top - image_frame.source.bottom,
                    ),
                ),
                ImageFrame::MiddleRight => Rect::new(
                    Vec2::new(
                        rect.x + rect.w - image_frame.source.right,
                        rect.y + image_frame.source.top,
                    ),
                    Vec2::new(
                        image_frame.source.right,
                        rect.h - image_frame.source.top - image_frame.source.bottom,
                    ),
                ),
                ImageFrame::BottomLeft => Rect::new(
                    Vec2::new(rect.x, rect.y + rect.h - image_frame.source.bottom),
                    Vec2::new(image_frame.source.left, image_frame.source.bottom),
                ),
                ImageFrame::BottomCenter => Rect::new(
                    Vec2::new(
                        rect.x + image_frame.source.left,
                        rect.y + rect.h - image_frame.source.bottom,
                    ),
                    Vec2::new(
                        rect.w - image_frame.source.left - image_frame.source.right,
                        image_frame.source.bottom,
                    ),
                ),
                ImageFrame::BottomRight => Rect::new(
                    Vec2::new(
                        rect.x + rect.w - image_frame.source.right,
                        rect.y + rect.h - image_frame.source.bottom,
                    ),
                    Vec2::new(image_frame.source.right, image_frame.source.bottom),
                ),
            };
            if result.w >= 0.0 && result.h >= 0.0 {
                result
            } else {
                Rect::default()
            }
        } else {
            match self {
                ImageFrame::None | ImageFrame::MiddleCenter => rect,
                _ => Rect::default(),
            }
        }
    }

    fn destination(
        self,
        rect: RauiRect,
        image_frame: Option<&ImageBoxFrame>,
        source_size: Option<Vec2>,
    ) -> Rect {
        if let Some(image_frame) = image_frame {
            let mut d = image_frame.destination;
            if image_frame.frame_keep_aspect_ratio {
                if let Some(source_size) = source_size {
                    d.left = (image_frame.source.left * rect.height()) / source_size.y;
                    d.right = (image_frame.source.right * rect.height()) / source_size.y;
                    d.top = (image_frame.source.top * rect.width()) / source_size.x;
                    d.bottom = (image_frame.source.bottom * rect.width()) / source_size.x;
                }
            }
            if d.left + d.right > rect.width() {
                let m = d.left + d.right;
                d.left = rect.width() * d.left / m;
                d.right = rect.width() * d.right / m;
            }
            if d.top + d.bottom > rect.height() {
                let m = d.top + d.bottom;
                d.top = rect.height() * d.top / m;
                d.bottom = rect.height() * d.bottom / m;
            }
            let result = match self {
                ImageFrame::None => raui_to_rect(rect),
                ImageFrame::TopLeft => {
                    Rect::new(Vec2::new(rect.left, rect.top), Vec2::new(d.left, d.top))
                }
                ImageFrame::TopCenter => Rect::new(
                    Vec2::new(rect.left + d.left, rect.top),
                    Vec2::new(rect.width() - d.left - d.right, d.top),
                ),
                ImageFrame::TopRight => Rect::new(
                    Vec2::new(rect.right - d.right, rect.top),
                    Vec2::new(d.right, d.top),
                ),
                ImageFrame::MiddleLeft => Rect::new(
                    Vec2::new(rect.left, rect.top + d.top),
                    Vec2::new(d.left, rect.height() - d.top - d.bottom),
                ),
                ImageFrame::MiddleCenter => Rect::new(
                    Vec2::new(rect.left + d.left, rect.top + d.top),
                    Vec2::new(
                        rect.width() - d.left - d.right,
                        rect.height() - d.top - d.bottom,
                    ),
                ),
                ImageFrame::MiddleRight => Rect::new(
                    Vec2::new(rect.right - d.right, rect.top + d.top),
                    Vec2::new(d.right, rect.height() - d.top - d.bottom),
                ),
                ImageFrame::BottomLeft => Rect::new(
                    Vec2::new(rect.left, rect.bottom - d.bottom),
                    Vec2::new(d.left, d.bottom),
                ),
                ImageFrame::BottomCenter => Rect::new(
                    Vec2::new(rect.left + d.left, rect.bottom - d.bottom),
                    Vec2::new(rect.width() - d.left - d.right, d.bottom),
                ),
                ImageFrame::BottomRight => Rect::new(
                    Vec2::new(rect.right - d.right, rect.bottom - d.bottom),
                    Vec2::new(d.right, d.bottom),
                ),
            };
            if result.w >= 0.0 && result.h >= 0.0 {
                result
            } else {
                Rect::default()
            }
        } else {
            match self {
                ImageFrame::None | ImageFrame::MiddleCenter => raui_to_rect(rect),
                _ => Rect::default(),
            }
        }
    }
}

enum Filter {
    None,
    Reset,
    Replace([Scalar; 8]),
    Combine([Scalar; 8]),
}

struct RauiRenderer<'a> {
    images: &'a HashMap<String, String>,
    frames: &'a HashMap<String, HashMap<String, Rect>>,
    images_sizes: &'a HashMap<String, Vec2>,
    approx_rect_values: UserInterfaceApproxRectValues,
    image_smoothing: bool,
    filter_stack: Vec<[Scalar; 8]>,
}

impl<'a> RauiRenderer<'a> {
    pub fn new(
        images: &'a HashMap<String, String>,
        frames: &'a HashMap<String, HashMap<String, Rect>>,
        images_sizes: &'a HashMap<String, Vec2>,
        approx_rect_values: UserInterfaceApproxRectValues,
        image_smoothing: bool,
        filter_stack_capacity: usize,
    ) -> Self {
        let mut filter_stack = Vec::with_capacity(filter_stack_capacity);
        filter_stack.push([0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
        Self {
            images,
            frames,
            images_sizes,
            approx_rect_values,
            image_smoothing,
            filter_stack,
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn make_transform_command(transform: &Transform, rect: RauiRect) -> Command<'static> {
        let size = rect.size();
        let offset = Vec2::new(rect.left, rect.top);
        let offset = Mat2d::translation(offset);
        let pivot = Vec2::new(
            lerp(0.0, size.x, transform.pivot.x),
            lerp(0.0, size.y, transform.pivot.y),
        );
        let pivot = Mat2d::translation(pivot);
        let inv_pivot = pivot.inverse().unwrap_or_default();
        let align = Vec2::new(
            lerp(0.0, size.x, transform.align.x),
            lerp(0.0, size.y, transform.align.y),
        );
        let align = Mat2d::translation(align);
        let translate = Mat2d::translation(raui_to_vec2(transform.translation));
        let rotate = Mat2d::rotation(transform.rotation);
        let scale = Mat2d::scale(raui_to_vec2(transform.scale));
        let skew = Mat2d::skew(raui_to_vec2(transform.skew));
        let matrix = pivot * translate * rotate * scale * skew * inv_pivot * align * offset;
        let [a, b, c, d, e, f] = matrix.0;
        Command::Transform(a, b, c, d, e, f)
    }

    #[allow(clippy::many_single_char_names)]
    fn make_simple_transform_command(rect: RauiRect) -> Command<'static> {
        let offset = Vec2::new(rect.left, rect.top);
        let offset = Mat2d::translation(offset);
        let [a, b, c, d, e, f] = offset.0;
        Command::Transform(a, b, c, d, e, f)
    }

    fn make_area_effect(effect: &Option<AreaBoxRendererEffect>) -> Filter {
        if let Some(effect) = effect {
            match effect.id.as_str() {
                "filter-reset" => Filter::Reset,
                "filter-replace" => Filter::Replace(effect.params),
                "filter-combine" => Filter::Combine(effect.params),
                _ => Filter::None,
            }
        } else {
            Filter::None
        }
    }

    fn make_rect_renderable(
        &self,
        color: RauiColor,
        rect: RauiRect,
        image_frame: Option<&ImageBoxFrame>,
        subframe: ImageFrame,
    ) -> Rectangle {
        Rectangle {
            color: raui_to_color(color),
            rect: self
                .approx_rect_values
                .calculate(subframe.destination(rect, image_frame, None)),
        }
    }

    fn make_image_renderable(
        &self,
        image: &str,
        image_source: Option<&RauiRect>,
        rect: RauiRect,
        image_frame: Option<&ImageBoxFrame>,
        subframe: ImageFrame,
    ) -> Image<'static> {
        let mut it = image.split('@');
        if let Some(sheet) = it.next() {
            if let Some(frame) = it.next() {
                if let Some(name) = self.images.get(sheet) {
                    if let Some(frames) = self.frames.get(sheet) {
                        let srect = match image_source {
                            Some(rect) => raui_to_rect(*rect),
                            None => frames.get(frame).copied().unwrap_or_default(),
                        };
                        let destination = self.approx_rect_values.calculate(subframe.destination(
                            rect,
                            image_frame,
                            Some(srect.size()),
                        ));
                        return Image {
                            image: name.to_owned().into(),
                            source: Some(subframe.source(srect, image_frame)),
                            destination: Some(destination),
                            alignment: Vec2::zero(),
                        };
                    }
                }
            }
        }
        let frame = match image_source {
            Some(rect) => raui_to_rect(*rect),
            None => self
                .images_sizes
                .get(image)
                .copied()
                .map(Rect::with_size)
                .unwrap_or_default(),
        };
        let destination = self.approx_rect_values.calculate(subframe.destination(
            rect,
            image_frame,
            Some(frame.size()),
        ));
        Image {
            image: image.to_owned().into(),
            source: Some(subframe.source(frame, image_frame)),
            destination: Some(destination),
            alignment: Vec2::zero(),
        }
    }

    fn make_text_renderable(
        text: &str,
        font: &TextBoxFont,
        rect: RauiRect,
        horizontal_align: TextBoxHorizontalAlign,
        color: RauiColor,
    ) -> Command<'static> {
        let (align, position) = match horizontal_align {
            TextBoxHorizontalAlign::Left => (TextAlign::Left, Vec2::new(rect.left, rect.top)),
            TextBoxHorizontalAlign::Center => (
                TextAlign::Center,
                Vec2::new(rect.left + rect.width() * 0.5, rect.top),
            ),
            TextBoxHorizontalAlign::Right => (TextAlign::Right, Vec2::new(rect.right, rect.top)),
        };
        Command::Draw(Renderable::Text(Text {
            color: raui_to_color(color),
            font: font.name.to_owned().into(),
            align,
            text: text.to_owned().into(),
            position,
            size: font.size,
            max_width: Some(rect.width()),
            ..Default::default()
        }))
    }

    fn image_size(&self, image: &str) -> Vec2 {
        let mut it = image.split('@');
        if let Some(sheet) = it.next() {
            if let Some(frame) = it.next() {
                if let Some(frames) = self.frames.get(sheet) {
                    if let Some(rect) = frames.get(frame) {
                        return Vec2::new(rect.w, rect.h);
                    }
                }
            }
        }
        if let Some(rect) = self.images_sizes.get(image).copied().map(Rect::with_size) {
            return Vec2::new(rect.w, rect.h);
        }
        Vec2::zero()
    }

    // TODO: refactor this shit!
    fn render_node(
        &mut self,
        unit: &WidgetUnit,
        mapping: &CoordsMapping,
        layout: &Layout,
        local: bool,
    ) -> Vec<Command<'static>> {
        match unit {
            WidgetUnit::None | WidgetUnit::PortalBox(_) => vec![],
            WidgetUnit::AreaBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    let transform = Self::make_simple_transform_command(local_space);
                    let area_effect = match Self::make_area_effect(&unit.renderer_effect) {
                        Filter::None => self.filter_stack.last().copied().unwrap(),
                        Filter::Reset => [0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                        Filter::Replace(data) => data,
                        Filter::Combine(data) => {
                            let old = self.filter_stack.last().copied().unwrap();
                            [
                                old[0] + data[0],
                                old[1] + data[1],
                                old[2] + data[2],
                                old[3] + data[3],
                                old[4] + data[4],
                                old[5] + data[5],
                                old[6] + data[6],
                                old[7] + data[7],
                            ]
                        }
                    };
                    self.filter_stack.push(area_effect);
                    let area_effect = format!(
                        "blur({}px) brightness({}%) contrast({}%) grayscale({}%) invert({}%) saturate({}%) sepia({}%) hue-rotate({}deg)",
                        area_effect[0],
                        area_effect[1] * 100.0,
                        area_effect[2] * 100.0,
                        area_effect[3] * 100.0,
                        area_effect[4] * 100.0,
                        area_effect[5] * 100.0,
                        area_effect[6] * 100.0,
                        area_effect[7],
                    );
                    let children = self.render_node(&unit.slot, mapping, layout, true);
                    self.filter_stack.pop();
                    std::iter::once(Command::Store)
                        .chain(std::iter::once(transform))
                        .chain(std::iter::once(Command::Filter(area_effect.into())))
                        .chain(children)
                        .chain(std::iter::once(Command::Restore))
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                }
            }
            WidgetUnit::ContentBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let mut items = unit
                        .items
                        .iter()
                        .map(|item| (item.layout.depth, item))
                        .collect::<Vec<_>>();
                    items.sort_by(|(a, _), (b, _)| a.partial_cmp(&b).unwrap());
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    let transform = Self::make_transform_command(&unit.transform, local_space);
                    let mask = if unit.clipping {
                        Command::Draw(Renderable::Mask(Mask {
                            elements: vec![PathElement::Rectangle(Rect::with_size(Vec2::new(
                                local_space.width(),
                                local_space.height(),
                            )))],
                        }))
                    } else {
                        Command::None
                    };
                    std::iter::once(Command::Store)
                        .chain(std::iter::once(transform))
                        .chain(std::iter::once(mask))
                        .chain(items.into_iter().flat_map(|(_, item)| {
                            self.render_node(&item.slot, mapping, layout, true)
                        }))
                        .chain(std::iter::once(Command::Restore))
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                }
            }
            WidgetUnit::FlexBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    let transform = Self::make_transform_command(&unit.transform, local_space);
                    std::iter::once(Command::Store)
                        .chain(std::iter::once(transform))
                        .chain(
                            unit.items.iter().flat_map(|item| {
                                self.render_node(&item.slot, mapping, layout, true)
                            }),
                        )
                        .chain(std::iter::once(Command::Restore))
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                }
            }
            WidgetUnit::GridBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    let transform = Self::make_transform_command(&unit.transform, local_space);
                    std::iter::once(Command::Store)
                        .chain(std::iter::once(transform))
                        .chain(
                            unit.items.iter().flat_map(|item| {
                                self.render_node(&item.slot, mapping, layout, true)
                            }),
                        )
                        .chain(std::iter::once(Command::Restore))
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                }
            }
            WidgetUnit::SizeBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    let transform = Self::make_transform_command(&unit.transform, local_space);
                    std::iter::once(Command::Store)
                        .chain(std::iter::once(transform))
                        .chain(self.render_node(&unit.slot, mapping, layout, true))
                        .chain(std::iter::once(Command::Restore))
                        .collect::<Vec<_>>()
                } else {
                    vec![]
                }
            }
            WidgetUnit::ImageBox(unit) => match &unit.material {
                ImageBoxMaterial::Color(image) => {
                    if let Some(item) = layout.items.get(&unit.id) {
                        let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                        let transform = Self::make_transform_command(&unit.transform, local_space);
                        let rect = RauiRect {
                            left: 0.0,
                            right: local_space.width(),
                            top: 0.0,
                            bottom: local_space.height(),
                        };
                        match &image.scaling {
                            ImageBoxImageScaling::Stretch => {
                                let renderable = self.make_rect_renderable(
                                    image.color,
                                    rect,
                                    None,
                                    ImageFrame::None,
                                );
                                vec![
                                    Command::Store,
                                    Command::Smoothing(self.image_smoothing),
                                    transform,
                                    Command::Draw(Renderable::Rectangle(renderable)),
                                    Command::Restore,
                                ]
                            }
                            ImageBoxImageScaling::Frame(frame) => {
                                let renderable_top_left = self.make_rect_renderable(
                                    image.color,
                                    rect,
                                    Some(frame),
                                    ImageFrame::TopLeft,
                                );
                                let renderable_top_center = self.make_rect_renderable(
                                    image.color,
                                    rect,
                                    Some(frame),
                                    ImageFrame::TopCenter,
                                );
                                let renderable_top_right = self.make_rect_renderable(
                                    image.color,
                                    rect,
                                    Some(frame),
                                    ImageFrame::TopRight,
                                );
                                let renderable_middle_left = self.make_rect_renderable(
                                    image.color,
                                    rect,
                                    Some(frame),
                                    ImageFrame::MiddleLeft,
                                );
                                let renderable_middle_right = self.make_rect_renderable(
                                    image.color,
                                    rect,
                                    Some(frame),
                                    ImageFrame::MiddleRight,
                                );
                                let renderable_bottom_left = self.make_rect_renderable(
                                    image.color,
                                    rect,
                                    Some(frame),
                                    ImageFrame::BottomLeft,
                                );
                                let renderable_bottom_center = self.make_rect_renderable(
                                    image.color,
                                    rect,
                                    Some(frame),
                                    ImageFrame::BottomCenter,
                                );
                                let renderable_bottom_right = self.make_rect_renderable(
                                    image.color,
                                    rect,
                                    Some(frame),
                                    ImageFrame::BottomRight,
                                );
                                if frame.frame_only {
                                    vec![
                                        Command::Store,
                                        Command::Smoothing(self.image_smoothing),
                                        transform,
                                        Command::Draw(Renderable::Rectangle(renderable_top_left)),
                                        Command::Draw(Renderable::Rectangle(renderable_top_center)),
                                        Command::Draw(Renderable::Rectangle(renderable_top_right)),
                                        Command::Draw(Renderable::Rectangle(
                                            renderable_middle_left,
                                        )),
                                        Command::Draw(Renderable::Rectangle(
                                            renderable_middle_right,
                                        )),
                                        Command::Draw(Renderable::Rectangle(
                                            renderable_bottom_left,
                                        )),
                                        Command::Draw(Renderable::Rectangle(
                                            renderable_bottom_center,
                                        )),
                                        Command::Draw(Renderable::Rectangle(
                                            renderable_bottom_right,
                                        )),
                                        Command::Restore,
                                    ]
                                } else {
                                    let renderable_middle_center = self.make_rect_renderable(
                                        image.color,
                                        rect,
                                        Some(frame),
                                        ImageFrame::MiddleCenter,
                                    );
                                    vec![
                                        Command::Store,
                                        Command::Smoothing(self.image_smoothing),
                                        transform,
                                        Command::Draw(Renderable::Rectangle(renderable_top_left)),
                                        Command::Draw(Renderable::Rectangle(renderable_top_center)),
                                        Command::Draw(Renderable::Rectangle(renderable_top_right)),
                                        Command::Draw(Renderable::Rectangle(
                                            renderable_middle_left,
                                        )),
                                        Command::Draw(Renderable::Rectangle(
                                            renderable_middle_center,
                                        )),
                                        Command::Draw(Renderable::Rectangle(
                                            renderable_middle_right,
                                        )),
                                        Command::Draw(Renderable::Rectangle(
                                            renderable_bottom_left,
                                        )),
                                        Command::Draw(Renderable::Rectangle(
                                            renderable_bottom_center,
                                        )),
                                        Command::Draw(Renderable::Rectangle(
                                            renderable_bottom_right,
                                        )),
                                        Command::Restore,
                                    ]
                                }
                            }
                        }
                    } else {
                        vec![]
                    }
                }
                ImageBoxMaterial::Image(image) => {
                    if let Some(item) = layout.items.get(&unit.id) {
                        let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                        let transform = Self::make_transform_command(&unit.transform, local_space);
                        let rect = RauiRect {
                            left: 0.0,
                            right: local_space.width(),
                            top: 0.0,
                            bottom: local_space.height(),
                        };
                        let alpha = Command::Alpha(image.tint.a);
                        let rect = if let Some(aspect) = unit.content_keep_aspect_ratio {
                            let size = self.image_size(&image.id);
                            let ox = rect.left;
                            let oy = rect.top;
                            let iw = rect.width();
                            let ih = rect.height();
                            let ra = size.x / size.y;
                            let ia = iw / ih;
                            let scale = if ra >= ia { iw / size.x } else { ih / size.y };
                            let w = size.x * scale;
                            let h = size.y * scale;
                            let ow = lerp(0.0, iw - w, aspect.horizontal_alignment);
                            let oh = lerp(0.0, ih - h, aspect.vertical_alignment);
                            RauiRect {
                                left: ox + ow,
                                right: ox + ow + w,
                                top: oy + oh,
                                bottom: oy + oh + h,
                            }
                        } else {
                            rect
                        };
                        match &image.scaling {
                            ImageBoxImageScaling::Stretch => {
                                let renderable = self.make_image_renderable(
                                    &image.id,
                                    image.source_rect.as_ref(),
                                    rect,
                                    None,
                                    ImageFrame::None,
                                );
                                vec![
                                    Command::Store,
                                    Command::Smoothing(self.image_smoothing),
                                    transform,
                                    alpha,
                                    Command::Draw(Renderable::Image(renderable)),
                                    Command::Restore,
                                ]
                            }
                            ImageBoxImageScaling::Frame(frame) => {
                                let renderable_top_left = self.make_image_renderable(
                                    &image.id,
                                    image.source_rect.as_ref(),
                                    rect,
                                    Some(frame),
                                    ImageFrame::TopLeft,
                                );
                                let renderable_top_center = self.make_image_renderable(
                                    &image.id,
                                    image.source_rect.as_ref(),
                                    rect,
                                    Some(frame),
                                    ImageFrame::TopCenter,
                                );
                                let renderable_top_right = self.make_image_renderable(
                                    &image.id,
                                    image.source_rect.as_ref(),
                                    rect,
                                    Some(frame),
                                    ImageFrame::TopRight,
                                );
                                let renderable_middle_left = self.make_image_renderable(
                                    &image.id,
                                    image.source_rect.as_ref(),
                                    rect,
                                    Some(frame),
                                    ImageFrame::MiddleLeft,
                                );
                                let renderable_middle_right = self.make_image_renderable(
                                    &image.id,
                                    image.source_rect.as_ref(),
                                    rect,
                                    Some(frame),
                                    ImageFrame::MiddleRight,
                                );
                                let renderable_bottom_left = self.make_image_renderable(
                                    &image.id,
                                    image.source_rect.as_ref(),
                                    rect,
                                    Some(frame),
                                    ImageFrame::BottomLeft,
                                );
                                let renderable_bottom_center = self.make_image_renderable(
                                    &image.id,
                                    image.source_rect.as_ref(),
                                    rect,
                                    Some(frame),
                                    ImageFrame::BottomCenter,
                                );
                                let renderable_bottom_right = self.make_image_renderable(
                                    &image.id,
                                    image.source_rect.as_ref(),
                                    rect,
                                    Some(frame),
                                    ImageFrame::BottomRight,
                                );
                                if frame.frame_only {
                                    vec![
                                        Command::Store,
                                        Command::Smoothing(self.image_smoothing),
                                        transform,
                                        alpha,
                                        Command::Draw(Renderable::Image(renderable_top_left)),
                                        Command::Draw(Renderable::Image(renderable_top_center)),
                                        Command::Draw(Renderable::Image(renderable_top_right)),
                                        Command::Draw(Renderable::Image(renderable_middle_left)),
                                        Command::Draw(Renderable::Image(renderable_middle_right)),
                                        Command::Draw(Renderable::Image(renderable_bottom_left)),
                                        Command::Draw(Renderable::Image(renderable_bottom_center)),
                                        Command::Draw(Renderable::Image(renderable_bottom_right)),
                                        Command::Restore,
                                    ]
                                } else {
                                    let renderable_middle_center = self.make_image_renderable(
                                        &image.id,
                                        image.source_rect.as_ref(),
                                        rect,
                                        Some(frame),
                                        ImageFrame::MiddleCenter,
                                    );
                                    vec![
                                        Command::Store,
                                        Command::Smoothing(self.image_smoothing),
                                        transform,
                                        alpha,
                                        Command::Draw(Renderable::Image(renderable_top_left)),
                                        Command::Draw(Renderable::Image(renderable_top_center)),
                                        Command::Draw(Renderable::Image(renderable_top_right)),
                                        Command::Draw(Renderable::Image(renderable_middle_left)),
                                        Command::Draw(Renderable::Image(renderable_middle_center)),
                                        Command::Draw(Renderable::Image(renderable_middle_right)),
                                        Command::Draw(Renderable::Image(renderable_bottom_left)),
                                        Command::Draw(Renderable::Image(renderable_bottom_center)),
                                        Command::Draw(Renderable::Image(renderable_bottom_right)),
                                        Command::Restore,
                                    ]
                                }
                            }
                        }
                    } else {
                        vec![]
                    }
                }
                ImageBoxMaterial::Procedural(_) => vec![],
            },
            WidgetUnit::TextBox(unit) => {
                if let Some(item) = layout.items.get(&unit.id) {
                    let local_space = mapping.virtual_to_real_rect(item.local_space, local);
                    let transform = Self::make_transform_command(&unit.transform, local_space);
                    let rect = RauiRect {
                        left: 0.0,
                        right: local_space.width(),
                        top: 0.0,
                        bottom: local_space.height(),
                    };
                    let mut font = unit.font.clone();
                    font.size *= mapping.scale().x.max(mapping.scale().y);
                    let renderable = Self::make_text_renderable(
                        &unit.text,
                        &font,
                        rect,
                        unit.horizontal_align,
                        unit.color,
                    );
                    vec![Command::Store, transform, renderable, Command::Restore]
                } else {
                    vec![]
                }
            }
        }
    }
}

impl<'a> Renderer<Vec<Command<'static>>, ()> for RauiRenderer<'a> {
    fn render(
        &mut self,
        tree: &WidgetUnit,
        mapping: &CoordsMapping,
        layout: &Layout,
    ) -> Result<Vec<Command<'static>>, ()> {
        Ok(self.render_node(tree, mapping, layout, false))
    }
}

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
pub enum FilterBoxProps {
    None,
    Reset,
    Replace(FilterBoxValues),
    Combine(FilterBoxValues),
}

impl Default for FilterBoxProps {
    fn default() -> Self {
        Self::None
    }
}

impl FilterBoxProps {
    pub fn effect(&self) -> Option<AreaBoxRendererEffect> {
        match self {
            Self::None => None,
            Self::Reset => Some(AreaBoxRendererEffect {
                id: "filter-reset".to_owned(),
                ..Default::default()
            }),
            Self::Replace(values) => Some(AreaBoxRendererEffect {
                id: "filter-replace".to_owned(),
                params: values.params(),
            }),
            Self::Combine(values) => Some(AreaBoxRendererEffect {
                id: "filter-combine".to_owned(),
                params: values.params(),
            }),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FilterBoxValues {
    #[serde(default)]
    pub blur: Scalar,
    #[serde(default)]
    pub brightness: Scalar,
    #[serde(default)]
    pub contrast: Scalar,
    #[serde(default)]
    pub grayscale: Scalar,
    #[serde(default)]
    pub invert: Scalar,
    #[serde(default)]
    pub saturate: Scalar,
    #[serde(default)]
    pub sepia: Scalar,
    #[serde(default)]
    pub hue_rotate: Scalar,
}

impl FilterBoxValues {
    pub fn params(&self) -> [Scalar; 8] {
        [
            self.blur,
            self.brightness,
            self.contrast,
            self.grayscale,
            self.invert,
            self.saturate,
            self.sepia,
            self.hue_rotate,
        ]
    }
}

pub fn filter_box(mut context: WidgetContext) -> WidgetNode {
    let slot = context.named_slots.remove("content").unwrap();
    let renderer_effect = context
        .props
        .read_cloned_or_default::<FilterBoxProps>()
        .effect();

    AreaBoxNode {
        id: context.id.to_owned(),
        slot: Box::new(slot),
        renderer_effect,
    }
    .into()
}
