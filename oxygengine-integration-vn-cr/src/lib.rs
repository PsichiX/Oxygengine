use oxygengine_composite_renderer::{
    component::{
        CompositeCamera, CompositeCameraAlignment, CompositeRenderAlpha, CompositeRenderLayer,
        CompositeRenderable, CompositeScalingMode, CompositeScalingTarget, CompositeTransform,
        CompositeUiElement, CompositeVisibility, UiElementType, UiImage, UiMargin,
    },
    composite_renderer::{Command, Image, Renderable, Text},
    math::{Color, Mat2d, Vec2},
    resource::{CompositeCameraCache, CompositeUiInteractibles},
};
use oxygengine_core::{
    app::AppBuilder,
    assets::{
        asset::AssetID,
        database::AssetsDatabase,
        protocol::{AssetLoadResult, AssetProtocol},
    },
    ecs::{
        world::{Builder, EntitiesRes},
        Component, Entity, Join, LazyUpdate, Read, ReadExpect, System, VecStorage, Write,
        WriteStorage,
    },
    hierarchy::{Name, Tag},
    prefab::{Prefab, PrefabError, PrefabManager, PrefabProxy},
    state::StateToken,
    Ignite, Scalar,
};
use oxygengine_input::resource::InputController;
use oxygengine_visual_novel::resource::VnStoryManager;
use serde::{Deserialize, Serialize};
#[cfg(not(feature = "scalar64"))]
use std::f32::consts::PI;
#[cfg(feature = "scalar64")]
use std::f64::consts::PI;
use std::{collections::HashMap, str::from_utf8};

pub mod prelude {
    pub use crate::*;
}

pub fn bundle_installer<'a, 'b>(builder: &mut AppBuilder<'a, 'b>, _: ()) {
    builder.install_resource(VnRenderingManager::default());
    builder.install_system(
        ApplyVisualNovelToCompositeRenderer::default(),
        "apply-visual-novel-to-composite-renderer",
        &["vn-story"],
    );
}

pub fn protocols_installer(database: &mut AssetsDatabase) {
    database.register(VnRenderingConfigAssetProtocol);
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs.register_component_factory_proxy::<PositionCameraAlignment, PositionCameraAlignmentPrefabProxy>("PositionCameraAlignment");
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum VnRenderingOverlayStyle {
    Color(Color),
    Image(String),
}

impl Default for VnRenderingOverlayStyle {
    fn default() -> Self {
        Self::Color(Color::black())
    }
}

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub struct VnRenderingConfig {
    #[serde(default = "VnRenderingConfig::default_background_camera_scaling_target")]
    pub background_camera_scaling_target: CompositeScalingTarget,
    #[serde(default = "VnRenderingConfig::default_background_camera_resolution")]
    pub background_camera_resolution: Scalar,
    #[serde(default = "VnRenderingConfig::default_background_camera_layer")]
    pub background_camera_layer: usize,
    #[serde(default = "VnRenderingConfig::default_characters_camera_scaling_target")]
    pub characters_camera_scaling_target: CompositeScalingTarget,
    #[serde(default = "VnRenderingConfig::default_characters_camera_resolution")]
    pub characters_camera_resolution: Scalar,
    #[serde(default = "VnRenderingConfig::default_characters_camera_layer")]
    pub characters_camera_layer: usize,
    #[serde(default = "VnRenderingConfig::default_ui_camera_scaling_target")]
    pub ui_camera_scaling_target: CompositeScalingTarget,
    #[serde(default = "VnRenderingConfig::default_ui_camera_resolution")]
    pub ui_camera_resolution: Scalar,
    #[serde(default = "VnRenderingConfig::default_ui_camera_layer")]
    pub ui_camera_layer: usize,
    #[serde(default)]
    pub overlay_style: VnRenderingOverlayStyle,
    #[serde(default)]
    pub hide_on_story_pause: bool,
    #[serde(default)]
    pub ui_component_template: Option<CompositeUiElement>,
    #[serde(default)]
    pub ui_dialogue_option_component_template: Option<CompositeUiElement>,
    #[serde(default = "VnRenderingConfig::default_ui_dialogue_default_theme")]
    pub ui_dialogue_default_theme: Option<String>,
    #[serde(default = "VnRenderingConfig::default_ui_dialogue_panel_path")]
    pub ui_dialogue_panel_path: String,
    #[serde(default = "VnRenderingConfig::default_ui_dialogue_text_path")]
    pub ui_dialogue_text_path: String,
    #[serde(default = "VnRenderingConfig::default_ui_dialogue_name_path")]
    pub ui_dialogue_name_path: String,
    #[serde(default = "VnRenderingConfig::default_ui_dialogue_skip_path")]
    pub ui_dialogue_skip_path: String,
    #[serde(default = "VnRenderingConfig::default_ui_dialogue_options_path")]
    pub ui_dialogue_options_path: String,
    #[serde(default = "VnRenderingConfig::default_ui_dialogue_option_text_path")]
    pub ui_dialogue_option_text_path: String,
    #[serde(default = "VnRenderingConfig::default_ui_dialogue_option_button_path")]
    pub ui_dialogue_option_button_path: String,
    #[serde(default = "VnRenderingConfig::default_ui_dialogue_default_name_color")]
    pub ui_dialogue_default_name_color: Color,
    #[serde(default = "VnRenderingConfig::default_input_pointer_trigger")]
    pub input_pointer_trigger: String,
    #[serde(default = "VnRenderingConfig::default_input_pointer_axis_x")]
    pub input_pointer_axis_x: String,
    #[serde(default = "VnRenderingConfig::default_input_pointer_axis_y")]
    pub input_pointer_axis_y: String,
    #[serde(default)]
    pub pointer_image: Option<String>,
    #[serde(default)]
    pub pointer_align: Vec2,
    #[serde(default = "VnRenderingConfig::default_forced_constant_refresh")]
    pub forced_constant_refresh: bool,
}

impl Default for VnRenderingConfig {
    fn default() -> Self {
        Self {
            background_camera_scaling_target: Self::default_background_camera_scaling_target(),
            background_camera_resolution: 1080.0,
            background_camera_layer: 1000,
            characters_camera_scaling_target: Self::default_characters_camera_scaling_target(),
            characters_camera_resolution: Self::default_characters_camera_resolution(),
            characters_camera_layer: Self::default_characters_camera_layer(),
            ui_camera_scaling_target: Self::default_ui_camera_scaling_target(),
            ui_camera_resolution: Self::default_ui_camera_resolution(),
            ui_camera_layer: Self::default_ui_camera_layer(),
            overlay_style: Default::default(),
            hide_on_story_pause: false,
            ui_component_template: None,
            ui_dialogue_option_component_template: None,
            ui_dialogue_default_theme: Self::default_ui_dialogue_default_theme(),
            ui_dialogue_panel_path: Self::default_ui_dialogue_panel_path(),
            ui_dialogue_text_path: Self::default_ui_dialogue_text_path(),
            ui_dialogue_name_path: Self::default_ui_dialogue_name_path(),
            ui_dialogue_skip_path: Self::default_ui_dialogue_skip_path(),
            ui_dialogue_options_path: Self::default_ui_dialogue_options_path(),
            ui_dialogue_option_text_path: Self::default_ui_dialogue_option_text_path(),
            ui_dialogue_option_button_path: Self::default_ui_dialogue_option_button_path(),
            ui_dialogue_default_name_color: Self::default_ui_dialogue_default_name_color(),
            input_pointer_trigger: Self::default_input_pointer_trigger(),
            input_pointer_axis_x: Self::default_input_pointer_axis_x(),
            input_pointer_axis_y: Self::default_input_pointer_axis_y(),
            pointer_image: None,
            pointer_align: Default::default(),
            forced_constant_refresh: Self::default_forced_constant_refresh(),
        }
    }
}

impl VnRenderingConfig {
    fn default_background_camera_scaling_target() -> CompositeScalingTarget {
        CompositeScalingTarget::BothMinimum
    }

    fn default_background_camera_resolution() -> Scalar {
        1080.0
    }

    fn default_background_camera_layer() -> usize {
        1000
    }

    fn default_characters_camera_scaling_target() -> CompositeScalingTarget {
        CompositeScalingTarget::Height
    }

    fn default_characters_camera_resolution() -> Scalar {
        1080.0
    }

    fn default_characters_camera_layer() -> usize {
        1001
    }

    fn default_ui_camera_scaling_target() -> CompositeScalingTarget {
        CompositeScalingTarget::Both
    }

    fn default_ui_camera_resolution() -> Scalar {
        1080.0
    }

    fn default_ui_camera_layer() -> usize {
        1002
    }

    fn default_ui_dialogue_default_theme() -> Option<String> {
        Some("default".to_owned())
    }

    fn default_ui_dialogue_panel_path() -> String {
        "panel".to_owned()
    }

    fn default_ui_dialogue_text_path() -> String {
        "panel/text".to_owned()
    }

    fn default_ui_dialogue_name_path() -> String {
        "panel/name".to_owned()
    }

    fn default_ui_dialogue_skip_path() -> String {
        "panel/skip".to_owned()
    }

    fn default_ui_dialogue_options_path() -> String {
        "panel/options".to_owned()
    }

    fn default_ui_dialogue_option_text_path() -> String {
        "button/text".to_owned()
    }

    fn default_ui_dialogue_option_button_path() -> String {
        "button".to_owned()
    }

    fn default_ui_dialogue_default_name_color() -> Color {
        Color::white()
    }

    fn default_input_pointer_trigger() -> String {
        "mouse-left".to_owned()
    }

    fn default_input_pointer_axis_x() -> String {
        "mouse-x".to_owned()
    }

    fn default_input_pointer_axis_y() -> String {
        "mouse-y".to_owned()
    }

    fn default_forced_constant_refresh() -> bool {
        true
    }
}

impl Prefab for VnRenderingConfig {}

pub struct VnRenderingConfigAsset(VnRenderingConfig);

impl VnRenderingConfigAsset {
    pub fn config(&self) -> &VnRenderingConfig {
        &self.0
    }
}

pub struct VnRenderingConfigAssetProtocol;

impl AssetProtocol for VnRenderingConfigAssetProtocol {
    fn name(&self) -> &str {
        "vn-rendering"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        match VnRenderingConfig::from_prefab_str(&data) {
            Ok(result) => AssetLoadResult::Data(Box::new(VnRenderingConfigAsset(result))),
            Err(error) => AssetLoadResult::Error(format!(
                "Error loading visual novel rendering config asset: {:?}",
                error
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VnStoryRenderer {
    background_camera: Entity,
    characters_camera: Entity,
    ui_camera: Entity,
    characters: HashMap<String, Entity>,
    background: Entity,
    dialogue_ui_element: Entity,
    dialogue_option_template: CompositeUiElement,
    overlay: Entity,
    pointer: Option<Entity>,
}

#[derive(Debug, Clone)]
pub enum VnRenderingManagerError {
    RenderingConfigNotFound(String),
}

#[derive(Debug, Default, Clone)]
pub struct VnRenderingManager {
    configs: HashMap<String, VnRenderingConfig>,
    stories: HashMap<String, VnStoryRenderer>,
    active_config: VnRenderingConfig,
    dirty_config: bool,
}

impl VnRenderingManager {
    pub fn config(&self) -> &VnRenderingConfig {
        &self.active_config
    }

    pub fn register_config(&mut self, name: &str, config: VnRenderingConfig) {
        self.configs.insert(name.to_owned(), config);
    }

    pub fn unregister_config(&mut self, name: &str) -> Option<VnRenderingConfig> {
        self.configs.remove(name)
    }

    pub fn select_config(&mut self, name: &str) -> Result<(), VnRenderingManagerError> {
        if let Some(config) = self.configs.get(name) {
            self.active_config = config.clone();
            self.dirty_config = true;
            Ok(())
        } else {
            Err(VnRenderingManagerError::RenderingConfigNotFound(
                name.to_owned(),
            ))
        }
    }

    pub fn use_default_config(&mut self) {
        self.active_config = Default::default();
        self.dirty_config = true;
    }
}

#[derive(Ignite, Debug, Clone, Copy)]
pub struct PositionCameraAlignment(pub Entity, pub Vec2);

impl Component for PositionCameraAlignment {
    type Storage = VecStorage<Self>;
}

impl PrefabProxy<PositionCameraAlignmentPrefabProxy> for PositionCameraAlignment {
    fn from_proxy_with_extras(
        proxy: PositionCameraAlignmentPrefabProxy,
        named_entities: &HashMap<String, Entity>,
        _: StateToken,
    ) -> Result<Self, PrefabError> {
        if let Some(entity) = named_entities.get(&proxy.0) {
            Ok(Self(*entity, proxy.1))
        } else {
            Err(PrefabError::Custom(format!(
                "Could not find entity named: {}",
                proxy.0
            )))
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PositionCameraAlignmentPrefabProxy(String, Vec2);

impl Prefab for PositionCameraAlignmentPrefabProxy {}

#[derive(Debug, Default)]
pub struct ApplyVisualNovelToCompositeRenderer {
    config_table: HashMap<AssetID, String>,
    dialogue_options_focus_phases: Vec<Scalar>,
}

impl<'s> System<'s> for ApplyVisualNovelToCompositeRenderer {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Read<'s, EntitiesRes>,
        Read<'s, LazyUpdate>,
        ReadExpect<'s, AssetsDatabase>,
        Write<'s, VnStoryManager>,
        Read<'s, CompositeCameraCache>,
        Read<'s, CompositeUiInteractibles>,
        Read<'s, InputController>,
        Write<'s, VnRenderingManager>,
        WriteStorage<'s, CompositeRenderable>,
        WriteStorage<'s, CompositeVisibility>,
        WriteStorage<'s, CompositeRenderAlpha>,
        WriteStorage<'s, CompositeTransform>,
        WriteStorage<'s, PositionCameraAlignment>,
        WriteStorage<'s, CompositeUiElement>,
    );

    // TODO: REFACTOR THIS SHIT
    #[allow(clippy::cognitive_complexity)]
    #[allow(clippy::many_single_char_names)]
    fn run(
        &mut self,
        (
            entities,
            lazy_update,
            assets,
            mut stories,
            camera_cache,
            interactibles,
            input,
            mut rendering,
            mut renderables,
            mut visibilities,
            mut alphas,
            mut transforms,
            mut alignments,
            mut ui_elements,
        ): Self::SystemData,
    ) {
        for id in assets.lately_unloaded_protocol("vn-rendering") {
            if let Some(path) = self.config_table.remove(id) {
                rendering.unregister_config(&path);
            }
        }

        for id in assets.lately_loaded_protocol("vn-rendering") {
            let id = *id;
            let asset = assets
                .asset_by_id(id)
                .expect("trying to use not loaded visual novel rendering config asset");
            let path = asset.path().to_owned();
            let asset = asset
                .get::<VnRenderingConfigAsset>()
                .expect("trying to use non visual novel rendering config asset");
            let config = asset.config().clone();
            rendering.register_config(&path, config);
            self.config_table.insert(id, path);
        }

        let config_changed = rendering.dirty_config;
        rendering.dirty_config = false;

        let to_remove = if config_changed {
            stories.stories_names().collect::<Vec<_>>()
        } else {
            stories.lately_unregistered().collect::<Vec<_>>()
        };
        for id in to_remove {
            if let Some(story) = rendering.stories.remove(id) {
                drop(entities.delete(story.background_camera));
                drop(entities.delete(story.characters_camera));
                drop(entities.delete(story.ui_camera));
                for entity in story.characters.values() {
                    drop(entities.delete(*entity));
                }
                drop(entities.delete(story.background));
                drop(entities.delete(story.dialogue_ui_element));
                drop(entities.delete(story.overlay));
                if let Some(pointer) = story.pointer {
                    drop(entities.delete(pointer));
                }
            }
        }

        let to_add = if config_changed {
            stories.stories_names().collect::<Vec<_>>()
        } else {
            stories.lately_registered().collect::<Vec<_>>()
        };
        for id in to_add {
            if let Some(story) = stories.get(id) {
                let background_camera = lazy_update
                    .create_entity(&entities)
                    .with(Name(format!("vn-camera-background-{}", id).into()))
                    .with(
                        CompositeCamera::with_scaling_target(
                            CompositeScalingMode::CenterAspect,
                            rendering.config().background_camera_scaling_target,
                        )
                        .tag(format!("vn-background-{}", id).into()),
                    )
                    .with(CompositeVisibility(false))
                    .with(CompositeRenderLayer(
                        rendering.config().background_camera_layer,
                    ))
                    .with(CompositeTransform::scale(
                        rendering.config().background_camera_resolution.into(),
                    ))
                    .build();
                let characters_camera = lazy_update
                    .create_entity(&entities)
                    .with(Name(format!("vn-camera-characters-{}", id).into()))
                    .with(
                        CompositeCamera::with_scaling_target(
                            CompositeScalingMode::CenterAspect,
                            rendering.config().characters_camera_scaling_target,
                        )
                        .tag(format!("vn-characters-{}", id).into()),
                    )
                    .with(CompositeVisibility(false))
                    .with(CompositeRenderLayer(
                        rendering.config().characters_camera_layer,
                    ))
                    .with(CompositeTransform::scale(
                        rendering.config().characters_camera_resolution.into(),
                    ))
                    .build();
                let ui_camera = lazy_update
                    .create_entity(&entities)
                    .with(Name(format!("vn-camera-ui-{}", id).into()))
                    .with(
                        CompositeCamera::with_scaling_target(
                            CompositeScalingMode::Aspect,
                            rendering.config().ui_camera_scaling_target,
                        )
                        .tag(format!("vn-ui-{}", id).into()),
                    )
                    .with(CompositeVisibility(false))
                    .with(CompositeRenderLayer(rendering.config().ui_camera_layer))
                    .with(CompositeTransform::scale(
                        rendering.config().ui_camera_resolution.into(),
                    ))
                    .build();
                let characters = story
                    .characters()
                    .map(|(n, _)| {
                        let entity = lazy_update
                            .create_entity(&entities)
                            .with(Tag(format!("vn-characters-{}", id).into()))
                            .with(Name(format!("vn-character-{}-{}", id, n).into()))
                            .with(CompositeRenderAlpha(0.0))
                            .with(CompositeRenderLayer(1))
                            .with(CompositeRenderable(().into()))
                            .with(PositionCameraAlignment(characters_camera, 0.0.into()))
                            .with(CompositeTransform::default())
                            .build();
                        (n.to_owned(), entity)
                    })
                    .collect::<HashMap<_, _>>();
                let background = lazy_update
                    .create_entity(&entities)
                    .with(Tag(format!("vn-background-{}", id).into()))
                    .with(Name(format!("vn-background-{}", id).into()))
                    .with(CompositeRenderable(().into()))
                    .with(CompositeTransform::default())
                    .build();
                let ui_element =
                    if let Some(mut component) = rendering.config().ui_component_template.clone() {
                        component.camera_name = format!("vn-camera-ui-{}", id).into();
                        if let Some(child) =
                            component.find_mut(&rendering.config().ui_dialogue_panel_path)
                        {
                            child.interactive = Some(format!("vn-ui-panel-{}", id).into());
                        }
                        if let Some(child) =
                            component.find_mut(&rendering.config().ui_dialogue_text_path)
                        {
                            child.interactive = Some(format!("vn-ui-text-{}", id).into());
                        }
                        if let Some(child) =
                            component.find_mut(&rendering.config().ui_dialogue_name_path)
                        {
                            child.interactive = Some(format!("vn-ui-name-{}", id).into());
                        }
                        if let Some(child) =
                            component.find_mut(&rendering.config().ui_dialogue_skip_path)
                        {
                            child.interactive = Some(format!("vn-ui-skip-{}", id).into());
                        }
                        component
                    } else {
                        let mut text = CompositeUiElement::default();
                        text.id = Some("text".into());
                        text.theme = rendering
                            .config()
                            .ui_dialogue_default_theme
                            .as_ref()
                            .map(|theme| format!("{}@text", theme).into());
                        text.interactive = Some(format!("vn-ui-text-{}", id).into());
                        text.element_type = UiElementType::Text(
                            Text::new_owned("Verdana".to_owned(), "".to_owned())
                                .size(48.0)
                                .color(Color::white()),
                        );
                        text.padding = UiMargin {
                            left: 64.0,
                            right: 64.0,
                            top: 64.0,
                            bottom: 0.0,
                        };
                        text.left_anchor = 0.0.into();
                        text.right_anchor = 1.0.into();
                        text.top_anchor = 0.0.into();
                        text.bottom_anchor = 1.0.into();
                        let mut name = CompositeUiElement::default();
                        name.id = Some("name".into());
                        name.theme = rendering
                            .config()
                            .ui_dialogue_default_theme
                            .as_ref()
                            .map(|theme| format!("{}@name", theme).into());
                        name.element_type = UiElementType::Text(
                            Text::new_owned("Verdana".to_owned(), "".to_owned())
                                .size(32.0)
                                .color(Color::white()),
                        );
                        name.interactive = Some(format!("vn-ui-name-{}", id).into());
                        name.padding = UiMargin {
                            left: 32.0,
                            right: 32.0,
                            top: 32.0,
                            bottom: 0.0,
                        };
                        name.left_anchor = 0.0.into();
                        name.right_anchor = 1.0.into();
                        name.top_anchor = 0.0.into();
                        name.bottom_anchor = 0.0.into();
                        name.alignment = (0.0.into(), 0.0.into()).into();
                        name.fixed_height = Some(64.0.into());
                        let mut options = CompositeUiElement::default();
                        options.id = Some("options".into());
                        options.element_type = UiElementType::None;
                        options.left_anchor = 0.0.into();
                        options.right_anchor = 1.0.into();
                        options.top_anchor = 0.0.into();
                        options.bottom_anchor = 0.0.into();
                        options.alignment = (0.0.into(), 1.0.into()).into();
                        let mut panel = CompositeUiElement::default();
                        panel.id = Some("panel".into());
                        panel.theme = rendering
                            .config()
                            .ui_dialogue_default_theme
                            .as_ref()
                            .map(|theme| format!("{}@panel", theme).into());
                        panel.interactive = Some(format!("vn-ui-panel-{}", id).into());
                        panel.element_type = UiElementType::Image(Box::new(UiImage::default()));
                        panel.padding = UiMargin {
                            left: 128.0,
                            right: 128.0,
                            top: 0.0,
                            bottom: 32.0,
                        };
                        panel.left_anchor = 0.0.into();
                        panel.right_anchor = 1.0.into();
                        panel.top_anchor = 1.0.into();
                        panel.bottom_anchor = 1.0.into();
                        panel.alignment = (0.0.into(), 1.0.into()).into();
                        panel.fixed_height = Some(256.0.into());
                        panel.children = vec![text, name, options];
                        let mut root = CompositeUiElement::default();
                        root.camera_name = format!("vn-camera-ui-{}", id).into();
                        root.element_type = UiElementType::None;
                        root.left_anchor = 0.0.into();
                        root.right_anchor = 1.0.into();
                        root.top_anchor = 0.0.into();
                        root.bottom_anchor = 1.0.into();
                        root.children = vec![panel];
                        root
                    };
                let dialogue_ui_element = lazy_update
                    .create_entity(&entities)
                    .with(Tag(format!("vn-ui-{}", id).into()))
                    .with(Name(format!("vn-dialogue-{}", id).into()))
                    .with(CompositeRenderAlpha(0.0))
                    .with(CompositeRenderable(().into()))
                    .with(ui_element)
                    .with(CompositeTransform::default())
                    .build();
                let dialogue_option_template = if let Some(mut component) = rendering
                    .config()
                    .ui_dialogue_option_component_template
                    .clone()
                {
                    component.left_anchor = 0.0.into();
                    component.right_anchor = 1.0.into();
                    component.top_anchor = 0.0.into();
                    component.bottom_anchor = 0.0.into();
                    component.alignment = (0.0.into(), 0.0.into()).into();
                    component
                } else {
                    let mut text = CompositeUiElement::default();
                    text.id = Some("text".into());
                    text.theme = rendering
                        .config()
                        .ui_dialogue_default_theme
                        .as_ref()
                        .map(|theme| format!("{}@option-text", theme).into());
                    text.element_type = UiElementType::Text(
                        Text::new_owned("Verdana".to_owned(), "".to_owned())
                            .size(28.0)
                            .color(Color::white()),
                    );
                    text.padding = UiMargin {
                        left: 32.0,
                        right: 32.0,
                        top: 8.0,
                        bottom: 0.0,
                    };
                    text.left_anchor = 0.0.into();
                    text.right_anchor = 1.0.into();
                    text.top_anchor = 0.0.into();
                    text.bottom_anchor = 1.0.into();
                    let mut background = CompositeUiElement::default();
                    background.theme = rendering
                        .config()
                        .ui_dialogue_default_theme
                        .as_ref()
                        .map(|theme| format!("{}@option-background", theme).into());
                    background.element_type = UiElementType::Image(Box::new(UiImage::default()));
                    background.left_anchor = 0.0.into();
                    background.right_anchor = 1.0.into();
                    background.top_anchor = 0.0.into();
                    background.bottom_anchor = 1.0.into();
                    let mut background_focused = CompositeUiElement::default();
                    background_focused.theme = rendering
                        .config()
                        .ui_dialogue_default_theme
                        .as_ref()
                        .map(|theme| format!("{}@option-background-focused", theme).into());
                    background_focused.element_type =
                        UiElementType::Image(Box::new(UiImage::default()));
                    background_focused.left_anchor = 0.0.into();
                    background_focused.right_anchor = 1.0.into();
                    background_focused.top_anchor = 0.0.into();
                    background_focused.bottom_anchor = 1.0.into();
                    let mut button = CompositeUiElement::default();
                    button.id = Some("button".into());
                    button.element_type = UiElementType::None;
                    button.left_anchor = 0.0.into();
                    button.right_anchor = 1.0.into();
                    button.top_anchor = 0.0.into();
                    button.bottom_anchor = 1.0.into();
                    button.children = vec![background, background_focused, text];
                    let mut root = CompositeUiElement::default();
                    root.camera_name = format!("vn-camera-ui-{}", id).into();
                    root.element_type = UiElementType::None;
                    root.padding = UiMargin {
                        left: 256.0,
                        right: 64.0,
                        top: 0.0,
                        bottom: 8.0,
                    };
                    root.left_anchor = 0.0.into();
                    root.right_anchor = 1.0.into();
                    root.top_anchor = 0.0.into();
                    root.bottom_anchor = 0.0.into();
                    root.fixed_height = Some(48.0.into());
                    root.children = vec![button];
                    root
                };
                let overlay_renderable = match &rendering.config().overlay_style {
                    VnRenderingOverlayStyle::Color(color) => {
                        Renderable::FullscreenRectangle(*color)
                    }
                    VnRenderingOverlayStyle::Image(image) => {
                        Image::new_owned(image.to_owned()).align(0.5.into()).into()
                    }
                };
                let overlay = lazy_update
                    .create_entity(&entities)
                    .with(Tag(format!("vn-ui-{}", id).into()))
                    .with(Name(format!("vn-overlay-{}", id).into()))
                    .with(CompositeRenderAlpha(0.0))
                    .with(CompositeRenderLayer(1))
                    .with(CompositeRenderable(overlay_renderable))
                    .with(CompositeCameraAlignment(0.0.into()))
                    .with(CompositeTransform::default())
                    .build();
                let pointer = rendering.config().pointer_image.as_ref().map(|image| {
                    lazy_update
                        .create_entity(&entities)
                        .with(Tag(format!("vn-ui-{}", id).into()))
                        .with(Name(format!("vn-pointer-{}", id).into()))
                        .with(CompositeRenderLayer(2))
                        .with(CompositeRenderable(
                            Image::new_owned(image.to_owned())
                                .align(rendering.config().pointer_align)
                                .into(),
                        ))
                        .with(CompositeTransform::default())
                        .build()
                });
                rendering.stories.insert(
                    id.to_owned(),
                    VnStoryRenderer {
                        background_camera,
                        characters_camera,
                        ui_camera,
                        characters,
                        background,
                        dialogue_ui_element,
                        dialogue_option_template,
                        overlay,
                        pointer,
                    },
                );
            }
        }

        // apply data to renderers.
        for (story_name, story_data) in &rendering.stories {
            if let Some(story) = stories.get(story_name) {
                let show = !story.is_paused() || !rendering.config().hide_on_story_pause;
                if let Some(visibility) = visibilities.get_mut(story_data.background_camera) {
                    visibility.0 = show;
                }
                if let Some(visibility) = visibilities.get_mut(story_data.characters_camera) {
                    visibility.0 = show;
                }
                if let Some(visibility) = visibilities.get_mut(story_data.ui_camera) {
                    visibility.0 = show;
                }
                if !show {
                    continue;
                }

                let show_characters = !story.is_complete() || story.active_scene().phase() < 0.5;
                let refresh = rendering.config().forced_constant_refresh;
                let scene_phase = story.active_scene().phase();
                let update_scene = refresh || story.active_scene().in_progress();
                let scene = if scene_phase < 0.5 {
                    if let Some(name) = story.active_scene().from() {
                        story.scene(name)
                    } else {
                        None
                    }
                } else if scene_phase > 0.5 {
                    if let Some(name) = story.active_scene().to() {
                        story.scene(name)
                    } else {
                        None
                    }
                } else {
                    None
                };

                if update_scene {
                    if let Some(alpha) = alphas.get_mut(story_data.overlay) {
                        alpha.0 = (scene_phase * PI).sin();
                    }
                }

                let background_renderable = if let Some(scene) = &scene {
                    if update_scene || scene.background_style.in_progress() {
                        let from = story.background(scene.background_style.from());
                        let to = story.background(scene.background_style.to());
                        if let (Some(from), Some(to)) = (from, to) {
                            let phase = scene.background_style.phase();
                            let transform_prev = {
                                let [a, b, c, d, e, f] = Mat2d::scale(from.scale.into()).0;
                                Command::Transform(a, b, c, d, e, f)
                            };
                            let transform_next = {
                                let [a, b, c, d, e, f] = Mat2d::scale(to.scale.into()).0;
                                Command::Transform(a, b, c, d, e, f)
                            };
                            Some(Renderable::Commands(vec![
                                Command::Store,
                                transform_prev,
                                Command::Alpha(1.0 - phase),
                                Command::Draw(
                                    Image::new_owned(from.image.to_owned())
                                        .align(0.5.into())
                                        .into(),
                                ),
                                Command::Restore,
                                Command::Store,
                                transform_next,
                                Command::Alpha(phase),
                                Command::Draw(
                                    Image::new_owned(to.image.to_owned())
                                        .align(0.5.into())
                                        .into(),
                                ),
                                Command::Restore,
                            ]))
                        } else {
                            Some(Renderable::None)
                        }
                    } else {
                        None
                    }
                } else {
                    Some(Renderable::None)
                };
                if let Some(background_renderable) = background_renderable {
                    if let Some(renderable) = renderables.get_mut(story_data.background) {
                        renderable.0 = background_renderable;
                    }
                }

                if let Some(scene) = &scene {
                    if update_scene
                        || scene.camera_position.in_progress()
                        || scene.camera_rotation.in_progress()
                    {
                        let position = scene.camera_position.value();
                        let rotation = scene.camera_rotation.value();
                        if let Some(transform) = transforms.get_mut(story_data.background_camera) {
                            transform.set_translation(Vec2::new(position.0, position.1));
                            transform.set_rotation(rotation);
                        }
                        if let Some(transform) = transforms.get_mut(story_data.characters_camera) {
                            transform.set_translation(Vec2::new(position.0, position.1));
                            transform.set_rotation(rotation);
                        }
                    }
                }

                if refresh || story.active_dialogue().in_progress() {
                    let from = story.active_dialogue().from();
                    let to = story.active_dialogue().to();
                    let phase = story.active_dialogue().phase();
                    let (main_alpha, name_alpha, text, name, options) = match (from, to) {
                        (Some(from), Some(to)) => {
                            if phase < 0.5 {
                                (
                                    1.0,
                                    1.0 - phase * 2.0,
                                    from.text.as_str(),
                                    from.character.as_str(),
                                    from.options.as_slice(),
                                )
                            } else {
                                (
                                    1.0,
                                    phase * 2.0,
                                    to.text.as_str(),
                                    to.character.as_str(),
                                    to.options.as_slice(),
                                )
                            }
                        }
                        (None, Some(to)) => (
                            phase,
                            1.0,
                            to.text.as_str(),
                            to.character.as_str(),
                            to.options.as_slice(),
                        ),
                        (Some(from), None) => (
                            1.0 - phase,
                            1.0,
                            from.text.as_str(),
                            from.character.as_str(),
                            from.options.as_slice(),
                        ),
                        (None, None) => (0.0, 1.0, "", "", [].as_ref()),
                    };
                    let name_color = if let Some(c) = story.character(name) {
                        let color = c.name_color();
                        Color::rgb(
                            (color.0 * 255.0).max(0.0).min(255.0) as u8,
                            (color.1 * 255.0).max(0.0).min(255.0) as u8,
                            (color.2 * 255.0).max(0.0).min(255.0) as u8,
                        )
                    } else {
                        Color::white()
                    };
                    if let Some(alpha) = alphas.get_mut(story_data.dialogue_ui_element) {
                        alpha.0 = main_alpha;
                    }
                    if let Some(ui_element) = ui_elements.get_mut(story_data.dialogue_ui_element) {
                        let item_height =
                            if let Some(v) = &story_data.dialogue_option_template.fixed_height {
                                ui_element.calculate_value(v, &[])
                            } else {
                                0.0
                            };
                        if let Some(child) =
                            ui_element.find_mut(&rendering.config().ui_dialogue_options_path)
                        {
                            child.alpha = phase.into();
                            if options.is_empty() {
                                child.fixed_height = None;
                                child.children.clear();
                                child.rebuild();
                            } else {
                                child.fixed_height =
                                    Some((item_height * options.len() as Scalar).into());
                                child.children.clear();
                                for (i, option) in options.iter().enumerate() {
                                    let mut element = story_data.dialogue_option_template.clone();
                                    if let Some(child) = element
                                        .find_mut(&rendering.config().ui_dialogue_option_text_path)
                                    {
                                        if let UiElementType::Text(t) = &mut child.element_type {
                                            t.text = option.text.clone().into();
                                        }
                                    }
                                    let focused_phase = option.focused.phase();
                                    if let Some(child) = element.find_mut(
                                        &rendering.config().ui_dialogue_option_button_path,
                                    ) {
                                        child.state.insert("focused".into(), focused_phase);
                                    }
                                    element.interactive =
                                        Some(format!("vn-ui-option-{}-{}", story_name, i).into());
                                    element.offset.y = (i as Scalar * item_height).into();
                                    child.children.push(element);
                                }
                                child.rebuild();
                            }
                        }
                        if let Some(child) =
                            ui_element.find_mut(&rendering.config().ui_dialogue_name_path)
                        {
                            child.alpha = name_alpha.into();
                            if let UiElementType::Text(t) = &mut child.element_type {
                                t.text = name.to_owned().into();
                                t.color = name_color;
                            }
                            child.rebuild();
                        }
                        if let Some(child) =
                            ui_element.find_mut(&rendering.config().ui_dialogue_text_path)
                        {
                            if let UiElementType::Text(t) = &mut child.element_type {
                                t.text = text.to_owned().into();
                            }
                            child.rebuild();
                        }
                        ui_element.rebuild();
                    }
                }

                for (name, entity) in &story_data.characters {
                    if let Some(character) = story.character(name) {
                        if refresh || character.visibility_anim().in_progress() {
                            if let Some(alpha) = alphas.get_mut(*entity) {
                                alpha.0 = if show_characters {
                                    character.visibility()
                                } else {
                                    0.0
                                };
                            }
                        }
                        if refresh || character.position_anim().in_progress() {
                            if let Some(alignment) = alignments.get_mut(*entity) {
                                let pos = character.position();
                                alignment.1 = Vec2::new(pos.0, pos.1);
                            }
                        }
                        if refresh
                            || character.rotation_anim().in_progress()
                            || character.scale_anim().in_progress()
                        {
                            if let Some(transform) = transforms.get_mut(*entity) {
                                let scl = character.scale();
                                transform.set_rotation(character.rotation());
                                transform.set_scale(Vec2::new(scl.0, scl.1));
                            }
                        }
                        if refresh
                            || character.style_transition().in_progress()
                            || character.alignment_anim().in_progress()
                        {
                            let (from_style, style_phase, to_style) = character.style();
                            let from = character.styles.get(from_style);
                            let to = character.styles.get(to_style);
                            let character_renderable = if let (Some(from), Some(to)) = (from, to) {
                                let align = character.alignment();
                                let align = Vec2::new(align.0, align.1);
                                Renderable::Commands(vec![
                                    Command::Store,
                                    Command::Alpha(1.0 - style_phase),
                                    Command::Draw(
                                        Image::new_owned(from.to_owned()).align(align).into(),
                                    ),
                                    Command::Alpha(style_phase),
                                    Command::Draw(
                                        Image::new_owned(to.to_owned()).align(align).into(),
                                    ),
                                    Command::Restore,
                                ])
                            } else {
                                Renderable::None
                            };
                            if let Some(renderable) = renderables.get_mut(*entity) {
                                renderable.0 = character_renderable;
                            }
                        }
                    } else if let Some(alpha) = alphas.get_mut(*entity) {
                        alpha.0 = 0.0;
                    }
                }
            }

            if let Some(story) = stories.get_mut(story_name) {
                let x = input.axis_or_default(&rendering.config().input_pointer_axis_x);
                let y = input.axis_or_default(&rendering.config().input_pointer_axis_y);
                let point = [x, y].into();

                if let Some(pos) = camera_cache.screen_to_world_space(story_data.ui_camera, point) {
                    if let Some(pointer) = &story_data.pointer {
                        if let Some(transform) = transforms.get_mut(*pointer) {
                            transform.set_translation(pos);
                        }
                    }

                    if story.is_waiting_for_dialogue_option_selection()
                        && story.active_dialogue().is_complete()
                    {
                        let dialogue = story.active_dialogue().to();
                        if let Some(dialogue) = dialogue {
                            let mut selected = None;
                            let mut focused = None;
                            let click = input
                                .trigger_or_default(&rendering.config().input_pointer_trigger)
                                .is_pressed();
                            for i in 0..dialogue.options.len() {
                                if interactibles.does_rect_contains_point(
                                    &format!("vn-ui-option-{}-{}", story_name, i),
                                    pos,
                                ) {
                                    focused = Some(i);
                                    if click {
                                        selected = Some(i);
                                    }
                                }
                            }
                            drop(story.focus_dialogue_option(focused));
                            if let Some(selected) = selected {
                                drop(story.select_dialogue_option(selected));
                            }
                        }
                    }
                }
            }
        }

        for (transform, alignment) in (&mut transforms, &alignments).join() {
            if let Some(size) = camera_cache.calculate_world_size(alignment.0) {
                transform.set_translation(alignment.1 * size);
            }
        }
    }
}
