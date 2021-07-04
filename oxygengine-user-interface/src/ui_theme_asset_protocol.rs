use core::{
    assets::protocol::{AssetLoadResult, AssetProtocol},
    prefab::Prefab,
};
use raui_core::{
    widget::{
        unit::text::{TextBoxDirection, TextBoxFont, TextBoxHorizontalAlign, TextBoxVerticalAlign},
        utils::Color,
    },
    Scalar,
};
use raui_material::theme::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::from_utf8};

#[derive(Debug, Serialize, Deserialize)]
pub enum UiTheme {
    AllWhite(#[serde(default)] ThemePropsExtras),
    Flat {
        #[serde(default)]
        default: Color,
        #[serde(default)]
        primary: Color,
        #[serde(default)]
        secondary: Color,
        #[serde(default)]
        background: Color,
        #[serde(default)]
        extras: ThemePropsExtras,
    },
    FlatLight(#[serde(default)] ThemePropsExtras),
    FlatDark(#[serde(default)] ThemePropsExtras),
    Custom(#[serde(default)] ThemeProps),
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::AllWhite(Default::default())
    }
}

impl UiTheme {
    pub fn props(&self) -> ThemeProps {
        match self {
            Self::AllWhite(extras) => Self::merge(new_all_white_theme(), extras),
            Self::Flat {
                default,
                primary,
                secondary,
                background,
                extras,
            } => Self::merge(
                make_default_theme(*default, *primary, *secondary, *background),
                extras,
            ),
            Self::FlatLight(extras) => Self::merge(new_light_theme(), extras),
            Self::FlatDark(extras) => Self::merge(new_dark_theme(), extras),
            Self::Custom(props) => props.clone(),
        }
    }

    pub fn merge(mut props: ThemeProps, extras: &ThemePropsExtras) -> ThemeProps {
        extras.active_colors.merge_to(&mut props.active_colors);
        extras
            .background_colors
            .merge_to(&mut props.background_colors);
        props
            .content_backgrounds
            .extend(extras.content_backgrounds.clone());
        for (k, v) in &extras.button_backgrounds {
            v.merge_to(props.button_backgrounds.entry(k.to_owned()).or_default());
        }
        props
            .icons_level_sizes
            .extend(extras.icons_level_sizes.clone());
        for (k, v) in &extras.text_variants {
            v.merge_to(props.text_variants.entry(k.to_owned()).or_default());
        }
        for (k, v) in &extras.text_families {
            make_text_family(k, v, &mut props.text_variants);
        }
        for (k, v) in &extras.switch_variants {
            v.merge_to(props.switch_variants.entry(k.to_owned()).or_default());
        }
        props
            .modal_shadow_variants
            .extend(extras.modal_shadow_variants.clone());
        props
    }
}

impl Prefab for UiTheme {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemeColorSetExtras {
    #[serde(default)]
    pub main: Option<Color>,
    #[serde(default)]
    pub light: Option<Color>,
    #[serde(default)]
    pub dark: Option<Color>,
}

impl ThemeColorSetExtras {
    pub fn merge_to(&self, other: &mut ThemeColorSet) {
        if let Some(v) = self.main {
            other.main = v;
        }
        if let Some(v) = self.light {
            other.light = v;
        }
        if let Some(v) = self.dark {
            other.dark = v;
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemeColorsExtras {
    #[serde(default)]
    pub default: ThemeColorSetExtras,
    #[serde(default)]
    pub primary: ThemeColorSetExtras,
    #[serde(default)]
    pub secondary: ThemeColorSetExtras,
}

impl ThemeColorsExtras {
    pub fn merge_to(&self, other: &mut ThemeColors) {
        self.default.merge_to(&mut other.default);
        self.primary.merge_to(&mut other.primary);
        self.secondary.merge_to(&mut other.secondary);
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemeColorsBundleExtras {
    #[serde(default)]
    pub main: ThemeColorsExtras,
    #[serde(default)]
    pub contrast: ThemeColorsExtras,
}

impl ThemeColorsBundleExtras {
    pub fn merge_to(&self, other: &mut ThemeColorsBundle) {
        self.main.merge_to(&mut other.main);
        self.contrast.merge_to(&mut other.contrast);
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemedTextMaterialExtras {
    #[serde(default)]
    pub horizontal_align: Option<TextBoxHorizontalAlign>,
    #[serde(default)]
    pub vertical_align: Option<TextBoxVerticalAlign>,
    #[serde(default)]
    pub direction: Option<TextBoxDirection>,
    #[serde(default)]
    pub font: Option<TextBoxFont>,
}

impl ThemedTextMaterialExtras {
    pub fn merge_to(&self, other: &mut ThemedTextMaterial) {
        if let Some(v) = self.horizontal_align {
            other.horizontal_align = v;
        }
        if let Some(v) = self.vertical_align {
            other.vertical_align = v;
        }
        if let Some(v) = self.direction {
            other.direction = v;
        }
        if let Some(v) = &self.font {
            other.font = v.clone();
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemedButtonMaterialExtras {
    #[serde(default)]
    pub default: Option<ThemedImageMaterial>,
    #[serde(default)]
    pub selected: Option<ThemedImageMaterial>,
    #[serde(default)]
    pub trigger: Option<ThemedImageMaterial>,
}

impl ThemedButtonMaterialExtras {
    pub fn merge_to(&self, other: &mut ThemedButtonMaterial) {
        if let Some(v) = &self.default {
            other.default = v.clone();
        }
        if let Some(v) = &self.selected {
            other.selected = v.clone();
        }
        if let Some(v) = &self.trigger {
            other.trigger = v.clone();
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemedSwitchMaterialExtras {
    #[serde(default)]
    pub on: Option<ThemedImageMaterial>,
    #[serde(default)]
    pub off: Option<ThemedImageMaterial>,
}

impl ThemedSwitchMaterialExtras {
    pub fn merge_to(&self, other: &mut ThemedSwitchMaterial) {
        if let Some(v) = &self.on {
            other.on = v.clone();
        }
        if let Some(v) = &self.off {
            other.off = v.clone();
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ThemePropsExtras {
    #[serde(default)]
    pub active_colors: ThemeColorsBundleExtras,
    #[serde(default)]
    pub background_colors: ThemeColorsBundleExtras,
    #[serde(default)]
    pub content_backgrounds: HashMap<String, ThemedImageMaterial>,
    #[serde(default)]
    pub button_backgrounds: HashMap<String, ThemedButtonMaterialExtras>,
    #[serde(default)]
    pub icons_level_sizes: Vec<Scalar>,
    #[serde(default)]
    pub text_variants: HashMap<String, ThemedTextMaterialExtras>,
    #[serde(default)]
    pub text_families: HashMap<String, ThemedTextMaterial>,
    #[serde(default)]
    pub switch_variants: HashMap<String, ThemedSwitchMaterialExtras>,
    #[serde(default)]
    pub modal_shadow_variants: HashMap<String, Color>,
}

pub struct UiThemeAsset(UiTheme);

impl UiThemeAsset {
    pub fn get(&self) -> &UiTheme {
        &self.0
    }
}

pub struct UiThemeAssetProtocol;

impl AssetProtocol for UiThemeAssetProtocol {
    fn name(&self) -> &str {
        "ui-theme"
    }

    fn on_load(&mut self, data: Vec<u8>) -> AssetLoadResult {
        let data = from_utf8(&data).unwrap();
        match UiTheme::from_prefab_str(data) {
            Ok(result) => AssetLoadResult::Data(Box::new(UiThemeAsset(result))),
            Err(error) => AssetLoadResult::Error(format!(
                "Error loading user interface theme asset: {:?}",
                error
            )),
        }
    }
}

fn make_text_family(
    base_id: &str,
    base_material: &ThemedTextMaterial,
    text_variants: &mut HashMap<String, ThemedTextMaterial>,
) {
    {
        let mut material = base_material.clone();
        material.font.size *= 2.0;
        text_variants.insert(format!("{}1", base_id), material);
    }
    {
        let mut material = base_material.clone();
        material.font.size *= 1.5;
        text_variants.insert(format!("{}2", base_id), material);
    }
    {
        let mut material = base_material.clone();
        material.font.size *= 1.17;
        text_variants.insert(format!("{}3", base_id), material);
    }
    {
        text_variants.insert(format!("{}4", base_id), base_material.clone());
    }
    {
        let mut material = base_material.clone();
        material.font.size *= 0.83;
        text_variants.insert(format!("{}5", base_id), material);
    }
    {
        let mut material = base_material.clone();
        material.font.size *= 0.67;
        text_variants.insert(format!("{}6", base_id), material);
    }
    text_variants.insert(base_id.to_owned(), base_material.clone());
}
