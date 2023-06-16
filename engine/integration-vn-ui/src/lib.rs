pub mod gui;

use crate::gui::{
    backgrounds::*, characters::*, dialogue::*, game_menu::*, main_menu::*, overlay::*,
    visual_novel_gui,
};
use oxygengine_animation::phase::{Ease, Phase};
use oxygengine_core::{
    app::AppBuilder,
    ecs::{
        pipeline::{PipelineBuilder, PipelineBuilderError},
        AccessType, Comp, ResQuery, Universe, WorldRef,
    },
    prefab::{Prefab as CorePrefab, PrefabComponent, PrefabManager},
};
use oxygengine_user_interface::{
    component::UserInterfaceView,
    raui::{core::prelude::*, material::prelude::*},
    resource::UserInterface,
};
use oxygengine_visual_novel::{
    background::Background, character::Character, dialogue::DialogueOption,
    resource::VnStoryManager, Position,
};
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use crate::{
        gui::{backgrounds::*, characters::*, dialogue::*, game_menu::*, main_menu::*, overlay::*},
        *,
    };
}

fn make_true() -> bool {
    true
}

/// (story ID)
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelSyncUserInterface(#[serde(default)] pub String);

impl CorePrefab for VisualNovelSyncUserInterface {}
impl PrefabComponent for VisualNovelSyncUserInterface {}

pub type ApplyVisualNovelToUserInterfaceSystemResources<'a> = (
    WorldRef,
    &'a mut UserInterface,
    &'a mut VnStoryManager,
    Comp<&'a VisualNovelSyncUserInterface>,
    Comp<&'a UserInterfaceView>,
);

pub fn apply_visual_novel_to_user_interface_system(universe: &mut Universe) {
    let (world, mut ui, mut manager, ..) =
        universe.query_resources::<ApplyVisualNovelToUserInterfaceSystemResources>();

    for (_, (sync, view)) in world
        .query::<(&VisualNovelSyncUserInterface, &UserInterfaceView)>()
        .iter()
    {
        if let Some(data) = ui.get_mut(view.app_id()) {
            if let Some(story) = manager.get(&sync.0) {
                if story.is_dirty() {
                    data.application.mark_dirty();
                }
            }
            for (_, msg) in data.signals_received() {
                if let Some(VisualNovelSignal { story, action }) = msg.as_any().downcast_ref() {
                    if let Some(story) = manager.get_mut(story) {
                        match action {
                            VisualNovelAction::None => {}
                            VisualNovelAction::SelectDialogueOption(None) => {
                                story.unselect_dialogue_option();
                            }
                            VisualNovelAction::FocusDialogueOption(index) => {
                                let _ = story.focus_dialogue_option(*index);
                            }
                            VisualNovelAction::SelectDialogueOption(Some(index)) => {
                                let _ = story.select_dialogue_option(*index);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(MessageData, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelSignal {
    #[serde(default)]
    pub story: String,
    #[serde(default)]
    pub action: VisualNovelAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualNovelAction {
    None,
    FocusDialogueOption(Option<usize>),
    SelectDialogueOption(Option<usize>),
}

impl Default for VisualNovelAction {
    fn default() -> Self {
        Self::None
    }
}

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelStoryUsed(pub String);

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelOverlayPhase(
    #[serde(default = "VisualNovelOverlayPhase::default_phase")] pub Phase,
);

impl Default for VisualNovelOverlayPhase {
    fn default() -> Self {
        Self(Self::default_phase())
    }
}

impl VisualNovelOverlayPhase {
    fn default_phase() -> Phase {
        Phase::ease(Ease::InOutCubic, 0.0..1.0, 0.0..1.0).unwrap()
    }
}

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelOverlayMaterial(#[serde(default)] pub ImageBoxMaterial);

#[derive(PropsData, Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct VisualNovelCharacterContentAlignment(pub Option<Position>);

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
pub enum VisualNovelTextTransition {
    Instant,
    Fade,
    Unfold,
}

impl Default for VisualNovelTextTransition {
    fn default() -> Self {
        Self::Fade
    }
}

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelDialogueOptionsList(pub VerticalBoxProps);

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelDialogueMessageLayout(pub Option<ContentBoxItemLayout>);

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelDialogueCharacterLayout(pub Option<ContentBoxItemLayout>);

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelDialogueTextLayout(pub Option<ContentBoxItemLayout>);

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelDialogueOptionsLayout(pub Option<ContentBoxItemLayout>);

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelDialogueMessageThemed {
    #[serde(default)]
    pub use_main_color: bool,
    #[serde(default)]
    pub color: ThemeColor,
    #[serde(default)]
    pub background_variant: Option<String>,
    #[serde(default)]
    pub text_variant: Option<String>,
    #[serde(default = "VisualNovelDialogueMessageThemed::default_margin")]
    pub margin: Rect,
}

impl Default for VisualNovelDialogueMessageThemed {
    fn default() -> Self {
        Self {
            use_main_color: false,
            color: Default::default(),
            background_variant: None,
            text_variant: None,
            margin: Self::default_margin(),
        }
    }
}

impl VisualNovelDialogueMessageThemed {
    fn default_margin() -> Rect {
        64.0.into()
    }
}

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelDialogueCharacterThemed {
    #[serde(default)]
    pub use_main_color: bool,
    #[serde(default)]
    pub color: ThemeColor,
    #[serde(default)]
    pub background_variant: Option<String>,
    #[serde(default)]
    pub text_variant: Option<String>,
    #[serde(default = "VisualNovelDialogueCharacterThemed::default_margin")]
    pub margin: Rect,
}

impl Default for VisualNovelDialogueCharacterThemed {
    fn default() -> Self {
        Self {
            use_main_color: false,
            color: Default::default(),
            margin: Self::default_margin(),
            background_variant: None,
            text_variant: None,
        }
    }
}

impl VisualNovelDialogueCharacterThemed {
    fn default_margin() -> Rect {
        Rect {
            left: 32.0,
            right: 32.0,
            top: 18.0,
            bottom: 18.0,
        }
    }
}

#[derive(PropsData, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelDialogueOptionThemed {
    #[serde(default = "VisualNovelDialogueOptionThemed::default_height")]
    pub height: SizeBoxSizeValue,
    #[serde(default)]
    pub use_main_color: bool,
    #[serde(default)]
    pub color_default: ThemeColor,
    #[serde(default = "VisualNovelDialogueOptionThemed::default_color_focused")]
    pub color_focused: ThemeColor,
    #[serde(default)]
    pub button_variant: Option<String>,
    #[serde(default)]
    pub text_variant_default: Option<String>,
    #[serde(default)]
    pub text_variant_focused: Option<String>,
    #[serde(default = "VisualNovelDialogueOptionThemed::default_margin")]
    pub margin: Rect,
}

impl Default for VisualNovelDialogueOptionThemed {
    fn default() -> Self {
        Self {
            height: Self::default_height(),
            use_main_color: false,
            color_default: Default::default(),
            color_focused: Self::default_color_focused(),
            button_variant: None,
            text_variant_default: None,
            text_variant_focused: None,
            margin: Self::default_margin(),
        }
    }
}

impl VisualNovelDialogueOptionThemed {
    fn default_height() -> SizeBoxSizeValue {
        SizeBoxSizeValue::Exact(48.0)
    }

    fn default_color_focused() -> ThemeColor {
        ThemeColor::Primary
    }

    fn default_margin() -> Rect {
        Rect {
            left: 32.0,
            right: 32.0,
            top: 14.0,
            bottom: 14.0,
        }
    }
}

#[derive(PropsData, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct VisualNovelGuiProps {
    #[serde(default = "make_true")]
    pub overlay: bool,
    #[serde(default = "make_true")]
    pub main_menu: bool,
    #[serde(default = "make_true")]
    pub game_menu: bool,
    #[serde(default = "make_true")]
    pub dialogue: bool,
    #[serde(default = "make_true")]
    pub characters: bool,
    #[serde(default = "make_true")]
    pub backgrounds: bool,
}

impl Default for VisualNovelGuiProps {
    fn default() -> Self {
        Self {
            overlay: true,
            main_menu: true,
            game_menu: true,
            dialogue: true,
            characters: true,
            backgrounds: true,
        }
    }
}

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelBackgroundsProps {
    #[serde(default)]
    pub phase: Scalar,
    #[serde(default)]
    pub from: Option<Background>,
    #[serde(default)]
    pub to: Option<Background>,
}

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelCharacterProps(#[serde(default)] pub Character);

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelDialogueMessageProps {
    #[serde(default)]
    pub character: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub container_alpha: Scalar,
    #[serde(default)]
    pub message_alpha: Scalar,
}

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelDialogueOptionsProps {
    #[serde(default)]
    pub options: Vec<DialogueOption>,
    #[serde(default)]
    pub alpha: Scalar,
}

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct VisualNovelDialogueOptionProps {
    #[serde(default)]
    pub index: usize,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub alpha: Scalar,
    #[serde(default)]
    pub focused: bool,
    #[serde(default)]
    pub focus_phase: Scalar,
}

pub fn ui_setup(app: &mut Application) {
    app.register_props::<VisualNovelStoryUsed>("VisualNovelStoryUsed");
    app.register_props::<VisualNovelOverlayMaterial>("VisualNovelOverlayMaterial");
    app.register_props::<VisualNovelCharacterContentAlignment>(
        "VisualNovelCharacterContentAlignment",
    );
    app.register_props::<VisualNovelTextTransition>("VisualNovelTextTransition");
    app.register_props::<VisualNovelDialogueOptionsList>("VisualNovelDialogueOptionsList");
    app.register_props::<VisualNovelDialogueMessageLayout>("VisualNovelDialogueMessageLayout");
    app.register_props::<VisualNovelDialogueCharacterLayout>("VisualNovelDialogueCharacterLayout");
    app.register_props::<VisualNovelDialogueTextLayout>("VisualNovelDialogueTextLayout");
    app.register_props::<VisualNovelDialogueOptionsLayout>("VisualNovelDialogueOptionsLayout");
    app.register_props::<VisualNovelDialogueMessageThemed>("VisualNovelDialogueMessageThemed");
    app.register_props::<VisualNovelDialogueCharacterThemed>("VisualNovelDialogueCharacterThemed");
    app.register_props::<VisualNovelDialogueOptionThemed>("VisualNovelDialogueOptionThemed");
    app.register_props::<VisualNovelGuiProps>("VisualNovelGuiProps");
    app.register_props::<VisualNovelBackgroundsProps>("VisualNovelBackgroundsProps");
    app.register_props::<VisualNovelCharacterProps>("VisualNovelCharacterProps");
    app.register_props::<VisualNovelDialogueMessageProps>("VisualNovelDialogueMessageProps");
    app.register_props::<VisualNovelDialogueOptionsProps>("VisualNovelDialogueOptionsProps");
    app.register_props::<VisualNovelDialogueOptionProps>("VisualNovelDialogueOptionProps");

    app.register_component("visual_novel_gui", visual_novel_gui);
    app.register_component(
        "visual_novel_overlay_container",
        visual_novel_overlay_container,
    );
    app.register_component("visual_novel_overlay", visual_novel_overlay);
    app.register_component("visual_novel_main_menu", visual_novel_main_menu);
    app.register_component("visual_novel_game_menu", visual_novel_game_menu);
    app.register_component(
        "visual_novel_dialogue_container",
        visual_novel_dialogue_container,
    );
    app.register_component(
        "visual_novel_dialogue_message",
        visual_novel_dialogue_message,
    );
    app.register_component(
        "visual_novel_dialogue_options",
        visual_novel_dialogue_options,
    );
    app.register_component("visual_novel_dialogue_option", visual_novel_dialogue_option);
    app.register_component(
        "visual_novel_characters_container",
        visual_novel_characters_container,
    );
    app.register_component("visual_novel_character", visual_novel_character);
    app.register_component(
        "visual_novel_backgrounds_container",
        visual_novel_backgrounds_container,
    );
    app.register_component("visual_novel_backgrounds", visual_novel_backgrounds);
}

pub fn bundle_installer<PB, Q>(
    builder: &mut AppBuilder<PB>,
    _: (),
) -> Result<(), PipelineBuilderError>
where
    PB: PipelineBuilder,
    Q: AccessType + ResQuery + 'static,
{
    builder.install_system::<ApplyVisualNovelToUserInterfaceSystemResources>(
        "apply-visual-novel-to-user-interface",
        apply_visual_novel_to_user_interface_system,
        &["vn-story"],
    )?;
    Ok(())
}

pub fn prefabs_installer(prefabs: &mut PrefabManager) {
    prefabs
        .register_component_factory::<VisualNovelSyncUserInterface>("VisualNovelSyncUserInterface");
}
